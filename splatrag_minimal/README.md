# splatrag_minimal — Minimal SplatRAG Reflective Memory Core (inside mvp)

This directory holds the smallest possible reference implementation of the core mathematical primitives that power reflective / evolvable ghost memory + dream consolidation.

It lives here (inside the mvp crate) because this is the clean, durable, git-tracked home the user chose for the North Star work that was already underway (ledgers, RepetitionContext reflex, corrections ingestion, etc.).

## What it actually is
- Signed Gaussian splats (center + valence + intensity + radius)
- Distance-weighted steering vector output: `v(x) = Σ α·I·k(x,s)·(μ_s - x)`
- Asymmetric persistence (pain lasts, pleasure decays)
- Active healing on success in previously negative regions
- Explicit offline `dream_consolidate()` step (decay + same-valence merging)
- Minimal "dream replay" style reinforcement of successful trajectories

All distilled from the two strongest lineages the user owns:
- Physics of Friendship (MountainCar) minimal splat + ghost + compute_ghost + dream replay
- SplatRAG (Gaussian volumetric reflexes + dream consolidation daemon + signed valence)

## The inversion / evolvability test
`core/inversion_test.py` is the first concrete proof artifact.

It generates "good route" vs "bad route" experiences, lets the system consolidate, then checks that the resulting steering vectors are opposites (cosine near -1).

When it passes (especially the explicit sign-flip case producing -1.0), it demonstrates the memory system can deterministically steer toward either pole. That symmetry is the minimal evidence that internal reflection can produce real self-improvement on hard problems without constant external "right answer" teaching.

## How it connects to the rest of the mvp / Niodoo stack
- The `compute_steering_vector()` method is deliberately shaped to be consumable by (or become the memory-derived part of) the user's existing `compute_ghost_vector` machinery (force addition, ghost basins, adapter injection of centroid+covariance, etc.).
- Later wiring work will take the output of this core and feed the real Ghost Vector path at inference time, using the signal-based repetition detector in `src/reflex/repetition.rs` as the trigger.
- Corrections stored in `ledgers/mvp_corrections.jsonl` + `niodoo-corrections` Qdrant collection can eventually be turned into new valence-tagged splats.

## Scope (brutally enforced)
- No full agent runtimes
- No heavy TDA / topology unless it is the minimal representation of "location + spread"
- No hoarding the 49 MB semantic backup or every old artifact
- Only the math that lets us run the inversion test and prove the dream reflection loop is real

## Running the test
```bash
cd mvp/splatrag_minimal/core
python3 inversion_test.py
```

Expected: PASS with at least one of the cosines strongly negative (sign-flip case is usually -1.0000).

## Files
- `MINIMAL_CORE_DESIGN.md` — the unified primitives
- `EVOLVABILITY_VIA_INVERSION.md` — the test philosophy and scope lock
- `core/minimal_splat_core.py` — the <200 LOC reference implementation
- `core/inversion_test.py` — the runnable proof

This is the "smallest best part" the user asked for, now kept together with the rest of the durable mvp work so it survives resets and lives in one tracked place.

— 2026-06-01 (moved here per explicit user direction)
