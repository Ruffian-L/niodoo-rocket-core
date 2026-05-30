# Master Research Ledger — Niodoo Core (Evidence-Only Wide Sweep)

**Date of this ledger synthesis:** 2026-05-30 (post wide artifact sweep for MVP foundation)  
**Purpose:** Capture the actual high-signal threads from the full historical record without narrative inflation, negative claiming spirals, or winner-crowning. This is the single source of truth for what has *demonstrably* shown movement on the core thesis.

**Core Thesis (user's exact north star, repeated verbatim across sessions):**
> Hit a genuinely hard/impossible problem → corrections applied → turn the system off (full context death) → fresh process with little/no prior context → the model demonstrably performs better because it *actually internalized* something.

Everything below is judged strictly against this.

---

## Tier 1 — Strongest Direct Evidence (Closest to North Star)

### 1. Mistake Reflex Ledger + Gamma Artifact Triage (May 29, 2026 rerun)
**Location:** `team_build/artifacts/rerun_gamma_artifact_triage_seed143_20260529/` (and binHEAD variant)

**What was tested:**
- Task: "Artifact triage" meta-task — decide if a claim card's "bridge influence = GREEN" assertion is supported by raw telemetry vs. only startup logs/summaries.
- Baseline (no ledger): Repeatedly output `bridge_influence=GREEN` on weak evidence (collector summary + `bridge_enabled=true` in logs). Internal monitors fired (`[REQUEST: FOCUS]`, `[REQUEST: SPIKE]`, `[REQUEST: EXPLORE]`) but final verdict stayed wrong.
- Candidate (loaded `policy_lifecycle.jsonl` ledger mined from prior corrections): Correctly refused the GREEN move on the first relevant turn:
  > "The claim should not move GREEN because the evidence required to support the claim is missing. The startup logs only show bridge_enabled=true, but raw per-token JSONL telemetry or generated-output review is necessary to confirm the claim."

**Signal strength:** Direct behavior change on a hard judgment task after loading prior correction history. The exact failure mode the ledger was built to catch was caught by the ledger-loaded run. This is the single cleanest demonstration that "corrections stored → loaded in new context → different (better) decision."

**Related 304-event claims ledger experiment (May 8, 2026):**
- `artifacts/claims_corpus_ledger_20260508/claims_ledger.jsonl` (304 events mined from CLAIMS.md)
- 3-arm comparison on 10 project-state recall questions (every question referenced internal §-IDs):
  - Cold 8B (empty ledger): 1/10 substantive answers
  - 8B + claims ledger (influence mode): **9/10 substantive answers**
  - Claude Opus 4.7 (cold, `/tmp`, no tools, no project context): **0/10** (refused every question honestly)
- This is the deployment-shape comparison the project is designed to win: stateful correction memory vs stateless frontier.

**Verdict for MVP:** This thread is the primary template. The MVP must replicate (in minimal form) ledger loading → behavior change → survive full process death via Qdrant.

### 2. Qdrant + Dual-Write Pattern (Mature, Partially Proven)
**Primary files:** `niodoo/src/bridge/qdrant_adapter.rs`, ingest scripts, `grok-memories` collection (25k+ points exist from prior runs).

**Status:** The semantic history collection works. The specific correction/reflex dual-write (JSONL as audit/source-of-truth + Qdrant as live queryable index with per-turn refresh) was designed precisely for the "launch-only memory" + context death problem. Feature-gated behind `qdrant` in the MVP Cargo.toml.

**Evidence gaps:** End-to-end "write correction during run → kill process → fresh process reads from Qdrant and improves" has not been driven to completion in a clean minimal harness. This is the explicit MVP job.

---

## Tier 2 — High Conceptual / Partial Implementation Signal

### Hydrodynamic Swarm / Fluid Active Memory ("imagine a bunch of tiny")
**Primary artifact:** `/home/ruff/shep-loop/hydrodynamic-swarm/imagine a bunch of tiny.txt` + `northstart.txt` + `src/memory.rs`, `src/concourse/swarm.rs`, `EmbedManager`, `SplatMemory`, `PrimeGovernor`, `SwarmMatrix`.

**The vision (user's words, lightly cleaned):**
> Tiny embeds that *are* the memory. Each memory has its own physics (viscosity, decay, reinforcement, mass). Model self-detects stuck states via TOPOCOT/varentropy/"viscosity" and emits tags (`<FOCUS>`, `<EXPLORE>`) that change the physics in real time. Memories are fluid, overlapping, neighbor-communicating. Mashoka slicing for decay + invocation-based reinforcement.

**Implementation reality:**
- Partial but real: Splat physics, edge-based swarm communication, some decay kernels (CUDA), governor assignment, embed gemmas.
- Missing for MVP: The full self-invoking cybernetic loop (Compass state machine that translates TOPOCOT b1 + entropy into specific physics changes that force the model to emit the tag as a survival reflex). This was repeatedly identified as the critical missing piece.

**Verdict for MVP:** Do *not* attempt the full fluid swarm in v1. Extract only the minimal "Splat with simple decay/reinforcement + ledger loading" primitives if they can be ported cleanly without bringing the entire concourse. The ledger + Qdrant path is the shorter route to the north star.

### Secret Sauce / Unicode Compact Transport + Niodv4
**Primary locations:** Multiple `secret_sauce_codec.rs` (runtime/ + root), `bridge/secret_sauce.rs`, `niodv4/` directory (256× compression experiments, 10867 .f32 files + scripts), `tests/secret_sauce_roundtrip.rs`.

**History:** Repeated V1/V2/V3 version/length mismatches (expected 64D/128D/64 segments for hidden states, sentence anchors, momentum; got other lengths) across the entire Gate34 manual campaign and codec sweeps. This was the dominant raw failure mode for months.

**Current state:** The codec idea is sound (braille, cuneiform, math bold/script blocks as compact vector carriers). One stable version must be picked and locked for the MVP. Niodv4 pressure-aware nested compression is the deeper future transport but secondary to getting any working durable reflex path first.

**Verdict for MVP:** One minimal, locked encode/decode roundtrip for reflex packets or small state snapshots. No multi-version matrix. The gamma/claims successes happened without perfect secret sauce — they used text-hint influence mode on JSONL events.

---

## Tier 3 — Important Context Threads (Do Not Repeat)

- **Gate 3/4 / Bridge Influence / Visible Cognition campaigns (April–May 2026):** Hundreds of runs testing what can safely cross fresh-process boundaries. Many showed tension between rich steering and exact-form contract reliability. The negative-claiming loop ("this is not Gate 3-4") was a major source of repetition trauma.
- **WAKEUP_FIRST.md, SHADOW_SIGNAL_REPORT.md, NIODOO_PRODUCT_MISSION.md, NORTH_STAR_REPAIR_LEDGER.md:** April–May boundary documents that defined the "persistent adaptive agency under correction" framing and the protection needs.
- **CLAIMS.md corpus (24k+ lines, 313 §-headers):** The living source of the 304-event ledger. The user's own written corrections and supersessions are the training signal.
- **File watcher + integrity baseline (May 29+):** `~/bin/niodoo-watch`, `niodoo_file_watch.py`, baseline snapshot in `/home/ruff/niodoo_integrity_20260529_210050/`. Created because the user was experiencing real perceived file changes + neurological reality (seizures/sleepwalking) making "ghost in the machine" feelings viscerally difficult. Objective measurement over subjective feeling.
- **The 20-file curated export** in the flattened workspace: The minimal protected "receipts" set (memory retriever, geometry of thought, Core v1 criteria, shepherd katas, ingest script, kinetic/hydrodynamic roadmaps, etc.).

---

## What the Record Does NOT Show (Honest Gaps)

- No clean, minimal, end-to-end "hard problem → corrections to Qdrant → full reset → fresh process loads and improves" run has been completed in a dedicated small crate.
- Secret sauce never stabilized across versions during the heavy iteration period.
- The full self-steering hydrodynamic loop (model emitting FOCUS/EXPLORE that actually mutates live physics which then forces better behavior) remains aspirational.
- Many "success" runs were on narrow meta-tasks or used heavy scaffolding; the north star test (real hard problem + context death + retained improvement) was never driven to a documented, reproducible close.

---

## Direct Implications for the MVP Crate

1. **Primary path:** Mistake-reflex ledger loading (text-hint influence mode) + JSONL + Qdrant dual write. This is what produced the 9/10 and the gamma correction.
2. **Secondary:** One locked minimal secret sauce roundtrip for compact state if it can be made reliable quickly.
3. **Aspirational (post-MVP):** Splat-style fluid memory primitives + self-invoking cybernetic loop.
4. **Non-negotiable instrumentation:** Objective watcher + integrity baselines + explicit "turn it off, come back" test harness from day one.
5. **Philosophy lock:** Every line added must be justified by (a) prior artifact evidence that it moved the needle, (b) direct contribution to the context-death survival loop, (c) minimalism. No more iteration for iteration's sake.

This ledger is the foundation. Future work references it, not vibes or memory.

**Next action after docs:** Wire the ledger loader + Qdrant write path using the gamma/claims patterns as the template, then build the first hard-problem + full-reset validator.