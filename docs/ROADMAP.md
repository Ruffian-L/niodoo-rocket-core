# MVP Roadmap — One Short Doc (2026-06-01)

**The only goal that matters (verbatim):**
> Hit a genuinely hard problem → corrections applied → full context death (kill the process) → fresh process → the model performs better because it *actually internalized* the corrections via durable memory.

This is the entire North Star. Everything else is either helping close this loop or historical repetition trauma.

**Current state (as of this doc):** We have the pieces (signal-based repetition detector in `src/reflex/repetition.rs`, dual-write scripts, `splatrag_minimal/` as the tiny math core with working inversion test). The missing thing is wiring them into one clean, measurable path with visible "we are on path" signals.

---

## Key Terms (the absolute minimum)

- **North Star Test**: The loop above. Only real success metric.
- **Context Death**: Deliberate full kill with no warm state surviving.
- **Correction / Ledger Event**: Structured record of a mistake + what should have happened instead. Written to `ledgers/mvp_corrections.jsonl` (git-tracked source of truth) + Qdrant `niodoo-corrections`.
- **Dual-Write**: Always write to JSONL + Qdrant when possible.
- **Repetition Strength**: 0-1 score from pure signal correlation (ghost pulls, flawed monitors, no-progress markers, latency spikes, etc.) in `RollingTelemetryWindow`. No string matching.
- **treat_as_old_mistake / RepetitionEscalation**: When strength crosses threshold, escalate the stored correction (bump action level, retrieve reflex, apply steering).
- **Splat / Steering Vector** (from `splatrag_minimal/`): Valence-tagged Gaussian memory. `compute_steering_vector()` produces signed pull/repel. The tiny core that can feed `compute_ghost_vector`.
- **Inversion Test**: Store good-route (+valence) vs bad-route (-valence). After `dream_consolidate()`, steering vectors should be near opposites (cosine ≤ -0.85, sign-flip case -1.0). This is the model for all mathematical checks.
- **Human Hype KPI**: A visible thing that makes *you* (Jason) feel in your body "we are winning and not sliding back into the well" (e.g. a dated artifact folder where the memory clearly survived the kill and changed the outcome; the evaluation script says GREEN; watcher stayed green the whole cycle).
- **Drift Red Flag**: Early warning we are repeating old trauma (codec variants, >2 sessions on transport with no north-star delta, negative claiming language, scope creep without a math + hype KPI justification).

See the actual code for precise definitions (`repetition.rs`, `splatrag_minimal/core/`, the scripts).

---

## The One Standardized Benchmark (right now)

**Primary problem for the real North Star Test**: Gamma artifact triage meta-task (the May 29 seed 143 rerun that produced the cleanest Tier 1 signal in the history).

**Why this one**: Baseline repeated the wrong answer on weak evidence. Ledger-loaded version caught the exact failure on the first relevant turn. It is a real hard judgment task on actual project artifacts.

**Measurement rules (frozen)**:
- Capture: `telemetry.jsonl` (with the signals the repetition detector uses), full stdout/stderr, the exact ledger written, watcher log, git state.
- Minimum: 5 matched pairs (baseline fail vs post-reset improved) on the same prompts/seeds.
- Attribution must be credible (the loaded corrections are the only material difference).
- Artifact: one dated folder with a one-page `NORTH_STAR_RUN.md` + the raw files above.

**The living math checks (these are the spec — run the code, don't read more docs)**:
- `splatrag_minimal/core/inversion_test.py` — the style every math check must follow. Currently gives clean PASS with sign-flip cosine = -1.0000.
- `cargo run -- simulate-window --scenario gamma_with_ledger` (or the real telemetry version via `scripts/process_real_run.py`).
- Dual-write fidelity check (JSONL count vs Qdrant points for the same run).

When someone asks "what is the benchmark?", the answer is: "Run the inversion test + process a real stuck gamma-style run through the repetition detector + show the dated artifact after a kill+reload on the gamma triage task."

---

## Checkpoints (tight, visible goals + signals)

**Checkpoint 0 — Foundation (Done)**
- We have the detector, the tiny splat core with working inversion test, the ledger scripts, and this short doc.
- Human hype: I can run the inversion test and see the evolvability signal (-1.0 on sign flip). The pieces are in one tracked mvp folder.

**Checkpoint 1 — Corrections + Detector Actually Fire on Real Runs (Next target)**
- Math: `process_real_run.py` on real telemetry produces corrections that load in a fresh process and change behavior. Repetition strength crosses threshold on real stuck patterns and triggers escalation.
- Human hype: I take one of my real gamma stuck runs, run it through the detector, see corrections land in the tracked ledger, `embed-up + ingest`, then in a fresh process the reflex actually prevents the old mistake. Watcher green the whole time. I have the dated folder.
- Red flag: More than 2 sessions on transport before this loop is green on at least one real run.

**Checkpoint 2 — First Real North Star Gate (The one that matters)**
- Math: On the gamma triage task, full sequence (fail → corrections captured → deliberate kill → fresh load) produces clear, attributable improvement (success rate delta or equivalent judgment quality) with 5+ matched pairs. Dual-write fidelity 100%. Bonus: real corrections run through the splat core give strong negative cosine on steering vectors.
- Human hype (the body signals): I have a dated artifact folder. I open it, read the baseline failure, the corrections, the fresh-process output, and feel "it actually survived the kill and changed the exact thing we stored." The evaluation commands print clear positive signals. No hedging.
- This is v1 complete when the numbers + the body feeling are both real.

**Checkpoint 3+ (only after 2 is real)**
- Reduction in how often you have to intervene (the model starts catching its own repetitive failures earlier via the detector + stored memory).
- Later: one locked minimal transport + the smallest safe splat decay/reinforcement pieces extracted from the tiny core. Never the full hydrodynamic thing until the loop is proven.

---

## Rules (to not get stuck in the well again)

- Every new piece of work must justify itself against Checkpoint 1 or 2 with both a mathematical check (runnable, thresholded, like the inversion test) **and** at least one human hype KPI.
- No new codec variants, "one more physics knob", or scope justified by "it will help later."
- The watcher must stay green on everything that contributes to a checkpoint.
- When in doubt, run the existing tests (`inversion_test.py`, `simulate-window`, `process_real_run.py` on a real stuck artifact) and look at a dated folder. If those don't light up, we are probably drifting.

**This is the whole plan in one short doc.**

The actual benchmark lives in the runnable tests:
- `splatrag_minimal/core/inversion_test.py`
- `src/reflex/repetition.rs` + the binary simulate + `scripts/process_real_run.py`

Run the code. Look at the artifacts. Feel whether the body signals are green. That is the roadmap.

No more separate walls of documentation. One short doc + the living tests. That's it.

— 2026-06-01 (consolidated per explicit "not too much documentation" direction)
