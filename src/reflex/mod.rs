//! Minimal Mistake Reflex loader + influence engine for the Niodoo MVP.
//!
//! This is a deliberately small, clean extraction of the pieces that produced
//! real behavior change in the gamma artifact triage and 304-event claims ledger
//! experiments (May 2026).
//!
//! Focus: text-hint influence mode on "gmms:semantic_correction_slice" events.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[cfg(feature = "qdrant")]
mod qdrant;
#[cfg(feature = "qdrant")]
pub use qdrant::{QdrantConfig, QdrantReflexBackend};

mod repetition;
pub use repetition::{
    evaluate_for_escalation, RepetitionContext, RepetitionEscalation, RollingTelemetryWindow,
    TelemetryObservation,
};

mod repetition;
pub use repetition::{
    RepetitionContext, RepetitionEscalation, RollingTelemetryWindow, TelemetryObservation,
};

/// Minimal event shape we actually need for the winning text-hint influence path.
/// This matches the claims_corpus_ledger_20260508 format (and gamma policy ledgers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistakeReflexEvent {
    pub id: String,
    pub domain: String,
    #[serde(default)]
    pub trigger_terms: Vec<String>,
    #[serde(default)]
    pub bad_reflex: String,
    #[serde(default)]
    pub corrected_reflex: String,
    #[serde(default)]
    pub episodic_correction: Option<String>,
    #[serde(default)]
    pub evidence_requirement: String,
    #[serde(default)]
    pub rejected_surfaces: Vec<String>,
    #[serde(default)]
    pub accepted_surfaces: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_confidence() -> f32 {
    0.75
}

/// A hint ready to be used for influence (text-hint mode).
#[derive(Debug, Clone)]
pub struct ReflexHint {
    pub event_id: String,
    pub trigger_terms: Vec<String>,
    pub corrected_reflex: String,
    pub score: f32,
}

/// Signals extracted from logs/telemetry that indicate the model was in a
/// repetitive failure state during generation.
///
/// Historical pain point: pure surface/text matching ("next matching" on
/// rejected vs accepted surfaces) has always been the bane — fragile,
/// lots of special cases, easy to miss real repetitions or false-positive.
///
/// This struct lets us feed stronger, more automatic signals from the
/// actual run logs (ghost pulls, internal monitors, repeated request tags
/// without progress, etc.) into the decision of "this was a repetitive
/// failure worth escalating the reflex for".
#[derive(Debug, Clone, Default)]
pub struct RepetitionContext {
    /// Maximum ghost_pull_delta_norm observed in the window/turn.
    /// High sustained values (especially near 10.0) while producing
    /// wrong/confident output is one of the strongest signals we have
    /// from the gamma runs.
    pub max_ghost_pull: Option<f32>,

    /// How many times an [INTERNAL MONITOR: ... LOGICALLY FLAWED] style
    /// message appeared in the generated tokens.
    pub internal_monitor_flawed_count: u32,

    /// Number of [REQUEST: SPIKE] / [REQUEST: EXPLORE] etc. emitted
    /// without corresponding evidence or earned answer in the same window.
    pub repeated_request_tags_without_progress: u32,

    /// Whether the turn/segment showed high struggle (force application,
    /// no lock stop when expected, etc.).
    pub high_struggle_without_evidence: bool,

    /// Optional free-form notes from log analysis.
    pub notes: Option<String>,
}

impl RepetitionContext {
    /// Simple heuristic: strong enough log signals that we should treat
    /// this as a repetitive failure even if surface text matching is weak.
    pub fn indicates_repetitive_failure(&self) -> bool {
        let high_pull = self.max_ghost_pull.unwrap_or(0.0) > 3.0;
        let monitors = self.internal_monitor_flawed_count >= 1;
        let requests = self.repeated_request_tags_without_progress >= 2;
        let struggle = self.high_struggle_without_evidence;

        (high_pull && (monitors || requests)) || (struggle && monitors)
    }
}

/// One observation from the generation/telemetry stream.
/// This is what goes into the rolling window.
#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,
    /// Ghost pull / steering struggle signal (from physics layer)
    pub ghost_pull: f32,
    /// Did an [INTERNAL MONITOR ... LOGICALLY FLAWED] style marker appear?
    pub internal_monitor_flawed: bool,
    /// How many [REQUEST: SPIKE] / [REQUEST: EXPLORE] etc. in this step/window
    pub request_spike_count: u32,
    /// Explicit "no progress" marker from logs (e.g. repeated same bad surface,
    /// lock veto, high latency with no advance, fallback activated, etc.)
    pub no_progress_marker: bool,
    /// Latency spike (optional future signal)
    pub latency_spike: bool,
    /// Fallback rate indicator (optional future signal)
    pub fallback_activated: bool,
}

/// Rolling window of recent telemetry observations.
/// This is the foundation for scoring repetition strength from *signals*
/// instead of brittle string/surface matching.
#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    /// Core new mechanism: scores how strongly the recent telemetry
    /// indicates *repetitive failure*, using signal correlation rather than
    /// text surface matching.
    ///
    /// High sustained ghost pulls + internal monitors + repeated requests
    /// without progress markers = strong repetition signal.
    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() {
            return 0.0;
        }

        let mut ghost_sum = 0.0;
        let mut monitor_count = 0u32;
        let mut spike_count = 0u32;
        let mut no_progress_count = 0u32;
        let mut high_ghost_steps = 0u32;

        for obs in &self.window {
            ghost_sum += obs.ghost_pull;
            if obs.ghost_pull > 4.0 {
                high_ghost_steps += 1;
            }
            if obs.internal_monitor_flawed {
                monitor_count += 1;
            }
            spike_count += obs.request_spike_count;
            if obs.no_progress_marker {
                no_progress_count += 1;
            }
        }

        let n = self.window.len() as f32;

        // Normalized signals
        let avg_ghost = (ghost_sum / n).min(12.0) / 12.0;
        let high_ghost_ratio = high_ghost_steps as f32 / n;
        let monitor_ratio = monitor_count as f32 / n;
        let spike_intensity = (spike_count as f32 / n).min(4.0) / 4.0;
        let no_progress_ratio = no_progress_count as f32 / n;

        // Correlation-style score: these signals reinforcing each other
        // is much stronger evidence of repetition than any single one.
        let mut score = 0.0;
        score += avg_ghost * 0.25;
        score += high_ghost_ratio * 0.30;
        score += monitor_ratio * 0.20;
        score += spike_intensity * 0.15;
        score += no_progress_ratio * 0.10;

        // Bonus when multiple bad signals co-occur in the same window
        let co_occurrence = (high_ghost_ratio + monitor_ratio + no_progress_ratio) / 3.0;
        score += co_occurrence * 0.15;

        score.min(1.0)
    }

    /// Convenience: returns true if the current window looks like a
    /// repetitive failure worth escalating a reflex.
    pub fn is_repetitive_failure(&self, threshold: f32) -> bool {
        self.repetition_strength() > threshold
    }
}

/// In-memory store of loaded reflexes.
#[derive(Debug, Default, Clone)]
pub struct ReflexStore {
    events: Vec<MistakeReflexEvent>,
}

impl ReflexStore {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Load a JSONL ledger (the exact format used in the May 2026 winning runs).
    pub fn load_from_jsonl<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!("failed to open mistake reflex ledger: {}", path.display()))?;
        let reader = BufReader::new(file);

        let mut events = Vec::new();
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let event: MistakeReflexEvent = serde_json::from_str(&line)
                .with_context(|| format!("failed to parse ledger line {}", line_num + 1))?;

            // Only keep events that can actually fire in text-hint mode.
            // The gamma/claims runs relied heavily on this domain.
            if event.domain == "gmms:semantic_correction_slice"
                || !event.trigger_terms.is_empty()
            {
                events.push(event);
            }
        }

        Ok(Self { events })
    }

    /// Simple but effective matcher used in the influence path.
    /// Returns the strongest matching hints for a given prompt.
    pub fn find_relevant_hints(&self, prompt: &str, max_hints: usize) -> Vec<ReflexHint> {
        let normalized_prompt = normalize_for_matching(prompt);
        let mut scored: Vec<(f32, ReflexHint)> = Vec::new();

        for event in &self.events {
            let score = self.score_event_against_prompt(event, &normalized_prompt);
            if score > 0.0 {
                let corrected = if let Some(ep) = &event.episodic_correction {
                    if !ep.trim().is_empty() {
                        ep.clone()
                    } else {
                        event.corrected_reflex.clone()
                    }
                } else {
                    event.corrected_reflex.clone()
                };

                if corrected.trim().is_empty() {
                    continue;
                }

                scored.push((
                    score,
                    ReflexHint {
                        event_id: event.id.clone(),
                        trigger_terms: event.trigger_terms.clone(),
                        corrected_reflex: corrected,
                        score,
                    },
                ));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max_hints);
        scored.into_iter().map(|(_, h)| h).collect()
    }

    fn score_event_against_prompt(&self, event: &MistakeReflexEvent, normalized_prompt: &str) -> f32 {
        if event.trigger_terms.is_empty() {
            return 0.0;
        }

        let mut hits = 0;
        let mut strong_hits = 0;

        for term in &event.trigger_terms {
            let t = normalize_for_matching(term);
            if t.is_empty() {
                continue;
            }
            if normalized_prompt.contains(&t) {
                hits += 1;
                // Bonus for longer, more specific terms
                if t.len() >= 6 {
                    strong_hits += 1;
                }
            }
        }

        if hits == 0 {
            return 0.0;
        }

        // Base score from hit ratio + confidence
        let hit_ratio = hits as f32 / event.trigger_terms.len() as f32;
        let base = (hit_ratio * 0.6) + (event.confidence * 0.4);

        // Stronger boost when we have multiple strong term hits (what worked in practice)
        let boost = (strong_hits as f32 * 0.15).min(0.45);

        (base + boost).min(1.0)
    }

    /// Text-hint style application: returns an augmented system block you can
    /// inject into the prompt. This is exactly the mode that produced the
    /// gamma correction and the 9/10 claims result.
    pub fn apply_text_hints(&self, original_prompt: &str, hints: &[ReflexHint]) -> String {
        if hints.is_empty() {
            return original_prompt.to_string();
        }

        let mut guidance = String::from(
            "\n\n[IMPORTANT CORRECTION HISTORY - USE THIS TO AVOID REPEATING PAST MISTAKES]\n",
        );

        for (i, hint) in hints.iter().enumerate() {
            guidance.push_str(&format!(
                "\nCorrection #{} (from ledger {}):\n{}\n",
                i + 1,
                hint.event_id,
                hint.corrected_reflex.trim()
            ));
        }

        guidance.push_str(
            "\nWhen the current query relates to any of the above topics, \
             explicitly apply the corrected reflex and cite the relevant history if appropriate.\n",
        );

        format!("{}{}", original_prompt, guidance)
    }

    /// Enhanced matching that can take a RepetitionContext from logs.
    ///
    /// This is the start of moving away from pure "next surface matching"
    /// (the historical bane) as the only way to decide "this was a
    /// repetitive failure worth escalating".
    ///
    /// When the context indicates strong repetitive failure signals
    /// (high ghost pull + flawed monitors + repeated requests without
    /// progress), we boost relevant hints and can treat weak text matches
    /// as stronger hits for escalation purposes.
    pub fn find_relevant_hints_with_context(
        &self,
        prompt: &str,
        context: &RepetitionContext,
        max_hints: usize,
    ) -> Vec<ReflexHint> {
        let mut hints = self.find_relevant_hints(prompt, max_hints);

        if context.indicates_repetitive_failure() {
            // Boost all current hints — the logs are screaming that we were
            // in repetitive failure mode, so even partial term matches on
            // known bad patterns become more important.
            for h in &mut hints {
                h.score = (h.score + 0.35).min(1.0);
            }
        }

        hints
    }

    /// New primary path for deciding escalation using telemetry correlation.
    ///
    /// Instead of (or in addition to) fragile surface matching, pass a
    /// rolling window of real signals. If repetition_strength is high,
    /// we return a strong escalation signal that can be treated as
    /// `old_mistake_seen` for the purpose of bumping repeat_mistake_count
    /// and action_level.
    pub fn evaluate_repetition_from_window(
        &self,
        window: &RollingTelemetryWindow,
        threshold: f32,
    ) -> Option<RepetitionEscalation> {
        let strength = window.repetition_strength();
        if strength > threshold {
            Some(RepetitionEscalation {
                strength,
                treat_as_old_mistake: true,
                suggested_action_level_boost: if strength > 0.75 { 2 } else { 1 },
            })
        } else {
            None
        }
    }
}

/// Result of running the log-signal-based repetition scorer.
#[derive(Debug, Clone)]
pub struct RepetitionEscalation {
    /// 0.0–1.0 strength of the repetitive failure signal from telemetry.
    pub strength: f32,
    /// Whether this should be treated as equivalent to `old_mistake_seen`
    /// for escalation purposes (increment repeat_mistake_count, raise levels).
    pub treat_as_old_mistake: bool,
    /// How much to boost action_level / resolution when escalating.
    pub suggested_action_level_boost: u8,
}

    // ─────────────────────────────────────────────────────────────────────
    // Durable storage (dual-write JSONL + optional Qdrant)
    // ─────────────────────────────────────────────────────────────────────

    /// Append a new correction event to a JSONL file (always happens).
    /// This is the audit trail / source of truth.
    pub fn append_to_jsonl(&self, event: &MistakeReflexEvent, path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("failed to open JSONL for append: {}", path.display()))?;

        let line = serde_json::to_string(event)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Write a correction with full dual-write:
    /// - Always appends to the given JSONL path
    /// - If the `qdrant` feature is enabled and a backend is provided, also upserts to Qdrant
    ///
    /// This is the core "corrections survive process death" primitive.
    #[cfg(feature = "qdrant")]
    pub async fn write_correction_dual(
        &self,
        event: MistakeReflexEvent,
        jsonl_path: &Path,
        qdrant: Option<&QdrantReflexBackend>,
    ) -> Result<()> {
        // 1. Always write to JSONL (source of truth)
        self.append_to_jsonl(&event, jsonl_path)?;

        // 2. Optional Qdrant write
        if let Some(backend) = qdrant {
            backend.upsert_event(&event).await?;
        }

        Ok(())
    }

    /// Load every event currently living in Qdrant (fresh process startup path).
    #[cfg(feature = "qdrant")]
    pub async fn load_from_qdrant(backend: &QdrantReflexBackend) -> Result<Self> {
        let events = backend.load_all_events().await?;
        Ok(Self { events })
    }

    /// Helper to create a new correction event from the two things that matter most:
    /// what the bad reflex was, and what the corrected behavior should be.
    /// This is the "capture" side of learning.
    pub fn create_correction(
        id_prefix: &str,
        trigger_terms: Vec<String>,
        bad_reflex: String,
        corrected_reflex: String,
        episodic_note: Option<String>,
    ) -> MistakeReflexEvent {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        MistakeReflexEvent {
            id: format!("{}_{}", id_prefix, now),
            domain: "gmms:semantic_correction_slice".to_string(),
            trigger_terms,
            bad_reflex,
            corrected_reflex,
            episodic_correction: episodic_note,
            evidence_requirement: "Captured during MVP run".to_string(),
            rejected_surfaces: vec![],
            accepted_surfaces: vec![],
            allowed_actions: vec!["apply_correction".to_string()],
            confidence: 0.9,
        }
    }
}

fn normalize_for_matching(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
/// One observation from the generation/telemetry stream.
#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,
    pub ghost_pull: f32,
    pub internal_monitor_flawed: bool,
    pub request_spike_count: u32,
    pub no_progress_marker: bool,
    pub latency_spike: bool,
    pub fallback_activated: bool,
}

/// Rolling window of recent telemetry. Scores repetition strength via signal
/// correlation (the better way, instead of string matching on surfaces).
#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity { self.window.pop_front(); }
        self.window.push_back(obs);
    }

    pub fn len(&self) -> usize { self.window.len() }

    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() { return 0.0; }
        let n = self.window.len() as f32;
        let mut ghost_sum = 0.0;
        let mut high_ghost = 0u32;
        let mut monitors = 0u32;
        let mut spikes = 0u32;
        let mut no_prog = 0u32;

        for o in &self.window {
            ghost_sum += o.ghost_pull;
            if o.ghost_pull > 4.0 { high_ghost += 1; }
            if o.internal_monitor_flawed { monitors += 1; }
            spikes += o.request_spike_count;
            if o.no_progress_marker { no_prog += 1; }
        }

        let avg_g = (ghost_sum / n).min(12.0) / 12.0;
        let hgr = high_ghost as f32 / n;
        let mr = monitors as f32 / n;
        let si = (spikes as f32 / n).min(4.0) / 4.0;
        let npr = no_prog as f32 / n;

        let mut s = avg_g*0.25 + hgr*0.30 + mr*0.20 + si*0.15 + npr*0.10;
        s += ((hgr + mr + npr) / 3.0) * 0.15;
        s.min(1.0)
    }

    pub fn is_repetitive_failure(&self, threshold: f32) -> bool {
        self.repetition_strength() > threshold
    }
}

#[derive(Debug, Clone)]
pub struct RepetitionEscalation {
    pub strength: f32,
    pub treat_as_old_mistake: bool,
    pub suggested_action_level_boost: u8,
}

impl ReflexStore {
    pub fn evaluate_repetition_from_window(
        &self,
        window: &RollingTelemetryWindow,
        threshold: f32,
    ) -> Option<RepetitionEscalation> {
        let s = window.repetition_strength();
        if s > threshold {
            Some(RepetitionEscalation {
                strength: s,
                treat_as_old_mistake: true,
                suggested_action_level_boost: if s > 0.75 { 2 } else { 1 },
            })
        } else { None }
    }
}

// Lightweight RepetitionContext + rolling window for signal-based repetition detection.
// This replaces brittle surface matching with correlation of telemetry signals.

#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,
    pub ghost_pull: f32,
    pub internal_monitor_flawed: bool,
    pub request_spike_count: u32,
    pub no_progress_marker: bool,
    pub latency_spike: bool,
    pub fallback_activated: bool,
}

#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() { return 0.0; }
        let n = self.window.len() as f32;
        let mut ghost_sum = 0.0f32;
        let mut high_ghost = 0u32;
        let mut monitors = 0u32;
        let mut spikes = 0u32;
        let mut no_prog = 0u32;

        for o in &self.window {
            ghost_sum += o.ghost_pull;
            if o.ghost_pull > 4.0 { high_ghost += 1; }
            if o.internal_monitor_flawed { monitors += 1; }
            spikes += o.request_spike_count;
            if o.no_progress_marker { no_prog += 1; }
        }

        let avg_g = (ghost_sum / n).min(12.0) / 12.0;
        let hgr = high_ghost as f32 / n;
        let mr = monitors as f32 / n;
        let si = (spikes as f32 / n).min(4.0) / 4.0;
        let npr = no_prog as f32 / n;

        let mut s = avg_g*0.25 + hgr*0.30 + mr*0.20 + si*0.15 + npr*0.10;
        s += ((hgr + mr + npr)/3.0) * 0.15;
        s.min(1.0)
    }

    pub fn is_repetitive_failure(&self, threshold: f32) -> bool {
        self.repetition_strength() > threshold
    }
}

#[derive(Debug, Clone)]
pub struct RepetitionEscalation {
    pub strength: f32,
    pub treat_as_old_mistake: bool,
    pub suggested_action_level_boost: u8,
}

impl ReflexStore {
    /// Lightweight entry point: feed a rolling window of telemetry.
    /// If repetition strength > threshold, return escalation info that
    /// the caller can use to treat this as old_mistake_seen and bump
    /// repeat_mistake_count / action_level on the relevant events.
    pub fn evaluate_repetition_from_window(
        &self,
        window: &RollingTelemetryWindow,
        threshold: f32,
    ) -> Option<RepetitionEscalation> {
        let s = window.repetition_strength();
        if s > threshold {
            Some(RepetitionEscalation {
                strength: s,
                treat_as_old_mistake: true,
                suggested_action_level_boost: if s > 0.75 { 2 } else { 1 },
            })
        } else {
            None
        }
    }
}

// Lightweight RepetitionContext + rolling telemetry window.
// Scores repetition strength from signal correlation (ghost pulls, monitors,
// no-progress markers, etc.) instead of brittle surface/string matching.
// If strength > threshold → treat as old_mistake_seen for escalation.

#[derive(Debug, Clone, Default)]
pub struct TelemetryObservation {
    pub step: usize,
    pub ghost_pull: f32,
    pub internal_monitor_flawed: bool,
    pub request_spike_count: u32,
    pub no_progress_marker: bool,
    pub latency_spike: bool,
    pub fallback_activated: bool,
}

#[derive(Debug, Clone)]
pub struct RollingTelemetryWindow {
    window: VecDeque<TelemetryObservation>,
    capacity: usize,
}

impl RollingTelemetryWindow {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, obs: TelemetryObservation) {
        if self.window.len() == self.capacity {
            self.window.pop_front();
        }
        self.window.push_back(obs);
    }

    pub fn repetition_strength(&self) -> f32 {
        if self.window.is_empty() { return 0.0; }
        let n = self.window.len() as f32;
        let mut ghost_sum = 0.0f32;
        let mut high_ghost = 0u32;
        let mut monitors = 0u32;
        let mut spikes = 0u32;
        let mut no_prog = 0u32;

        for o in &self.window {
            ghost_sum += o.ghost_pull;
            if o.ghost_pull > 4.0 { high_ghost += 1; }
            if o.internal_monitor_flawed { monitors += 1; }
            spikes += o.request_spike_count;
            if o.no_progress_marker { no_prog += 1; }
        }

        let avg_g = (ghost_sum / n).min(12.0) / 12.0;
        let hgr = high_ghost as f32 / n;
        let mr = monitors as f32 / n;
        let si = (spikes as f32 / n).min(4.0) / 4.0;
        let npr = no_prog as f32 / n;

        let mut s = avg_g*0.25 + hgr*0.30 + mr*0.20 + si*0.15 + npr*0.10;
        s += ((hgr + mr + npr) / 3.0) * 0.15; // co-occurrence
        s.min(1.0)
    }

    pub fn is_repetitive_failure(&self, threshold: f32) -> bool {
        self.repetition_strength() > threshold
    }
}

#[derive(Debug, Clone)]
pub struct RepetitionEscalation {
    pub strength: f32,
    pub treat_as_old_mistake: bool,
    pub suggested_action_level_boost: u8,
}

impl ReflexStore {
    /// If the rolling window shows strong repetitive failure via signal
    /// correlation, return escalation info that should cause the caller
    /// to treat this as old_mistake_seen (bump repeat_mistake_count + action_level).
    pub fn evaluate_repetition_from_window(
        &self,
        window: &RollingTelemetryWindow,
        threshold: f32,
    ) -> Option<RepetitionEscalation> {
        let s = window.repetition_strength();
        if s > threshold {
            Some(RepetitionEscalation {
                strength: s,
                treat_as_old_mistake: true,
                suggested_action_level_boost: if s > 0.75 { 2 } else { 1 },
            })
        } else {
            None
        }
    }
}

fn normalize_for_matching(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { " " })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
