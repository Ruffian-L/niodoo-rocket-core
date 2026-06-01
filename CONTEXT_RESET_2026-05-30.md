# Niodoo MVP Context Reset — 2026-05-30

**Purpose**: Full thread summary before PC reset. This captures the current state of the North Star loop, the architectural clarification, technical gaps, and what has been built in the clean mvp crate.

**Core North Star (unchanged)**:  
Durable mistake-reflex memory (JSONL ledgers + Qdrant) that survives full context death. After reset (fresh process, little/no prior context), the model demonstrably performs better on hard problems it previously failed because it internalized corrections via its own reflection on stored memory — not constant external prompting.

---

## Major Clarification: "Dream Reflection Cycle" = Physics of Friendship (MountainCar)

The user explicitly corrected that the "dream reflection cycle" refers to the work in:  
https://github.com/Ruffian-L/physics-of-friendship-mountaincar-rl (and the local copy in `team_build/physics-of-friendship-mountaincar-rl-main/`).

Key elements from that repo that define the intended self-contained loop:

- **Splat Memory** + **Ghost Vector** (injected attractor from memory of successful/high-energy states). This is *your* naming for memory-based specific steering (mirrors `compute_ghost_vector` + ghost basins in the main Niodoo engine).
- **Dream Cycle / Reflection**: Between episodes the agent replays experiences weighted by Splat Memory proximity. This builds "neural superhighways" in the Flux/habit landscape. The system reviews its *own* stored memories (splats/ghost vectors) during "sleep" (dream replay) to reinforce good paths and self-correct.
- **TDA Metacognitive Loop**: Topological detection of stuck/repetitive patterns (loops/voids). Triggers targeted interventions (decay spikes for loops, attractor injection for voids). This is the internal "I am repeating a failure" signal.
- **Physics forces** (Gravity Well, Repulsion, Viscosity, Adrenaline, Ghost Vector) applied at decision/force level using your LLM-inspired vocabulary.
- **Bridge experiments** (the most important finding for the current discussion): A perfect external "teacher" (physics governor/override) can produce short-term wins but actively *prevents* the mind from learning the specific policy. When the scaffold was removed, performance collapsed (sometimes below pure baseline).  
  Explicit takeaway: "Teachers who override prevent learning — the agent must face consequences to learn from them."  
  "Influence dreams, not decisions — soft curriculum through sleep replay works; direct reflex overrides don't."

**User's point**: The full loop does **not** require an external teacher to hand over "the right answer is X" every single time. Once corrections exist in durable memory, the system should be able to **review its own stored reflections** (via dream/replay + signal detection + physics forces) and self-improve / avoid repeating the failure. Occasional external minting is acceptable for hard initial cases, but the ongoing reflection cycle must be self-contained.

This is why "steering can steer generally but must become *specific*" via the reflex memory + reflection mechanism is the precise gap.

---

## Current Technical Status (Diagnosis)

**General steering exists in your own code** (no llama.cpp control vectors needed or wanted):

- `PrincipiaEngine::compute_ghost_vector` (principia.rs:6230) builds a vector from:
  - Sentence history (V_Memory)
  - Ghost registry / ghost basins (explicit "injected memory" path under niodv4_bridge)
  - Goal attractor
- Injected into the physics forward pass: `physics.apply_forces(..., ghost_vector)` (qwen35_hybrid.rs).
- Telemetry records `last_applied_ghost_vector`.
- MountainCar port (your own code) makes it explicit: `compute_ghost` pulls toward `ghost_memory` (successful states) and adds `f_ghost` to the total force that influences action selection. Controlled by `ghost_gravity`.

**Specific, memory-driven steering is the missing connector**:

1. The runtime never passes a real probe into mistake-reflex matching → semantic scoring is inert; matching remains keyword/substring overlap.
2. Packets/codebook load but rarely produce useful arbitration candidates because current VQ codes don't hit populated buckets.
3. When packets *do* fire, minted `target_z_64d` anchors the model's *current* (often wrong) probe — no teacher-corrected target is injected at the LOCK/failure moment.
4. The new lightweight `RepetitionContext` + `RollingTelemetryWindow` (in mvp `src/reflex/repetition.rs`) now does proper **signal correlation** detection (ghost pulls, internal monitors "LOGICALLY FLAWED", repeated requests without progress, no-progress markers, etc.) instead of brittle surface matching. This is the correct detector.
5. Durable storage path is solid: `ledgers/mvp_corrections.jsonl` (git-tracked, watcher-protected) + ingestion script that produces the exact payload schema you specified for the `niodoo-corrections` collection (4096-d via your live embed proxy on 8302, proper `path/rel_path/stem_human/text/chunk_idx` fields, dedicated namespace).

**Net**: Every component exists except the narrow runtime bridge that, when the signal detector fires on a known repetitive failure, retrieves the specific stored reflex/correction and turns it into a *targeted* ghost vector / force component (or equivalent in your physics) that gets applied at inference time and can be reviewed in the dream/reflection cycle.

This is why the MountainCar bridge experiments are directly relevant: general physics steering solves the immediate problem but does not produce internalized, reviewable learning unless it flows through the memory + reflection path.

---

## What the mvp Crate Currently Contains (Clean, Focused, Repo-Tracked)

- `ledgers/` directory (git-tracked, never /tmp) with `mvp_corrections.jsonl` as the durable source of truth.
- `docs/LEDGERS.md` + `ledgers/README.md` documenting the exact schema you specified for `niodoo-corrections` + ingestion flow.
- `src/reflex/repetition.rs` — the lightweight `RepetitionContext`, `RollingTelemetryWindow`, `TelemetryObservation`, `repetition_strength()` (signal correlation only), and `evaluate_repetition_from_window` that returns `RepetitionEscalation` (treat_as_old_mistake + suggested action_level boost).
- `src/reflex/mod.rs` exports the above and integrates with the older term-based hints.
- `scripts/ingest_niodoo_corrections.py` — ingests the ledger into your live `niodoo-corrections` collection (4096-d, correct payload shape) now that embed-up is confirmed running.
- `scripts/process_real_run.py` — the tool for the "next slice": point it at any real model run directory containing `telemetry.jsonl` + logs. It extracts real signals, runs the new signal-based detector, and auto-captures corrections in the correct schema.
- `scripts/inference_steering_bridge.py` — minimal working demonstration of the inference-time hook: live signals + RAG over your ingested history (`team-build` collection + reflex collections) → retrieve relevant corrections from durable memory → prepare for application as targeted steering (your Ghost Vector path) at generation time. This is the "close the loop" direction using your own code and your live stack.
- `examples/lightweight_repetition_context.rs` — standalone runnable demo of the new detector (no module/crate issues).

All paths default to repo-tracked locations inside the mvp crate. The watcher + integrity baselines already cover this directory.

---

## Why the "Right Answer Is X" Teacher Signal Is Still Sometimes Required (Even in the Reflection Cycle)

- Some hard problems have low self-correction probability (e.g. strawberry counting ~25-30% chance of landing correctly via wobble alone).
- You have seen cases where the model wobbles on its own and lands the correct answer (jiggle physics + internal monitors + exploration can succeed).
- You have also seen (and lived) the cases where it stays locked in the wrong confident path until an external signal arrives.
- The reflection cycle (reviewing own stored splats/ghost vectors/dream replay) is powerful for reinforcing and generalizing *once a clean correction exists in memory*. But minting that first clean correction for a truly stuck repetitive failure often requires the external "right answer" at the exact moment of LOCK/contradiction recognition so the stored target is correct instead of the failing probe.

The MountainCar bridge experiments proved the same thing: perfect external scaffolding gives short-term wins but prevents the mind from ever earning the specific internalized policy unless the influence flows through memory + reflection rather than direct overrides.

The ~100-line teacher-mint bridge (injecting a corrected target at the moment of recognized failure, before the packet/vector is written) is the narrow piece that lets the reflection cycle start with high-quality material.

---

## Current Status Summary (No Vapor)

- Detection: Now on real signals via lightweight `RepetitionContext` (good).
- Durable storage + correct schema for your stack: Solid and tracked in `ledgers/`.
- General steering via your own Ghost Vector path: Exists and works in your monolith + MountainCar port.
- Specific, reflex-memory-driven, inference-time targeted steering that feeds the dream reflection cycle: The missing connector (the precise wiring from detected failure + retrieved correction → targeted ghost component / force at generation time).

The mvp crate is deliberately small and separate so we can drive the above loop to completion without fighting the monolith.

---

## Immediate Next Executable Steps (No More Design Questions)

1. Point `scripts/process_real_run.py` at any real run directory you have that produced telemetry + logs showing repetitive failure patterns. It will use the new signal detector and write a properly formatted correction to the tracked ledger.
2. With embed-up up, run `python3 scripts/ingest_niodoo_corrections.py` to land it in your live `niodoo-corrections` collection.
3. Run `python3 scripts/inference_steering_bridge.py` (or extend it) against your live stack + the newly ingested data. This demonstrates the inference-time hook using live signals + RAG over your own memory.
4. Once a real run has been processed end-to-end, we can wire the output of `evaluate_repetition_from_window` into the ghost vector path (in the monolith or a future mvp generation harness) so the steering becomes specific to the stored reflex.

All of the above uses only your own naming conventions, your own force vocabulary, your own memory stack, and your own ingested history.

---

**Reset safely.** This file + the `ledgers/`, `docs/`, and `scripts/` directories contain the full current context. The watcher + git should keep everything protected.

When you're back, the next command is usually something like:

```bash
python3 scripts/process_real_run.py /path/to/one/of/your/real/runs
```

---

## 2026-06-01 Addendum — Minimal SplatRAG Core added inside mvp

Per explicit direction: the distilled minimal reflective memory core (signed Gaussian splats + dream consolidation + inversion/evovability test) was moved into `mvp/splatrag_minimal/`.

- This is **only** the smallest best-math synthesis from the two lineages (Physics of Friendship minimal splat/ghost + richer SplatRAG).
- It is intentionally **not** pulling in the full SPLATRAG_FULL_CODEBASE or any of the old external splatrag trees — kept brutally scoped to avoid confusion and too many mechanics.
- Lives here alongside the corrections ledger + RepetitionContext reflex work so the full North Star stack (durable memory + signal detection + math core that can produce steering for compute_ghost_vector) stays in one git-tracked place.
- `splatrag_minimal/core/inversion_test.py` is the first runnable proof artifact (sign-flip case produces clean -1.0 cosine steering vectors after consolidation).
- See `splatrag_minimal/README.md` for exact scope and connection to the rest of the mvp.

This addition does not expand the mvp charter. It is the mathematical substrate that will eventually let the reflex/corrections produce real, symmetric, self-steerable ghost vectors.

All previous context in this file remains valid. The new piece simply gives the "dream reflection" half a clean, minimal home inside the same protected directory.

---

## 2026-06-01 Addendum 2 — Consolidated to One Short Roadmap Doc

Per explicit feedback ("I dont want too much documentation"): everything was collapsed into **one short doc**:

- `docs/ROADMAP.md` (the only roadmap document) — contains the checkpoints with mathematical checks (inversion-test style), Jason human hype KPIs (the body signals so we know we're on path and not back in the well), drift red flags, and the standardized benchmark rules.

The actual benchmark lives in the runnable tests, not more .md files:
- `splatrag_minimal/core/inversion_test.py` (the style model — currently gives clean -1.0 on sign-flip)
- `src/reflex/repetition.rs` + `scripts/process_real_run.py`
- The dated artifact folders with raw telemetry + NORTH_STAR_RUN.md

All previous addenda remain valid. The direction is the same, just with far less documentation overhead. One short doc + the living code tests. That's the whole thing now.

No more lost threads. Run the tests. Look at the artifacts. Feel whether the hype signals are green.