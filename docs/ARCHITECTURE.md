# Architecture — Minimal Layers for the MVP

**Core Idea (one sentence):**

A small long-running model + durable memory of its own corrections (proven to change behavior) + occasional stronger guidance = cumulative self-improvement that survives full context death.

This is the only architecture that matters until the north star test in MVP_DEFINITION.md produces real numbers.

---

## The Proven Minimal Stack (Grounded in MASTER_RESEARCH_LEDGER)

### 1. Mistake Reflex / Correction Layer (Tier 1 Evidence — Ship This First)

- **What it is:** JSONL events (304 in the canonical claims corpus ledger, plus policy lifecycle ledgers from gamma runs) that encode prior mistakes, supersessions, and corrections mined from the user's own CLAIMS.md and runtime failures.
- **How it works:** Loaded at startup via `--mistake-reflex-path` (or automatic in later phases). In `influence` / `text-hint` mode the events change generation behavior on matching prompts without requiring captured hidden-state probes.
- **Evidence:** 
  - Gamma artifact triage rerun (seed 143, May 29): Baseline repeated false GREEN; ledger-loaded candidate correctly refused on the first relevant turn and cited missing raw telemetry.
  - Claims AHA 3-arm (May 8): 8B + 304-event ledger = 9/10 substantive answers on internal project questions vs 1/10 cold and 0/10 for cold Opus.
- **Implementation source to port:** `team_build/niodoo/src/runtime/mistake_reflex.rs` (the permissive trigger-hits + strict anchor paths, domain="gmms:semantic_correction_slice").
- **MVP status:** This is the primary path. Everything else is optional until this layer + Qdrant closes the loop.

### 2. Durable Storage Layer (The Missing Piece — Qdrant Dual-Write)

- **Pattern:** Dual write on every correction/reflex event.
  - JSONL (always, under `ledgers/`): Human-auditable, git-tracked, repo-protected source of truth. **Never** written to /tmp.
  - Qdrant (feature-gated, best-effort against your real memory stack on 6360): Fast index + per-turn refresh.
- **Why dual:** JSONL survives everything, is versioned, and covered by your watcher + integrity system. Qdrant makes the memory live and queryable when the custom stack cooperates.
- **Existing code:** `niodoo/src/bridge/qdrant_adapter.rs` + ingest scripts + the `grok-memories` collection (25k+ points from prior semantic history runs).
- **MVP job:** Wire the correction/reflex-specific collection (separate from pure semantic history) with the same dual-write discipline. This is what makes "kill the process, fresh launch reads the corrections" actually work.

### 3. Steering Layer (Mature, Keep Minimal)

- Physics-based hidden state intervention (gravity wells, ghost basins, repulsion, dynamic ramp, layer banding, TOPOCOT-aware pressure).
- Already mature in the main niodoo engine.
- For MVP: Treat as a black box that the reflex layer can influence. Do not re-implement or heavily tune unless a specific reflex needs it to demonstrate the improvement.

### 4. Compact Transport Layer (One Locked Version Only — Do Not Re-Fight)

- Purpose: Turn high-dimensional memory (hidden states, anchors, momentum vectors) into something small enough to write to disk/Qdrant and re-inject efficiently.
- Historical reality: Multiple incompatible secret sauce versions (V1/V2/V3) with length mismatches were the single longest repeating trap in the entire record.
- **MVP rule (from KNOWN_PITFALLS):** Pick one minimal stable format (or even stay with pure text-hint JSONL events for the first working north star test). Lock it. The gamma and claims wins did not require perfect 64D/128D Unicode braille transport.
- Future (Phase 4): Stabilized secret sauce or niodv4 pressure-aware nested compression as a portability upgrade, never as a blocker.

### 5. Active / Fluid Memory Layer (Aspirational — Phase 4+ Only)

- Vision (user's words from `shep-loop/hydrodynamic-swarm/imagine a bunch of tiny.txt`): Tiny compressed embeds that *are* the memory. Each carries its own physics (viscosity, decay via mashoka slicing, reinforcement on invocation, mass). Model detects stuck states via TOPOCOT/varentropy and emits tags that mutate the physics in real time. Memories overlap and communicate with neighbors.
- Partial implementation exists: SplatMemory, SwarmMatrix, EmbedManager, PrimeGovernor, decay kernels, some edge communication.
- Missing critical piece: The Compass state machine that translates raw signals into specific physics changes that make emitting `<FOCUS>` or `<EXPLORE>` the model's survival reflex.
- **MVP rule:** Extract only the smallest clean splat decay + reinforcement primitives on top of the ledger if they are low-risk. Never make the full fluid swarm the primary architecture for v1. That vision is beautiful and is the long-term target — after the basic durable reflex loop is proven.

---

## What We Explicitly Do NOT Build in This MVP

- Full hydrodynamic/fluid active memory as the core system.
- Self-invoking cybernetic loop (model tags mutating live physics).
- Multiple secret sauce codec variants or a general compression framework.
- Heavy GPU topology work (lophat etc.) unless already working and free.
- Massive autonomous agent swarms or general orchestration.
- Any claim of general superiority on standard benchmarks.
- Another research monolith.

---

## Success Criteria (Repeated From MVP_DEFINITION — This Is The Only Measure)

We consider the MVP successful **only** when we can point to concrete artifacts showing:

- A real problem the base model fails at.
- Corrections written to the tracked `ledgers/` JSONL (dual with Qdrant when possible) during the run.
- Full deliberate process kill.
- Fresh process (minimal context) loading the stored corrections.
- Clear, reproducible, attributable improvement on the same problem class after the reset.
- Protection watcher green + full raw telemetry captured.

Everything else — cool physics, beautiful logs, interesting internal monitors, partial codec roundtrips — is secondary until the above exists with numbers.

---

## Protection & Observability (Non-Negotiable Architecture)

- File watcher (`niodoo_file_watch.py` + `~/bin/niodoo-watch` wrappers) running on the mvp tree and all critical artifact directories.
- Periodic git status + integrity baseline snapshots.
- All runs produce dated, git-committed or explicitly snapshotted artifact trees.
- No "ghost" explanations. Objective measurement first.

This is not hygiene. It is the direct response to the user's documented experience of file changes and the neurological load that made subjective trust expensive.

---

## Summary

The architecture is deliberately the smallest slice that can carry the proven reflex signals (gamma + claims) across a real kill boundary using the storage layer that was designed for exactly this job (Qdrant dual-write).

Everything else is beautiful future work that becomes tractable the moment Phase 2 of the roadmap produces the first real north star numbers.

Build only what serves that moment. Document only what is true. Protect everything.
