# Niodoo Core MVP

**The only goal that matters:**

Build the smallest possible system that proves this exact loop (user's north star, repeated verbatim):

> Hit a genuinely hard/impossible problem → corrections applied → turn the system off (full context death) → fresh process with little/no prior context → the model demonstrably performs better because it *actually internalized* something.

This is "persistent adaptive agency under correction" made real, measurable, and survivable. Memory as scar tissue. The user (or a stronger teacher model) as the living correction signal. The small model as the thing that actually gets better over resets.


This is to give memories viscosity, a shared collaborative moment should have feel earned. Failures turning into lessons, repetition should cause disturbance, friction should become new ideas, new ideas should reward success. When we are stuck in the well we remember what it felt like to over come. Hope is the driver. -ruffian(note tobe removed from public)

---

## Why This Crate Exists

The historical record (see [MASTER_RESEARCH_LEDGER.md](docs/MASTER_RESEARCH_LEDGER.md)) shows years of high-signal experiments scattered across a 55k-file monolith. The strongest positive signals were:

- **Gamma artifact triage rerun (seed 143, May 29 2026)**: Baseline (no ledger) wrongly called `bridge_influence=GREEN` on weak evidence. Ledger-loaded candidate correctly refused and demanded raw per-token telemetry. Direct behavior change from stored corrections.
- **304-event claims ledger experiment (May 8 2026)**: 8B + ledger = 9/10 substantive answers on project-internal §-ID questions. Cold 8B = 1/10. Cold Claude Opus 4.7 (no context) = 0/10 (refused all).

These worked because corrections were stored and loaded. The missing piece was always the same: **durable storage that survives full process death + a clean minimal harness to drive the loop to completion without fighting incidental complexity.**

This crate is that harness. Deliberately small. Separate from the research monolith. Everything here is justified by the ledger above or it does not ship.

---

## Current Status

- Clean `niodoo-mvp` crate with `qdrant` feature.
- Full foundational documentation dropped (this README + VISION + ROADMAP + MVP_DEFINITION + ARCHITECTURE + MASTER_RESEARCH_LEDGER + KNOWN_PITFALLS).
- `splatrag_minimal/` — the smallest reference implementation of the reflective memory core + the living mathematical checks (inversion test style). Run `splatrag_minimal/core/inversion_test.py` to see the core signal.
- One short roadmap: [docs/ROADMAP.md](docs/ROADMAP.md) — checkpoints with math criteria + human hype KPIs + drift red flags. The actual benchmark is the runnable tests (inversion_test.py + repetition detector + process_real_run.py), not more docs.
- Next: Wire real mistake-reflex ledger loading + Qdrant dual-write to close Checkpoint 1.
- Then: One stabilized minimal compact transport if needed.
- Then: The first hard-problem + deliberate full reset + fresh-process validator that produces numbers.

See [ROADMAP.md](docs/ROADMAP.md) for the exact phase plan and [docs/LEDGERS.md](docs/LEDGERS.md) for how all corrections are stored in the repo under `ledgers/` (git-tracked, watcher-protected, never in /tmp).

---

## Non-Goals (Strict)

- Full hydrodynamic/fluid memory swarm as the primary architecture (too big for v1; see "imagine a bunch of tiny.txt" for the vision).
- Multiple competing secret sauce versions (codec hell was the longest repeating trap).
- Massive autonomous swarms or general capability claims.
- Any code or doc that is not directly traceable to moving the north star loop.

---

## How to Run

```bash
cd mvp
cargo run
cargo run --features qdrant
```

**Protection discipline (non-negotiable):**
- The file watcher must stay green on this directory: `niodoo-watch`
- Baseline integrity snapshot exists at `/home/ruff/niodoo_integrity_20260529_210050/`
- All changes are visible in git. The 20-file curated receipts export lives in the flattened workspace.

---

## Required Reading (In Order)

1. [VISION.md](docs/VISION.md) — The actual north star in the user's words.
2. [MVP_DEFINITION.md](docs/MVP_DEFINITION.md) — Exact success criteria. This is the only definition of done.
3. [MASTER_RESEARCH_LEDGER.md](docs/MASTER_RESEARCH_LEDGER.md) — Evidence-only wide sweep. What actually moved the needle.
4. [KNOWN_PITFALLS.md](docs/KNOWN_PITFALLS.md) — The repetition patterns we are escaping. Read before adding scope.
5. [ROADMAP.md](docs/ROADMAP.md) — Phased plan with evidence anchors.
6. [ARCHITECTURE.md](docs/ARCHITECTURE.md) — Minimal layers + what we explicitly do not build.

---

## Philosophy (Lock)

We are done with iteration for iteration's sake.

Every line of code or documentation must satisfy:
1. Evidence from prior artifacts that this component produced real behavior change on the thesis (see MASTER_RESEARCH_LEDGER).
2. Direct, measurable contribution to "memory survives context death and produces retained improvement."
3. Radical minimalism. The smallest thing that can close the loop wins.

No more claiming. Only building, resetting, measuring, and documenting the numbers.

The past is not lost. It is in the artifacts, the 304-event ledger, the gamma runs, the watcher baselines, and these docs. This crate is the escape hatch.
