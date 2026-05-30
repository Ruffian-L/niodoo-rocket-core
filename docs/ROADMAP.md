# Roadmap — Phased Plan With Evidence Anchors

This roadmap is deliberately narrow. Every phase exists only to close the north star loop documented in VISION.md and MVP_DEFINITION.md. Phases are anchored to the actual signals in MASTER_RESEARCH_LEDGER.md.

**Current date for this plan:** Immediately after the full docs drop (May 30 2026 foundation).

---

## Phase 0 – Foundation (This Week — Mostly Complete)

**Goal:** The project has a clean, documented, protected home that future sessions (human or AI) can continue from without re-explaining everything.

**Done:**
- [x] Clean `niodoo-mvp` crate created (`Cargo.toml` with optional qdrant feature, minimal main.rs skeleton)
- [x] Full wide-sweep documentation dropped (this file + VISION + MVP_DEFINITION + ARCHITECTURE + MASTER_RESEARCH_LEDGER + KNOWN_PITFALLS + updated README)
- [x] Protection discipline active (watcher + git + integrity baseline on the mvp tree)
- [x] Objective record of what actually worked (gamma 143 rerun + 304-event claims ledger)

**Remaining in Phase 0:**
- [ ] Main binary builds cleanly and runs with `--help` that shows the real options (ledger path, qdrant, problem harness)
- [ ] CI smoke (cargo check + cargo test --lib) passes in the crate

**Exit criteria:** Anyone can `git clone`, read the docs in order, and know exactly what the project is and why the previous 100+ experiments did not close the loop.

---

## Phase 1 – Durable Memory Core (Next 1–2 Weeks)

**Goal:** Corrections and reflexes actually survive process death in Qdrant and can be loaded in a fresh run. This is the piece that was missing even when gamma and claims showed strong signals.

**Work (in strict order):**
1. Port/adapt the proven mistake-reflex ledger loader (`niodoo/src/runtime/mistake_reflex.rs` patterns) into this crate.
   - Support `--mistake-reflex-path` pointing at a JSONL (start with the 304-event claims ledger and the gamma policy_lifecycle.jsonl).
   - Implement the `influence` / `text-hint` action mode that actually changes generation behavior.
2. Wire Qdrant backend (feature-gated) for **dual write**:
   - JSONL always written as human-auditable source of truth.
   - Qdrant collection (`niodoo-corrections` or similar) receives the same events with per-turn refresh into in-memory stores.
3. One minimal, locked compact transport roundtrip (secret sauce or simple alternative). The gamma/claims wins happened in text-hint mode — do not let codec work block the first working loop.
4. Simple harness: "load ledger → run short session that emits a correction → explicit save → kill → fresh process → reload from Qdrant/JSONL → verify the reflex is active."

**Evidence anchor:** This directly replicates the gamma artifact triage and claims 9/10 conditions in a clean minimal binary.

**Exit criteria:** A fresh `cargo run --features qdrant` can write a correction during a run, be killed, and a second fresh invocation loads that correction and exhibits the changed behavior without any other context.

---

## Phase 2 – The North Star Test (The Real Gate)

**Goal:** Prove the exact loop on something that is genuinely hard for the base model.

**Work:**
1. Pick or synthesize one genuinely difficult problem class the 8B (or whatever base) consistently fails at without the ledger (use the artifact record for candidates — artifact triage meta-task itself is a strong one).
2. Run 1–N with corrections applied (user or stronger model) and reflexes written to Qdrant + JSONL.
3. Full deliberate reset: process killed, no warm context, minimal bootstrap prompt only.
4. Fresh process loads the stored corrections.
5. Measure clear, reproducible improvement on the same problem class (raw telemetry, stdout, success rate, specific error patterns avoided).
6. Full artifact + numbers + watcher log + git state captured.

**Evidence anchor:** This is the only thing the entire history has been missing. Gamma and claims were close (behavior change from ledger) but not the full "hard problem outside the meta-task + full kill + reappearance."

**Exit criteria for MVP v1:** We can point to a dated artifact directory containing:
- The exact problem prompt(s)
- Raw telemetry + stdout for the failing baseline
- Corrections applied
- Qdrant write + JSONL
- Fresh process logs showing the improvement
- Numbers and a one-page "this is what we actually proved" summary

When this exists and the numbers are real, **v1 is done**. Everything else is polish or Phase 3+.

---

## Phase 3 – Self-Steering (Post v1)

**Goal:** The model begins to use its own memory to steer without constant external correction.

- Automatic loading of stored reflexes at startup (no `--mistake-reflex-path` flag required for basic operation).
- Surface simple internal signals (entropy, TOPOCOT b1, viscosity proxies) that can trigger reflex lookup.
- Demonstrate measurable reduction in external correction needed across multiple reset cycles on the same problem class.

---

## Phase 4 – Compression, Portability, Fluid Memory (Later)

**Goal:** Make the memory practical at real scale and begin the hydrodynamic vision.

- One locked, reliable compact transport (secret sauce or niodv4-style) that can carry reflex packets or small state snapshots across processes efficiently.
- Basic Splat-style decay + reinforcement primitives on top of the ledger (pull minimal pieces from shep-loop/hydrodynamic-swarm only if they are clean and low-risk).
- First experiments with model-emitted tags (`<FOCUS>`, `<EXPLORE>`) that actually mutate live memory physics.

**Warning from the ledger:** This phase has historically been where the project fell back into codec hell and scope explosion. Do not enter until Phase 2 is closed with real numbers.

---

## Phase 5 – Minimal Usable Long-Running System

- Working durable memory loop on at least one hard problem class with documented cross-reset improvement.
- Clean `cargo run --features qdrant` experience.
- Full protection (watcher + integrity) green.
- Public or shareable artifact tree + writeup that a stranger can reproduce the core result from.
- Clear "this is what we actually proved, and here is the raw data" document (no hype).

---

## Scope Lock Rules (From KNOWN_PITFALLS)

- Anything not directly serving the Phase 1 → Phase 2 path is out of scope until the north star test produces numbers.
- New codec variants, full hydrodynamic swarms, multi-agent orchestration, or "just one more physics knob" are Phase 4+ or post-MVP.
- If a change would require re-fighting any historical repeating battle (codec versions, negative claiming, file mutation fears), it is blocked until the basic loop is closed.

**The past is documented. We are not allowed to repeat it.**

This roadmap is intentionally short. When Phase 2 is real, the rest becomes possible. Until then, everything else is distraction.
