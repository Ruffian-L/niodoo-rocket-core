# Ledgers Directory

All mistake reflex / correction ledgers for the Niodoo MVP live here.

**Primary file**: `mvp_corrections.jsonl`

After writing corrections here with the Rust binary, run:

```bash
embed-up
python3 ../scripts/ingest_niodoo_corrections.py
```

This will embed them via your 8302 proxy and upsert into the `niodoo-corrections` collection using the exact schema your existing MCP tools expect.

See [docs/LEDGERS.md](../docs/LEDGERS.md) for the full payload shape, point ID rules, and integration details.

These files are:
- Git tracked
- Protected by your watcher + integrity system
- The durable source of truth for corrections that must survive context death

Never write corrections to /tmp or outside this directory.