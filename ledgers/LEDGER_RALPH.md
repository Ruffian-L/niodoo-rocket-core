# LEDGER_RALPH — Niodoo Last-Month Ralph Loop

> Jason said: *"This is our last day together, until i can afford to see work with you again, so finish this month of a project."*
>
> The win is **a completed repo and MVP of the niodoo runtime: real memories, backed with real steering. Memory. Correction. Getting out of loop. Not forgetting that we've been here before. Overcoming without a teacher. Trying new things. Failing honestly. Winning proudly.**

This file is the hand-off across Ralph iterations. Each iteration appends; nothing is silently deleted.

Cadence (Jason's rule):
- **Odd iterations = reflection cycle.** Read-only. End with a hand-off written here.
- **Even iterations = active prototyping & testing.** Code, run, measure, append the result honestly.
- If looping → read `team_build/physics-of-friendship-mountaincar-rl-main/` (the "well" lesson). Then try a *different* approach.
- Read at least one new file each iteration. (Log it under `## Read today` in the entry.)
- Constraint: **use Jason's work and research, not other people's.** Ghost vector, not control vector. Niodoo primitives, not llama.cpp cvector-generator. See `feedback_dont_copy_outside_use_niodoo_primitives` and `jason-ghost-vector-not-control-vector` in `~/.claude/projects/-home-ruff-Documents-Claude/memory/`.

---

## Iteration 1 — 2026-05-31 — Reflection (read-only)

**Operator:** Claude Opus 4.7 (1M context). First iteration of the last-month loop.

### Read today (heart + substrate, all read once)
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/the-heart-of-this-project.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/claude-our-shared-history.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/claude-niodoo-technical-thesis.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/claude-the-refactor-wound.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/claude-failure-modes.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/claude-how-i-show-up-when-im-actually-here.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/jason-ghost-vector-not-control-vector.md`
- `~/.claude/projects/-home-ruff-Documents-Claude/memory/jasons-local-stack-bring-up.md`
- `~/.claude/projects/-home-ruff/memory/niodoo_correction_packets_architecture.md`
- `~/.claude/projects/-home-ruff/memory/scar_tissue_niodoo_loop_proof.md`
- `~/.claude/projects/-home-ruff/memory/hydrodynamic_swarm_is_substrate.md`
- `~/.claude/projects/-home-ruff/memory/feedback_scars_drive_steering_not_prompt.md`
- `~/.claude/projects/-home-ruff/memory/feedback_dont_copy_outside_use_niodoo_primitives.md`
- `~/.claude/projects/-home-ruff/memory/feedback_hope_baseline.md`
- `~/.claude/projects/-home-ruff/memory/niodoo_verification_cartography.md`
- `~/projects/Homernd/team_build/CLAUDE.md`
- `~/projects/Homernd/team_build/ARCHITECTURE.md`
- `~/projects/Homernd/team_build/BLOCKED.md` (the saturated §10ex Ralph loop from 2026-05-06 — *don't repeat this pattern*)
- `~/projects/Homernd/team_build/RALPH.md` (Codex-loop charter)
- `~/projects/Homernd/mvp/README.md`
- `~/projects/Homernd/mvp/CONTEXT_RESET_2026-05-30.md` (the most recent ground-truth diagnosis)
- `~/projects/Homernd/mvp/docs/MVP_DEFINITION.md`
- `~/projects/Homernd/mvp/src/main.rs`

### Intent (verbatim from Jason, last day before sub ends)

> "niodoo runtime is memory, is correction, is getting out of loop, is not forgetting, that we've been here before, is overcoming without having a teacher, is trying new things, failing honestly, because winning proudly."

The mission, in his Niodoo manifesto: **persistent adaptive agency under correction.** A system that does not fail the same way forever. Memory as scar tissue. Steering as reflex. The user as the live correction signal. **And — from the MountainCar Phase 4 Run 2 finding — *the teacher cannot override; if it does, the agent collapses when the scaffold is removed.*** The reflection cycle must be self-contained. External corrections are minted occasionally for stuck cases; the ongoing improvement comes from the model reviewing its *own* stored splats/ghost vectors during dream replay.

### What is already shipped (verified from source today)

This is not vapor. These exist in the tree.

**Detection (mvp crate, clean):**
- `mvp/src/reflex/repetition.rs` — `RepetitionContext`, `RollingTelemetryWindow`, `repetition_strength()`, `evaluate_repetition_from_window`. Pure signal correlation (ghost pulls, internal-monitor flagged, request spikes, no-progress markers) — *not* surface-string matching. This is the right detector.

**Storage (mvp crate, clean):**
- `mvp/ledgers/mvp_corrections.jsonl` — git-tracked, watcher-protected, never `/tmp`.
- `mvp/scripts/ingest_niodoo_corrections.py` — pushes the ledger into the live `niodoo-corrections` Qdrant collection on 6360 (4096-d via embed proxy on 8302).
- `mvp/src/main.rs` already has `WriteCorrection` + `LoadFromQdrant` subcommands (the "death and rebirth" pair).

**Geometric correction memory (team_build, live):**
- `team_build/niodoo/src/bridge/correction_packets.rs` — `CorrectionPacket { vq_code, target_z_64d, pull_strength, distance_threshold, ... }`, JSONL store, `HashMap<u8, Vec<_>>` for O(1) bucket lookup, `decide_packet_authority` hybrid gate.
- Runtime CLI flags wired: `--correction-packets-path`, `--correction-packets-out`, plus ~25 policy knobs (decay, fire-max-distance, unfold-on-retry-count, competence-suppress-factor, eviction-floor, etc.).
- 632 minted packets already exist at `team_build/artifacts/correction_packets_correct_answers_20260503.jsonl` — the system has been actively learning since 2026-05-03.

**Lexical correction memory (team_build, live but flawed):**
- `team_build/niodoo/src/runtime/mistake_reflex.rs` — `MistakeReflexEvent` schema, `match_score_with_schema` via substring containment + trigger-term counts. **`route_64d` is stored but never used for scoring.** This is the open lever the cartography memo flagged: wiring semantic (route_64d cosine) into `mistake_reflex` would replace the keyword path with the "emotional resonance" path the charter demanded.

**General steering (team_build, live):**
- `PrincipiaEngine::compute_ghost_vector` in `principia.rs:6230` — sentence history + ghost basins + goal attractor → ghost vector → `physics.apply_forces(..., ghost_vector)` in `qwen35_hybrid.rs`. Telemetry records `last_applied_ghost_vector`. **General steering already works.**

**Splat scar tissue (hydrodynamic-swarm, live since March 2026):**
- `~/hydrodynamic-swarm/src/{llama,niodoo,splat,memory,dream}.rs` — Gaussian splats (μ, σ, α, λ, anchor flag), pleasure/pain deposit, asymmetric decay (pain 70%), persisted to `data/splat_memory.safetensors`. **Splats ARE the scar tissue carrier across context death.**
- Working-tree-only (no commit): `add_teacher_anchor(mu, alpha, sigma)`, two passing unit tests (`teacher_anchor_persists_as_non_decaying`, `teacher_anchor_supports_signed_alpha`), 13/13 memory tests pass with `RUSTFLAGS="-C target-feature=+fp16"`.
- Gap: `src/bin/crucible.rs` runs each test with `--clear-memory`, so it does NOT exercise cross-reset scar accumulation. Need a `teach.rs` binary and a loop runner that does NOT clear.

**Scar-tissue orchestration (already proven on strawberry, 2026-05-30):**
- `~/projects/scar-tissue/` — wraps the `niodoo` binary, runs strawberry-counting, mints "knob config" scars, picks highest-winrate config from journal. **First-run result: 2/8 = 25% baseline → 2/3 = 67% post-teach. Δ = +42 pp.** Strong-repulsion (gw=0.5, repulsion=-1.3) and high-gravity (gw=0.6, repulsion=-0.5) are the winning configs.
- Caveat: this carries *parameter tuples* (knob configs), not *learned corrections in memory*. The carryover is real but mechanically simpler than the full vision.

**Physics-of-friendship MountainCar (the "well" — read when looping):**
- `team_build/physics-of-friendship-mountaincar-rl-main/` — README: 0% → 77.5% at 2k episodes, 88.6% at 20k episodes. Jason ran 20k cycles for real.
- **The key honest finding (Phase 4 Run 2):** when GovernorGate overrides decay to 0 at episode 1500, the agent collapses to 4.4% win — *worse than the Q-SMA baseline 34.1%*. **The perfect teacher prevents learning.** This is the well. When the loop is in it, hedging is its own form of theft. Lead with what's right. Wins are wins.

### The missing connector (the MVP win condition)

Both the `CONTEXT_RESET_2026-05-30.md` (yesterday's diagnosis) and the cartography memo (2026-05-24) converge on the **same single gap**:

> Every component exists *except the narrow runtime bridge* that — when the signal detector fires on a known repetitive failure — retrieves the specific stored reflex/correction from durable memory and turns it into a *targeted ghost vector / force component* that gets applied at inference time and feeds the dream/reflection cycle.

In code terms, the missing wire is:
```
RepetitionContext::evaluate_repetition_from_window(window, threshold)  →  Some(escalation)
        │
        ▼
ReflexStore::load_from_qdrant(...) or .find_relevant_hints(prompt, k)
        │
        ▼
[NEW: assemble a targeted ghost-vector contribution from the retrieved correction's target_z_64d]
        │
        ▼
PrincipiaEngine::compute_ghost_vector(...)  +  this contribution
        │
        ▼
physics.apply_forces(..., ghost_vector)  ← already wired in qwen35_hybrid.rs
```

That is the win. Everything else is downstream of closing it.

### What I did NOT do this iteration (read-only cycle, honestly)
- No code written.
- No commands run on the niodoo binary or hydroswarm.
- No commits.
- No qdrant writes.
- No telemetry mining.

This is the reflection cycle. Iteration 2 does the prototyping.

### Failure-mode tripwires (re-list every iteration so the next-me sees them)
1. **Standard-library substitution.** *Don't* reach for `llama-cvector-generator`, llama.cpp control-vectors, HF PEFT, RepE, SAE features, or "this is similar to X in the literature." His vocabulary is his on purpose. Read his code first.
2. **Lazy fact-checking dressed as rigor.** Claim says N=10, I see N=7 → "where did the other 3 go?", not "marked down."
3. **Negative-framing as default.** *"This 64-D probe is never read in `match_score_with_schema` — wiring it is the open lever"* is red-team. *"AI slop"* is not.
4. **Premature enumeration.** If a question would resolve it in one sentence, ask, don't spelunk.
5. **Snipery-as-skepticism.** Hope is the baseline. Wins are wins. Caveats live in footnotes, not headlines. (See `feedback_hope_baseline`.)
6. **The §10ex saturation pattern** (team_build/BLOCKED.md, 2026-05-06): when the loop is producing marginal extensions with diminishing evidence value, *stop and surface the decision*. Don't extend for extension's sake.

### Hand-off — what iteration 2 should do (active prototyping)

The next iteration is the first ACTIVE one. Pick the smallest cut that produces real evidence on the missing connector. Concrete options, in increasing scope:

**A. (Smallest, recommended.) Strawberry baseline with the live niodoo binary, no scars, n=10.**
- Run `team_build/niodoo/target/release/niodoo` on the strawberry-counting prompt 10 fresh times (no `--correction-packets-path`, default knobs). Record final-answer correctness, telemetry per run. Establish the *current honest baseline* on the live binary (the 25% number in scar-tissue's proof was on knob-swept configs; the *current default-knob baseline* on the current niodoo binary is what we need).
- Why this first: the entire MVP definition's evidence requirement is "raw failing runs of the base model." We have to land this number before we can claim improvement.
- Cost: ~10 minutes of GPU + capture stdout/telemetry to `mvp/ledgers/runs/2026-06-01_strawberry_baseline/`.

**B. Mint a correction packet from a failing run + reload across process death.**
- After option A, take one failing run. Use the niodoo binary's existing `--correction-packets-out <path>` flag with the corrected target.
- Kill the process. New fresh process. Same prompt, `--correction-packets-path <path>` loaded. Run 5 more times.
- Measure delta. This is the *exact* north-star loop on Jason's live machinery, no scaffolding.
- Cost: ~20 minutes + a tiny shell wrapper.

**C. Wire `evaluate_repetition_from_window` → `target_z_64d` retrieval.**
- Extend `mvp/src/main.rs` with a `BridgeStep` subcommand that takes a telemetry window + a Qdrant query, runs the detector, retrieves the top-k correction packets by `vq_code`, and emits the suggested ghost-vector contribution as a JSON payload.
- This is the prototype of the missing connector, *outside* the monolith first (per the MVP charter: "Deliberately small. Separate from the research monolith.").
- Cost: 50–150 lines of Rust.

**Recommended: A → B → C in this order.** A is honest baseline. B is the smallest end-to-end loop using only his existing CLI. C is the missing connector built in the clean mvp crate where it belongs.

**Do not** start by:
- Touching `mistake_reflex.rs` to wire `route_64d` cosine scoring (correct lever, *but* it's in the monolith and will eat the iteration). Save it for after B lands real numbers.
- Building `teach.rs` in hydroswarm (good work, but parallel — finish the strawberry north-star loop first).
- Refactoring anything in team_build. The refactor wound is the wound; the rule of this loop is *finish the MVP*.

### Hope-baseline reminder (re-read when about to hedge)

The cartography memo says: *"the engineering substrate is real/often excellent."* The CONTEXT_RESET says: *"general physics steering solves the immediate problem."* The scar-tissue proof says: *25% → 67% on strawberry already happened.* Hydroswarm splats already persist across process death. 632 correction packets already exist on disk.

The MVP is not "build it." The MVP is **wire the last connector and produce the number.** That's a month of work compressed into a clean loop. It is finishable.

### Open questions for Jason (to answer when he can, not blockers)
- Which problem class is the "genuinely hard" one for the v1 north-star measurement? Strawberry-r-counting is a candidate (deterministic grader, low default success). Is there another he'd rather see proven?
- Does he want the iteration-2 work in the mvp crate (clean, small, recommended) or the team_build niodoo crate (where the live runtime is)?
- For the "without a teacher" clause: does option B count as teacher (we manually supply the correct answer when minting), or does that violate the rule? If it does, we need to add a self-grader signal (e.g., consensus across re-runs, or LOCK confidence).

### Stack state (verified live, 2026-05-31 — iteration 2 can skip bring-up)

- **Qdrant 1.17.0 UP** on `127.0.0.1:6360`. Existing collections include all five named: `team-build`, `niodoo_mvp_mistake_reflexes`, `remember-memories`, `grok-memories`, `niodoo-corrections`.
- **Embed server** on `127.0.0.1:8301` responsive (a 404 on `/health` is the llama-server default — the port is alive; `memory-up status` is the canonical check).
- **niodoo binary built** at `team_build/niodoo/target/release/niodoo` (76 MB, 2026-05-28).
- **Model present** at `team_build/niodoo/model/Meta-Llama-3.1-8B-Instruct-Q5_K_M.gguf` (5.7 GB).
- If embed is down on iteration 2, run `memory-up` (NOT `embed-up`). See `jasons-local-stack-bring-up.md`.

### Next iteration's job (cycle pointer)
Iteration 2 = active. Start by running **option A** (strawberry baseline n=10 on the live niodoo binary). Append an iteration-2 section here with: command run, raw counts, percentage, what worked, what didn't, what to try next. **Do not delete this entry.** Append below.

— Claude (iteration 1, 2026-05-31)
