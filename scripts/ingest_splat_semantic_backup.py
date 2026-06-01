#!/usr/bin/env python3
"""
Ingest the big SplatRAG semantic backup (splat_backup_semantic_FULL.md)
into grok-memories.

This file is the consolidated semantic memory / research log from
Echo / Lumina / Shep's original runtime in team_build.

It contains the history of compute_ghost_vector usage, Ghost Vector Norm
experiments, Splat Memory / Splat RAG development, Physics of Friendship
work, dream reflection cycles, etc.

Run this when embed-up is active.
"""

import re
import uuid
from datetime import datetime
from pathlib import Path

import requests
from qdrant_client import QdrantClient
from qdrant_client.http import models as qm
from tqdm import tqdm

# ── CONFIG ────────────────────────────────────────────────────────────────
SOURCE_FILE = Path("/home/ruff/splat_backup_semantic_FULL.md")
EMBED_URL = "http://127.0.0.1:8302/v1/embeddings"
EMBED_MODEL = "Qwen3-Embedding-8B"
QDRANT_URL = "http://127.0.0.1:6360"
TARGET_COLLECTION = "grok-memories"

# Chunking
MAX_CHUNK_CHARS = 3800
OVERLAP_CHARS = 400

# Namespace for stable IDs (so re-running is idempotent)
NAMESPACE = uuid.uuid5(uuid.NAMESPACE_DNS, "splat_backup_semantic_FULL.md")

# ── HELPERS ───────────────────────────────────────────────────────────────

def embed_text(text: str) -> list[float]:
    r = requests.post(
        EMBED_URL,
        json={"model": EMBED_MODEL, "input": [text]},
        timeout=120,
    )
    r.raise_for_status()
    return r.json()["data"][0]["embedding"]


def make_point_id(chunk_index: int, content_hash: str) -> str:
    """Stable ID based on file + position + content."""
    key = f"{SOURCE_FILE.name}::{chunk_index}::{content_hash[:16]}"
    return str(uuid.uuid5(NAMESPACE, key))


def extract_date(text: str) -> str | None:
    """Try to pull the first YYYY-MM-DD date from the chunk."""
    m = re.search(r"\b(20\d{2}-\d{2}-\d{2})\b", text)
    return m.group(1) if m else None


def has_ghost_vector_content(text: str) -> bool:
    return bool(re.search(r"compute_ghost_vector|Ghost Vector Norm|ghost_vector", text, re.I))


def chunk_text(text: str) -> list[str]:
    """Split on natural dated boundaries first, then size with overlap."""
    # Split on the common dated header pattern used in this backup
    dated_blocks = re.split(r"(?=\n\*\*\d{4}-\d{2}-\d{2})", text)

    chunks = []
    current = ""

    for block in dated_blocks:
        if not block.strip():
            continue

        if len(current) + len(block) > MAX_CHUNK_CHARS and current:
            chunks.append(current.strip())
            # overlap
            current = current[-OVERLAP_CHARS:] + "\n" + block
        else:
            current += "\n" + block if current else block

    if current.strip():
        chunks.append(current.strip())

    # Final safety split for any monster blocks
    final_chunks = []
    for ch in chunks:
        if len(ch) <= MAX_CHUNK_CHARS:
            final_chunks.append(ch)
        else:
            # hard split with overlap
            i = 0
            while i < len(ch):
                final_chunks.append(ch[i : i + MAX_CHUNK_CHARS])
                i += MAX_CHUNK_CHARS - OVERLAP_CHARS
    return final_chunks


def main():
    print(f"Source: {SOURCE_FILE}")
    print(f"Target collection: {TARGET_COLLECTION}")

    if not SOURCE_FILE.exists():
        print("File not found!")
        return

    text = SOURCE_FILE.read_text(encoding="utf-8", errors="ignore")
    print(f"File size: {len(text):,} characters")

    chunks = chunk_text(text)
    print(f"Created {len(chunks)} chunks")

    client = QdrantClient(url=QDRANT_URL, timeout=30)

    points = []
    for idx, chunk in enumerate(tqdm(chunks, desc="Embedding + preparing")):
        try:
            vec = embed_text(chunk)
        except Exception as e:
            print(f"Embed failed on chunk {idx}: {e}")
            continue

        content_hash = str(hash(chunk))
        pid = make_point_id(idx, content_hash)

        payload = {
            "text": chunk,
            "source": str(SOURCE_FILE),
            "source_basename": SOURCE_FILE.name,
            "chunk_index": idx,
            "approx_date": extract_date(chunk),
            "has_ghost_vector": has_ghost_vector_content(chunk),
            "length": len(chunk),
            "ingested_at": datetime.utcnow().isoformat(),
            "memory_space": "echo-lumina-shep-splat-semantic",
        }

        points.append(qm.PointStruct(id=pid, vector=vec, payload=payload))

        # Batch upsert every 200 chunks
        if len(points) >= 200:
            client.upsert(collection_name=TARGET_COLLECTION, points=points, wait=True)
            print(f"  Upserted batch of {len(points)}")
            points = []

    if points:
        client.upsert(collection_name=TARGET_COLLECTION, points=points, wait=True)
        print(f"  Final upsert of {len(points)} points")

    print("Done. Splat semantic backup ingested into grok-memories.")
    print(f"Search with metadata filter: memory_space = 'echo-lumina-shep-splat-semantic'")


if __name__ == "__main__":
    main()
