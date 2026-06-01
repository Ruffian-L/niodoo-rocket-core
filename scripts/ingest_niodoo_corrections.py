#!/usr/bin/env python3
"""
Ingest Niodoo MVP mistake reflex corrections into the `niodoo-corrections` Qdrant collection.

This follows the exact schema, payload shape, point ID convention, and embedding path
that the rest of the user's memory stack (grok-memories + remember-memories) uses.

Usage (after running embed-up):
    python3 scripts/ingest_niodoo_corrections.py

It can consume:
- The main ledger: ledgers/mvp_corrections.jsonl
- Individual files dropped in corrections/ (future)

Environment variables (override if needed):
    NIODOO_QDRANT_URL     default http://127.0.0.1:6360
    NIODOO_EMBED_URL      default http://127.0.0.1:8302/v1/embeddings
    NIODOO_COLLECTION     default niodoo-corrections
"""

from __future__ import annotations

import json
import os
import sys
import uuid
from pathlib import Path
from typing import Any

import requests
from qdrant_client import QdrantClient
from qdrant_client.http import models as qm

# ── configuration (matches user's real stack) ────────────────────────────────
QDRANT_URL = os.environ.get("NIODOO_QDRANT_URL", "http://127.0.0.1:6360")
EMBED_URL  = os.environ.get("NIODOO_EMBED_URL",  "http://127.0.0.1:8302/v1/embeddings")
EMBED_MODEL = "Qwen3-Embedding-8B"
COLLECTION  = os.environ.get("NIODOO_COLLECTION", "niodoo-corrections")

# Dedicated namespace so IDs never collide with grok-memories
NAMESPACE_NIODOO_CORRECTIONS = uuid.UUID("f0023e76-bec5-59f2-b0a0-bef200c9dea8")

# Where to look for corrections
LEDGER_PATH = Path(__file__).parent.parent / "ledgers" / "mvp_corrections.jsonl"
CORRECTIONS_DIR = Path(__file__).parent.parent / "corrections"

CHUNK_CHARS = 3500  # keep chunks reasonable for the embedder


def embed_texts(texts: list[str]) -> list[list[float]]:
    """Call the local OpenAI-compatible embed proxy."""
    r = requests.post(
        EMBED_URL,
        json={"model": EMBED_MODEL, "input": texts},
        timeout=180,
    )
    r.raise_for_status()
    return [d["embedding"] for d in r.json()["data"]]


def make_point_id(rel_path: str, chunk_idx: int) -> str:
    """Idempotent point ID using the dedicated namespace."""
    return str(uuid.uuid5(NAMESPACE_NIODOO_CORRECTIONS, f"{rel_path}::{chunk_idx}"))


def correction_to_payload(text: str, rel_path: str, chunk_idx: int, n_chunks: int) -> dict[str, Any]:
    """Build payload that mirrors grok-memories so existing MCP tools just work."""
    name = Path(rel_path).name
    stem = Path(rel_path).stem
    # simple human stem cleaning (dashes/underscores → spaces, strip hash suffixes)
    stem_human = stem.replace("-", " ").replace("_", " ").split("#")[0].strip()

    return {
        "path": str(Path("/home/ruff") / rel_path),
        "rel_path": rel_path,
        "name": name,
        "stem_human": stem_human,
        "ext": Path(rel_path).suffix.lower() or ".jsonl",
        "kind": "text",
        "chunk_idx": chunk_idx,
        "n_chunks": n_chunks,
        "text": text,
    }


def chunk_text(text: str) -> list[str]:
    if len(text) <= CHUNK_CHARS:
        return [text]
    chunks = []
    i = 0
    while i < len(text):
        chunks.append(text[i : i + CHUNK_CHARS])
        i += CHUNK_CHARS
    return chunks


def load_corrections_from_ledger(ledger: Path) -> list[tuple[str, str]]:
    """Returns list of (rel_path, text) from the main ledger JSONL."""
    if not ledger.exists():
        return []
    items = []
    with ledger.open() as f:
        for i, line in enumerate(f):
            if not line.strip():
                continue
            try:
                event = json.loads(line)
                # Use the corrected_reflex or episodic_correction as the main text
                text = event.get("corrected_reflex") or event.get("episodic_correction") or ""
                if not text.strip():
                    continue
                rel = f"corrections/mvp_reflex_{event.get('id', f'line{i}')}.md"
                items.append((rel, text))
            except Exception as e:
                print(f"[warn] bad line in ledger: {e}", file=sys.stderr)
    return items


def main() -> int:
    print(f"Qdrant: {QDRANT_URL}")
    print(f"Embed:  {EMBED_URL}")
    print(f"Collection: {COLLECTION}")

    client = QdrantClient(url=QDRANT_URL, prefer_grpc=False, timeout=15)

    # Ensure collection exists with correct schema
    try:
        info = client.get_collection(COLLECTION)
        print(f"Collection exists: {info.config.params.vectors.size}d")
    except Exception:
        print("Creating collection niodoo-corrections (4096-d Cosine)...")
        client.create_collection(
            collection_name=COLLECTION,
            vectors_config=qm.VectorParams(size=4096, distance=qm.Distance.COSINE),
        )

    corrections = load_corrections_from_ledger(LEDGER_PATH)

    # Future: also scan corrections/ directory for individual files
    if not corrections:
        print("No corrections found in ledgers/mvp_corrections.jsonl")
        print("You can also drop files into corrections/ (future support).")
        return 0

    print(f"Found {len(corrections)} correction(s) to ingest...")

    for rel_path, text in corrections:
        chunks = chunk_text(text)
        n_chunks = len(chunks)

        embeddings = embed_texts(chunks)

        points = []
        for idx, (chunk, vec) in enumerate(zip(chunks, embeddings)):
            pid = make_point_id(rel_path, idx)
            payload = correction_to_payload(chunk, rel_path, idx, n_chunks)
            points.append(qm.PointStruct(id=pid, vector=vec, payload=payload))

        client.upsert(collection_name=COLLECTION, points=points, wait=True)
        print(f"  ✓ {rel_path} ({n_chunks} chunk(s))")

    print("\nDone. Corrections are now in niodoo-corrections and searchable via existing MCP tools.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())