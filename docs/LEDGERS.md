# Mistake Reflex Ledgers — Storage & Tracking

> **Note for future sessions**: The Grok TUI collapses scrollback.  
> All critical integration details (collection schema, payload shape, point ID rules, ingestion flow) are documented here so you never have to ask "what was the JSON shape again?"

---

## Target Collection (Your Real Stack)

**Collection name**: `niodoo-corrections`

```json
{
  "name": "niodoo-corrections",
  "vectors": { "size": 4096, "distance": "Cosine" }
}
```

- **Embedding model**: Qwen3-Embedding-8B (4096-d), served by llama.cpp via the OpenAI-compatible proxy.
- **Qdrant**: `http://127.0.0.1:6360`
- **Embed proxy**: `http://127.0.0.1:8302/v1/embeddings` (bring up with `embed-up`)

This collection is deliberately separate from `grok-memories` so reflex corrections can be searched independently while still being discoverable by your existing MCP tools (because the payload schema mirrors `grok-memories`).

---

## Payload Schema (per point)

Every correction point must follow this shape so the existing search / RAG tools continue to work unchanged:

| field        | type | notes |
|--------------|------|-------|
| path         | str  | absolute path on disk |
| rel_path     | str  | path relative to source root |
| name         | str  | basename |
| stem_human   | str  | dashes/underscores → spaces, hash-suffix stripped |
| ext          | str  | lowercase, includes the `.` |
| kind         | str  | text \| pdf \| docx \| image \| binary |
| chunk_idx    | int  | 0-based |
| n_chunks     | int  | total chunks for this file |
| text         | str  | the chunk content (≤3500 chars recommended) |

**Point ID convention (idempotent)**:
```python
pid = str(uuid.uuid5(NAMESPACE_NIODOO_CORRECTIONS, f"{rel_path}::{chunk_idx}"))
```

Dedicated namespace (so IDs never collide with `grok-memories`):
```python
NAMESPACE_NIODOO_CORRECTIONS = uuid.UUID("f0023e76-bec5-59f2-b0a0-bef200c9dea8")
```

---

## How to Ingest

1. Make sure your embed proxy is up:
   ```bash
   embed-up
   ```

2. Run the dedicated ingester:
   ```bash
   cd /home/ruff/projects/Homernd/mvp
   python3 scripts/ingest_niodoo_corrections.py
   ```

The script reads `ledgers/mvp_corrections.jsonl` (the durable source of truth produced by the Rust MVP), chunks the `corrected_reflex` / `episodic_correction` text, embeds via your 8302 proxy, and upserts into `niodoo-corrections` with the exact payload schema above.

---

## Minimal Example (one correction)

See the full script at `scripts/ingest_niodoo_corrections.py` for the production version.

The core pattern the user provided:

```python
import uuid, requests
from qdrant_client import QdrantClient
from qdrant_client.http import models as qm

QDRANT = "http://127.0.0.1:6360"
EMBED  = "http://127.0.0.1:8302/v1/embeddings"
COLL   = "niodoo-corrections"
NS     = uuid.UUID("f0023e76-bec5-59f2-b0a0-bef200c9dea8")

text  = "wrong output … → corrected output …"
rel   = "corrections/2026-05-30/example.md"
pid   = str(uuid.uuid5(NS, f"{rel}::0"))

vec = requests.post(EMBED, json={"model": "Qwen3-Embedding-8B", "input": [text]},
                    timeout=180).json()["data"][0]["embedding"]
assert len(vec) == 4096

QdrantClient(QDRANT).upsert(
    collection_name=COLL,
    points=[qm.PointStruct(id=pid, vector=vec, payload={
        "path": f"/home/ruff/{rel}",
        "rel_path": rel,
        "name": "example.md",
        "stem_human": "example",
        "ext": ".md",
        "kind": "text",
        "chunk_idx": 0,
        "n_chunks": 1,
        "text": text,
    })],
    wait=True,
)
```

---

## Rust MVP Side (source of truth)

The Rust binary (`cargo run --features qdrant -- capture-correction ...`) writes to:

- `ledgers/mvp_corrections.jsonl` (always — this is the git-tracked, watcher-protected audit trail)

From there the Python ingester above turns the corrections into properly embedded, searchable points in `niodoo-corrections`.

This two-stage design (Rust durable ledger → Python embed + upsert using your proven stack) is the reliable integration path that matches how the rest of your memory system works.

**Rule**: Corrections NEVER land in `/tmp` or any ephemeral location. All reflex ledgers live inside this repository under `ledgers/` so they are:

- Git tracked and versioned
- Protected by the existing file watcher + integrity baselines
- Human-auditable and reproducible across sessions
- Part of the permanent record (like the 304-event `claims_corpus_ledger_20260508`)

This is the durable memory substrate for the MVP.

---

## Directory Layout

```
mvp/
├── ledgers/
│   ├── .gitkeep
│   ├── mvp_corrections.jsonl          ← primary working ledger
│   └── seeds/                         ← historical or imported ledgers (e.g. gamma policy ledgers, claims corpus subsets)
│       └── README.md
├── docs/
│   └── LEDGERS.md                     ← this file
└── ...
```

All JSONL files in `ledgers/` are the canonical source of truth for mistake reflexes.

---

## Ledger Format

Each line is one `MistakeReflexEvent` (minimal shape used by the proven gamma/claims influence path):

```json
{
  "id": "unique-event-id",
  "domain": "gmms:semantic_correction_slice",
  "trigger_terms": ["term1", "§10dv", "specific", "keywords"],
  "bad_reflex": "description of the repeated mistake",
  "corrected_reflex": "the exact correction / desired behavior",
  "episodic_correction": "optional extra context or story",
  "evidence_requirement": "what evidence was required to accept this correction",
  "rejected_surfaces": ["phrases that should be rejected"],
  "accepted_surfaces": ["phrases that indicate the correction was applied"],
  "allowed_actions": ["apply_correction", "cite_history"],
  "confidence": 0.92
}
```

The `domain` field is important for compatibility with older tooling.

---

## How Corrections Get Written

### 1. Manual / Capture (current)
```bash
# From inside the mvp directory
cargo run --features qdrant -- capture-correction \
  --bad "repeated the same judgment error" \
  --corrected "When a prior correction exists for this pattern, explicitly recall and apply it"

# Or the lower-level write
cargo run --features qdrant -- write-correction
```

Both default to `ledgers/mvp_corrections.jsonl`.

### 2. Programmatic (future)
The `ReflexStore` API supports:
- `append_to_jsonl()`
- `write_correction_dual()` (JSONL + best-effort Qdrant)

---

## Qdrant Relationship (Your Real Stack)

- JSONL is **always** the source of truth and is tracked in this repo.
- Qdrant (your `memory-up` instance on 6360) is the live/queryable index when available.
- Because your memory stack is heavily customized ("Mickey Mouse"), direct qdrant-client writes can hit transport/h2 issues. The MVP treats Qdrant as best-effort.
- Your existing hooks (`remember-after-save-qdrant.sh`, PostToolUse, etc.) can be pointed at `ledgers/*.jsonl` files if desired.

When the transport situation stabilizes, the same events will flow into your `niodoo_mvp_mistake_reflexes` collection automatically.

---

## Protection & Integrity

These files are covered by:
- The `niodoo-watch` system you already have running
- Git history
- Periodic integrity baselines

Never move corrections outside `ledgers/`. If you need to archive an old ledger, move it into `ledgers/archive/` (still tracked).

---

## Loading Ledgers

```bash
# Test influence with the working ledger
cargo run -- reflex --ledger ledgers/mvp_corrections.jsonl --prompt "your test prompt here"

# Inspect contents
cargo run -- inspect-ledger
```

The Reflex command defaults to `ledgers/mvp_corrections.jsonl` when run from inside the mvp directory.

---

## Seeding from Historical Data

You can copy subsets of high-signal historical ledgers here:

- `claims_corpus_ledger_20260508/claims_ledger.jsonl` (the 304-event one that produced 9/10)
- Gamma policy lifecycle ledgers

Place them under `ledgers/seeds/` with a short `README.md` explaining provenance and what they proved.

---

## Philosophy

This directory is the living "scar tissue" of the project.

Every time the system repeats a mistake and we capture the correction, it goes here — permanently, reproducibly, and protected.

This is how we escape the repetition trauma documented in `KNOWN_PITFALLS.md`.

Corrections written here are the substrate that must survive full context death and produce measurable improvement on the other side.

No more `/tmp`. No more lost threads. Proper repo documentation and tracking from now on.