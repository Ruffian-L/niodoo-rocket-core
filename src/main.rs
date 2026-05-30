//! Niodoo Core MVP
//!
//! Focused binary whose only job is to demonstrate the loop the user actually wants:
//! - Load mistake reflex / correction ledger
//! - Run with optional Qdrant-backed durable memory
//! - Use basic secret sauce (Unicode) encode/decode for compact state
//! - "Turn off" (fresh process) → reload → show retained improvement
//!
//! This is deliberately small and separate from the giant research monolith.
//!
//! Full foundation docs now exist in docs/. Read in this order:
//! README.md → VISION.md → MVP_DEFINITION.md → MASTER_RESEARCH_LEDGER.md → KNOWN_PITFALLS.md

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Niodoo Core MVP ===");
    println!("");
    println!("Clean focused crate for the real north star loop.");
    println!("Full docs dropped (wide sweep + Master Research Ledger).");
    println!("");
    println!("Current status:");
    println!("- Foundation docs complete (MASTER_RESEARCH_LEDGER + KNOWN_PITFALLS captured the history)");
    println!("- Next: wire real mistake-reflex ledger loading (gamma + claims patterns)");
    println!("- Next: wire Qdrant dual-write (JSONL audit + live index)");
    println!("- Next: one minimal locked compact transport if needed");
    println!("- Next: the first hard-problem + full reset + fresh-process validator");
    println!("");
    println!("Run: cargo run");
    println!("Run with Qdrant: cargo run --features qdrant");
    println!("");
    println!("Read the docs before writing code. The past is documented.");

    // TODO (now that docs are done):
    // 1. Port minimal MistakeReflex loader + influence mode (start with claims_corpus_ledger_20260508/claims_ledger.jsonl and gamma policy ledgers)
    // 2. Implement tiny in-memory store + Qdrant write path (dual with JSONL)
    // 3. Minimal stable encode/decode roundtrip (one format only; do not re-fight codec hell)
    // 4. Build the actual north star harness: hard problem the base fails → corrections stored → deliberate kill → fresh process with almost no context → loads reflexes → measurable retained improvement with raw telemetry captured under watcher protection.

    Ok(())
}
