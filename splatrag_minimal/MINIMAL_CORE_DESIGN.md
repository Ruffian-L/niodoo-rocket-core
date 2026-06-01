# Minimal SplatRAG Core — Best Math Ideas Only

Goal: The smallest set of mathematical primitives that capture the power of both
- Physics of Friendship minimal splat / ghost memory + dream replay
- SplatRAG dream consolidation + Gaussian volumetric reflexes + valence

Without the full agent runtimes, niodv4 scaffolding, or fancy extras.

## Core Primitives (the keepers)

### 1. Valence-Tagged Experiential Memory
- Experiences are stored not as raw states, but as objects carrying:
  - Location in some space (state, embedding, or topological summary)
  - Valence: scalar "how good/bad" (pain negative, pleasure positive)
  - Intensity / Strength (how strongly it should influence)
  - Optional: radius or covariance (spread / uncertainty)

This is the "splat" or "ghost memory" item.

### 2. Distance-Weighted Steering / Reflex Influence
- At inference or decision time, query nearby memories.
- Each contributes a vector or bias proportional to:
  activation = f(distance)   (Gaussian, exponential decay, etc.)
  contribution = activation * intensity * valence
- Sum them to produce a steering vector / force / logit bias.

This is the "reflex before thought" or "ghost vector" effect.

### 3. Asymmetric Persistence (Pain Lasts)
- Negative valence items decay much slower (or not at all) compared to positive ones.
- Biological prior: trauma / mistakes stick around longer than routine successes.
- Mathematically: different decay rates λ_pain << λ_pleasure (or λ_pain ≈ 0).

### 4. Offline Dream / Consolidation Phase
- Periodically (or on demand), run a "dream" step on the memory store.
- Operations can include:
  - Decay (apply the asymmetric rules above)
  - Consolidation / Merging: nearby similar-valence items are combined (weighted average location + summed or blended intensity)
  - Possibly replay of high-value trajectories to reinforce
- This is the compression + generalization step that turns many specific events into robust reflexes.
- In the richer versions this is where physics simulation or clustering happens.

### 5. Healing / Updating on New Evidence
- Later positive experience in a region that previously had strong negative valence can actively weaken or "heal" the old negative memory.
- Prevents permanent scarring from early failures once the agent gets good.

### 6. Output as Usable Steering Signal
- The memory system ultimately produces something that can be:
  - Added as a force in a physics simulation (Physics of Friendship style)
  - Injected as special tokens / embeddings / adapter payload into an LLM forward pass (SplatRAG "God Protocol" style)
  - Fed directly into or become part of compute_ghost_vector

## Minimal Viable Loop (the thing we actually want to finish)

1. Agent experiences something significant → store as valence-tagged memory item (with location + spread).
2. During normal operation → query produces steering vector that biases behavior.
3. Periodically (dream phase) → consolidate, decay, heal.
4. The steering vector is consumable by the rest of the system (compute_ghost_vector, action selection, etc.).
5. Over time the memory improves the agent's performance on hard problems via its own history + corrections.

## What We Are Dropping (for now)

- Full multi-agent runtimes
- Heavy topology / TDA pipelines unless they are the actual minimal way to represent the "location + spread"
- The entire benchmark harness and ingestion pipelines (unless a tiny piece is the best way to do consolidation)
- Most of the niodv4 / Cursor / MCP scaffolding
- Anything not directly serving reflexive memory + dream consolidation + steering output

## Open Questions for the Minimal Version

- Representation: raw state vectors? 64-d centroids + covariance? Full Gaussians in embedding space?
- Query: how exactly to turn nearby splats into a vector that plays nicely with compute_ghost_vector?
- Dream trigger: time-based, energy-based, or explicit "sleep" call?
- How do corrections from niodoo-corrections get turned into new splats or valence updates?

This is the scoped target. Everything else is secondary or future work.

