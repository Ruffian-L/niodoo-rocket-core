# Evolvability via Vector Inversion — Minimal SplatRAG Core

**Date:** 2026-06-01  
**Status:** Working document for the minimal viable core

## The Core Insight (User's Formulation)

If we can define "good route" vs "bad route" (or any clean opposite concepts), and the system produces vectors that are the same but **inverted**, and it can steer **equally and deterministically** toward either pole, then we have evidence of a truly reflective, evolvable system rather than one-sided memorization or bias.

This is the test that would make the whole theory feel real and worth finishing.

Related concepts from the history (worbglob vector autopsy, ontological inversion, negative gain experiments) were already exploring sign-flips and anti-vectors. The goal now is to make this property **first-class and provable** in a minimal implementation.

## Two Lineages — Best Mathematical Primitives Only

### 1. Physics of Friendship (Minimal Splat / Ghost Memory Lineage)
- Ghost memory as a buffer/deque of high-value past states or trajectories.
- `compute_ghost(state)` (or equivalent) that produces a steering vector/force from similar past experiences.
- Dream replay: offline sampling of successful trajectories to populate or reinforce the memory.
- The output is directly added as a force or bias into the dynamics / action selection.
- Philosophy: Keep it physics-native and force-based. The memory *is* the steering.

**Strongest minimal ideas here:**
- Memory items as attractors/repulsors.
- Direct production of a vector that can be injected into whatever the policy or forward pass is.
- Dream as replay/population rather than complex merging.

### 2. SplatRAG (Dream Consolidation / Gaussian Volumetric Reflex Lineage)
- Experiences stored as explicit Gaussian splats with:
  - Center (location in state/embedding space)
  - Valence (signed scalar: pain negative, pleasure positive)
  - Intensity (strength, decays asymmetrically)
  - Radius / spread
- Query produces per-action or directional bias: `activation * intensity * valence * confidence` (Gaussian falloff).
- **Dream consolidation**: Offline phase that merges nearby similar splats (weighted average center + summed/blended intensity). This is the compression + generalization step.
- Asymmetric decay: Pain memories persist much longer than pleasure.
- Active healing: Later success in a previously painful region can actively weaken old negative splats.
- Output is a steering vector that can be turned into injectable payloads (e.g., centroid + covariance projected via adapter, as seen in the bench's splat_engine "God Protocol").

**Strongest minimal ideas here:**
- Signed valence as first-class.
- Gaussian (or smooth distance kernel) for smooth, distance-weighted influence.
- Explicit consolidation step during dream/offline periods.
- The idea that memory items have geometry (not just points) that can be projected/injected.

## The Minimal Viable Core (Synthesis)

We keep only the primitives that directly enable the inversion test + clean steering:

1. **Valence-tagged memory items** (with location + optional spread/geometry).
2. **Symmetric distance-weighted activation** that produces a signed steering vector (positive valence → one direction, negative → opposite).
3. **Offline dream/consolidation** phase (merging + any lightweight physics or replay that strengthens the memories without destroying sign information).
4. **Asymmetric persistence** (pain lasts) as a simple prior.
5. **Output as a vector** that can be consumed by `compute_ghost_vector` (or become it) — whether as a force, an embedding payload, or a bias.

The inversion test then becomes:
- Store or replay "good route" memories with positive valence.
- Store or replay "bad route" memories with negative valence (or inverted geometry).
- Query with a probe that should favor one or the other.
- Check that the resulting steering vectors are approximately negatives of each other.
- Check that the system can deterministically steer toward either pole when the memories are flipped.

If this holds cleanly (especially after consolidation), we have a system whose memory actually encodes opposition in a steerable, evolvable way.

## Relation to `compute_ghost_vector`

In the user's systems:
- `compute_ghost_vector` is the function that turns memory (V_Memory, ghost basins/registry, splats, etc.) into the actual tensor that gets injected or added during inference / action selection.
- In the mountaincar lineage it was more force-based.
- In the Niodoo/Rust + bench lineage it involves projecting memory geometry (centroid + covariance, splat payloads) via adapters into the hidden state.

The minimal core above is designed so its output is a clean vector that can be fed into whatever version of `compute_ghost_vector` the user wants to use. The inversion property should be visible at the output of `compute_ghost_vector` if the memory feeding it has the right structure.

## Scope Rules (To Prevent Drift)

- We only keep math that directly supports signed valence + symmetric activation + consolidation + vector output.
- No full agent loops, no heavy TDA pipelines unless they are the minimal representation of "location + spread", no entire benchmark harness unless a tiny piece is the cleanest way to test inversion.
- The first deliverable is a tiny reference implementation + a test that demonstrates the inversion property on synthetic or mountaincar-style "good route vs bad route" data.
- Everything else (full integration with the current Niodoo runtime, large-scale runs, etc.) is explicitly future work.

## Current Assets We Can Use Immediately

- Clean extracted `SPLATRAG_FULL_CODEBASE.md` (the real code, not the semantic log).
- Mountaincar `splat_memory.py` (Gaussian + consolidation + healing).
- SplatRagBench pieces (dream consolidation daemon, splat injection via adapter).
- The semantic backup ingestion still running in background (history + context, can be queried later).

## Proposed Tiny First Deliverable

A single small Python module (or notebook) that:
- Implements the minimal core above.
- Has a synthetic "good route / bad route" generator.
- Runs the inversion test and reports how clean the opposite vectors are, and how deterministically the system steers when memories are flipped.
- Can later be pointed at real data from the mountaincar experiments or the user's corrections.

This is small enough to finish in days/weeks instead of months, yet if it works it proves the core theory is sound and gives a foundation to grow from without the previous scope explosions.

---

This document exists so we don't drift. Any new idea must justify itself against the inversion/evolvability test and the minimal primitives above.

Next step: User to confirm or adjust the scope, then we start extracting the exact math and writing the tiny reference + test.
