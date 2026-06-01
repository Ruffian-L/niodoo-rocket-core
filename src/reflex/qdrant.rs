//! Minimal Qdrant backend for MistakeReflex events (behind `qdrant` feature).
//!
//! Dual-write philosophy (same as the main tree):
//! - JSONL is always the source of truth / audit trail.
//! - Qdrant is the fast, queryable, cross-process durable index.
//! - On write: append to JSONL + upsert to Qdrant.
//! - On fresh start with Qdrant: can load the entire set of corrections.

use super::MistakeReflexEvent;
use anyhow::{Context, Result};
use qdrant_client::qdrant::{
    point_id::PointIdOptions, CreateCollection, Distance, PointId, PointStruct, ScrollPoints,
    UpsertPoints, Value, VectorParams, Vectors, VectorsConfig, WithPayloadSelector,
    with_payload_selector,
};
use qdrant_client::{Payload, Qdrant};
use serde_json::Map as JsonMap;
use std::collections::HashMap;
use uuid::Uuid;

/// Deterministic UUID namespace so the same event.id always maps to the same Qdrant point.
const MVP_QDRANT_UUID_NS: Uuid = Uuid::from_bytes([
    0x6e, 0x69, 0x6f, 0x64, 0x6f, 0x6f, 0x5f, 0x6d, 0x76, 0x70, 0x5f, 0x71, 0x64, 0x72, 0x61, 0x6e,
]);

#[derive(Clone, Debug)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub collection: String,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            // Aligned to ruff's actual running memory stack (memory-up)
            url: "http://127.0.0.1:6360".to_string(),
            api_key: None,
            // Aligned to the user's real stack schema (see docs/LEDGERS.md)
            collection: "niodoo-corrections".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct QdrantReflexBackend {
    client: Qdrant,
    cfg: QdrantConfig,
}

impl QdrantReflexBackend {
    pub async fn new(cfg: QdrantConfig) -> Result<Self> {
        // Matches the user's proven Python stack (index_remember.py):
        // - http://127.0.0.1:6360
        // - prefer_grpc=False equivalent (we stay on REST/HTTP)
        // - short timeout, best-effort everything, never hard-fail the MVP
        let mut builder = Qdrant::from_url(&cfg.url);
        if let Some(ref key) = cfg.api_key {
            if !key.is_empty() {
                builder = builder.api_key(key.clone());
            }
        }

        // Short timeout like their scripts (they use timeout=10)
        let client = builder
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .with_context(|| format!("failed to connect to Qdrant at {}", cfg.url))?;

        // Ensure collection exists (simple 0-dim vector since we don't use ANN search yet)
        // Best-effort collection creation. User's custom memory stack (memory-up on 6360)
        // may have restrictions, different vector configs, or already have the collection.
        // We don't want this to hard-fail the whole MVP.
        let collections = match client.list_collections().await {
            Ok(c) => c.collections,
            Err(e) => {
                eprintln!("[qdrant] Warning: could not list collections: {e}. Proceeding anyway.");
                vec![]
            }
        };

        let exists = collections.iter().any(|c| c.name == cfg.collection);

        if !exists {
            let create_result = client
                .create_collection(CreateCollection {
                    collection_name: cfg.collection.clone(),
                    vectors_config: Some(VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            VectorParams {
                                size: 1,
                                distance: Distance::Cosine.into(),
                                ..Default::default()
                            },
                        )),
                    }),
                    ..Default::default()
                })
                .await;

            if let Err(e) = create_result {
                eprintln!("[qdrant] Warning: could not auto-create collection '{}': {e}", cfg.collection);
                eprintln!("[qdrant] You may need to create it manually or it may already exist under a different config.");
            }
        }

        Ok(Self { client, cfg })
    }

    /// Upsert a single reflex event into Qdrant.
    /// Uses deterministic ID based on event.id so it's idempotent.
    /// On this custom memory stack, transport errors are common — we make it best-effort.
    pub async fn upsert_event(&self, event: &MistakeReflexEvent) -> Result<()> {
        // Try the official client first (may hit h2 issues on custom stacks)
        let point = point_from_event(event, &self.cfg.collection);
        let req = UpsertPoints {
            collection_name: self.cfg.collection.clone(),
            wait: Some(true),
            points: vec![point],
            ..Default::default()
        };
        match self.client.upsert_points(req).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                eprintln!("[qdrant] Official client failed (expected on custom 6360 stack): {}", e);
                eprintln!("[qdrant] Falling back to raw REST (matching your Python indexer pattern)...");
            }
        }

        // Raw REST fallback using reqwest — direct HTTP to their 6360, no gRPC, no version check drama
        self.upsert_via_rest(event).await
    }

    async fn upsert_via_rest(&self, event: &MistakeReflexEvent) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()?;

        let point_id = format!("{:?}", uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, event.id.as_bytes())); // simple deterministic

        // Minimal point for their stack (they mostly care about payload + vectors from their embedder)
        let body = serde_json::json!({
            "points": [{
                "id": point_id,
                "payload": event,
                "vector": [0.0]  // dummy; real vectors come from their 8302 embed service in the indexer path
            }]
        });

        let url = format!("{}/collections/{}/points?wait=true", self.cfg.url.trim_end_matches('/'), self.cfg.collection);

        let resp = client.put(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            if text.contains("doesn't exist") {
                println!("[qdrant] Collection missing — attempting raw REST create (1-dim dummy vector for now)...");
                self.create_collection_via_rest().await?;
                // retry upsert once
                let resp2 = client.put(&url).json(&body).send().await?;
                if resp2.status().is_success() {
                    println!("[qdrant] Raw REST upsert succeeded after creating collection on your 6360 stack");
                    return Ok(());
                }
            }
            anyhow::bail!("raw REST upsert failed: {}", text);
        }

        println!("[qdrant] Raw REST upsert succeeded to {} on your 6360 stack", self.cfg.collection);
        Ok(())
    }

    async fn create_collection_via_rest(&self) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()?;

        let url = format!("{}/collections/{}", self.cfg.url.trim_end_matches('/'), self.cfg.collection);

        let body = serde_json::json!({
            "vectors": {
                "size": 1,
                "distance": "Cosine"
            }
        });

        let resp = client.put(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("raw create collection failed: {}", text);
        }

        println!("[qdrant] Collection {} created via raw REST on your stack", self.cfg.collection);
        Ok(())
    }

    /// Load ALL reflex events currently stored in the Qdrant collection.
    /// This is what a fresh process would call on startup.
    /// Made tolerant for custom stacks.
    pub async fn load_all_events(&self) -> Result<Vec<MistakeReflexEvent>> {
        let mut events = Vec::new();
        let mut next_offset: Option<PointId> = None;
        let limit: u32 = 256;

        loop {
            let req = ScrollPoints {
                collection_name: self.cfg.collection.clone(),
                limit: Some(limit),
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(with_payload_selector::SelectorOptions::Enable(true)),
                }),
                with_vectors: Some(false.into()),
                offset: next_offset.clone(),
                ..Default::default()
            };

            match self.client.scroll(req).await {
                Ok(resp) => {
                    let has_more = resp.next_page_offset.is_some() && !resp.result.is_empty();
                    for point in resp.result {
                        if let Some(event) = event_from_point(&point) {
                            events.push(event);
                        }
                    }
                    next_offset = resp.next_page_offset;
                    if !has_more {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[qdrant] Warning: load from {} failed: {}", self.cfg.collection, e);
                    eprintln!("[qdrant] Returning whatever we have so far ({} events).", events.len());
                    break;
                }
            }
        }

        Ok(events)
    }
}

// ── conversion helpers ──────────────────────────────────────────────────

fn point_id_for(id: &str) -> PointId {
    let uuid = Uuid::new_v5(&MVP_QDRANT_UUID_NS, id.as_bytes());
    PointId {
        point_id_options: Some(PointIdOptions::Uuid(uuid.to_string())),
    }
}

fn point_from_event(event: &MistakeReflexEvent, _collection: &str) -> PointStruct {
    // Store the entire event as payload (JSON)
    let payload_value = serde_json::to_value(event).unwrap_or(serde_json::Value::Null);
    let payload: Payload = match payload_value {
        serde_json::Value::Object(map) => {
            let entries: HashMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, json_to_qdrant_value(v))).collect();
            entries.into()
        }
        _ => Payload::new(),
    };

    // Dummy 1-dim vector (we're not using vector search yet, just the payload store)
    let dummy_vec: Vec<f32> = vec![0.0];

    PointStruct {
        id: Some(point_id_for(&event.id)),
        vectors: Some(Vectors {
            vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                dummy_vec.into(),
            )),
        }),
        payload: payload.into(),
    }
}

fn event_from_point(point: &qdrant_client::qdrant::RetrievedPoint) -> Option<MistakeReflexEvent> {
    let payload_map = &point.payload;
    let mut serde_map = JsonMap::new();

    for (k, v) in payload_map {
        serde_map.insert(k.clone(), qdrant_value_to_json(v));
    }

    serde_json::from_value(serde_json::Value::Object(serde_map)).ok()
}

// Very small JSON <-> Qdrant Value bridge (good enough for our event shape)
fn json_to_qdrant_value(v: serde_json::Value) -> Value {
    use qdrant_client::qdrant::value::Kind;

    match v {
        serde_json::Value::Null => Value { kind: Some(Kind::NullValue(0)) },
        serde_json::Value::Bool(b) => Value { kind: Some(Kind::BoolValue(b)) },
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value { kind: Some(Kind::IntegerValue(i)) }
            } else if let Some(f) = n.as_f64() {
                Value { kind: Some(Kind::DoubleValue(f)) }
            } else {
                Value { kind: Some(Kind::NullValue(0)) }
            }
        }
        serde_json::Value::String(s) => Value { kind: Some(Kind::StringValue(s)) },
        serde_json::Value::Array(arr) => {
            let vals: Vec<Value> = arr.into_iter().map(json_to_qdrant_value).collect();
            Value { kind: Some(Kind::ListValue(qdrant_client::qdrant::ListValue { values: vals })) }
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value> = obj
                .into_iter()
                .map(|(k, v)| (k, json_to_qdrant_value(v)))
                .collect();
            Value { kind: Some(Kind::StructValue(qdrant_client::qdrant::Struct { fields: map })) }
        }
    }
}

fn qdrant_value_to_json(v: &Value) -> serde_json::Value {
    use qdrant_client::qdrant::value::Kind;

    match &v.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(*b),
        Some(Kind::IntegerValue(i)) => serde_json::Value::Number((*i).into()),
        Some(Kind::DoubleValue(f)) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s.clone()),
        Some(Kind::ListValue(list)) => {
            serde_json::Value::Array(list.values.iter().map(qdrant_value_to_json).collect())
        }
        Some(Kind::StructValue(s)) => {
            let map: JsonMap<_, _> = s
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), qdrant_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        None => serde_json::Value::Null,
    }
}