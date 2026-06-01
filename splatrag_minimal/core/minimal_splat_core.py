"""
Minimal SplatRAG Core - Best Math Ideas Only

Synthesizes the strongest primitives from:
- Physics of Friendship minimal splat / ghost memory + dream replay
- SplatRAG Gaussian volumetric reflexes + dream consolidation + valence

Goal: Small, clean, actually finishable module that can produce
a steering vector suitable for compute_ghost_vector or direct force injection.

No full agent runtime. No heavy scaffolding. Just the math.
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Optional, Tuple
from collections import deque
import time

@dataclass
class Splat:
    """Core memory unit: a valence-tagged Gaussian in some state space."""
    center: np.ndarray          # Location in state/embedding space
    valence: float              # Negative = pain/avoid, positive = pleasure/attract
    intensity: float            # How strong the memory is (decays, reinforced)
    radius: float = 0.15        # Influence spread
    action: Optional[int] = None  # Optional: which action was taken
    hits: int = 0
    reinforcements: int = 1

    def is_alive(self, min_intensity: float = 0.01) -> bool:
        return self.intensity > min_intensity

class MinimalSplatMemory:
    """
    Minimal reflexive memory with dream consolidation.

    Stores experiences as valence-tagged Gaussians.
    Produces steering vectors via distance-weighted valence.
    Has an explicit offline "dream" step for consolidation + asymmetric decay.

    This is the smallest version that still captures the dream reflection magic.
    """

    def __init__(
        self,
        pain_threshold: float = -0.1,
        pleasure_threshold: float = 0.03,
        default_radius: float = 0.15,
        max_splats: int = 2000,
        decay_rate_pleasure: float = 0.998,
        decay_rate_pain: float = 1.0,           # Pain is basically immortal
        consolidation_radius: float = 0.08,
        reflex_weight: float = 0.1,
    ):
        self.pain_threshold = pain_threshold
        self.pleasure_threshold = pleasure_threshold
        self.default_radius = default_radius
        self.max_splats = max_splats
        self.decay_rate_pleasure = decay_rate_pleasure
        self.decay_rate_pain = decay_rate_pain
        self.consolidation_radius = consolidation_radius
        self.reflex_weight = reflex_weight

        self.splats: List[Splat] = []
        self.current_episode = 0

        # For minimal "dream replay" style (Physics of Friendship style)
        self.successful_trajectories: deque = deque(maxlen=500)

    # ------------------------------------------------------------------
    # STORAGE (with valence thresholding + reinforcement + healing)
    # ------------------------------------------------------------------
    def store_experience(
        self,
        state: np.ndarray,
        action: Optional[int],
        energy_delta: float,
        success: bool = False,
    ):
        """Store only significant experiences as splats."""
        if energy_delta < self.pain_threshold:
            valence = energy_delta * 2.0
            intensity = abs(energy_delta) * 3.0
        elif energy_delta > self.pleasure_threshold or success:
            valence = max(energy_delta * 3.0, 0.5 if success else 0.0)
            intensity = 8.0 if success else energy_delta * 5.0
        else:
            return  # Unremarkable — ignore

        # Active healing + reinforcement (core SplatRAG idea)
        for splat in self.splats:
            if np.linalg.norm(state - splat.center) < splat.radius * 1.5:
                if valence > 0 and splat.valence < 0:
                    # Healing: success in previously painful region
                    splat.intensity *= 0.1
                    if splat.intensity < 0.01:
                        splat.intensity = 0
                    return
                if splat.action == action:
                    # Reinforcement (compound trauma or pleasure)
                    splat.intensity = min(30.0, splat.intensity + intensity * 0.5)
                    splat.valence = splat.valence * 0.7 + valence * 0.3
                    splat.reinforcements += 1
                    return

        # New splat
        splat = Splat(
            center=state.copy(),
            action=action,
            valence=valence,
            intensity=min(10.0, intensity),
            radius=self.default_radius,
        )
        self.splats.append(splat)

        if len(self.splats) > self.max_splats:
            self.splats.sort(key=lambda s: s.intensity)
            self.splats = self.splats[-self.max_splats :]

    # ------------------------------------------------------------------
    # QUERY → Steering Vector (the actual output we care about)
    # ------------------------------------------------------------------
    def compute_steering_vector(
        self, state: np.ndarray, output_dim: Optional[int] = None
    ) -> np.ndarray:
        """
        The key function that should eventually feed compute_ghost_vector.

        Returns a vector in the same space as `state` (or projected if output_dim given)
        that represents the net reflexive pull from memory.
        Positive valence → attract toward that region.
        Negative valence → repel.
        """
        if not self.splats:
            return np.zeros(output_dim or len(state))

        state_2d = state[:2] if len(state) > 2 else state
        steering = np.zeros(2)

        for splat in self.splats:
            if not splat.is_alive():
                continue
            dist = np.linalg.norm(state_2d - splat.center)
            if dist > splat.radius * 3:
                continue

            activation = np.exp(-0.5 * (dist / splat.radius) ** 2)
            signal = activation * splat.intensity * splat.valence
            direction = (splat.center - state_2d) / (dist + 1e-8)
            steering += signal * direction

        if output_dim and output_dim != 2:
            # Simple projection or padding if needed by the caller
            if output_dim > 2:
                steering = np.pad(steering, (0, output_dim - 2))
            else:
                steering = steering[:output_dim]

        return steering * self.reflex_weight

    # ------------------------------------------------------------------
    # DREAM / CONSOLIDATION (the "reflection" step)
    # ------------------------------------------------------------------
    def dream_consolidate(self, steps: int = 1):
        """
        The offline dream phase.
        Combines:
        - Asymmetric decay (pain persists, pleasure fades)
        - Consolidation / merging of nearby similar splats
        """
        self.current_episode += steps

        # Asymmetric decay
        for splat in self.splats:
            rate = self.decay_rate_pain if splat.valence < 0 else self.decay_rate_pleasure
            splat.intensity *= rate ** steps

        # Remove dead
        self.splats = [s for s in self.splats if s.is_alive()]

        # Consolidation (the key SplatRAG addition)
        if len(self.splats) > 50:
            self._merge_similar_splats()

    def _merge_similar_splats(self):
        """Minimal consolidation: merge nearby same-valence splats."""
        merged = set()
        new_splats = []

        for i, s1 in enumerate(self.splats):
            if i in merged:
                continue
            group = [s1]
            for j, s2 in enumerate(self.splats):
                if j <= i or j in merged:
                    continue
                if np.sign(s1.valence) != np.sign(s2.valence):
                    continue
                if np.linalg.norm(s1.center - s2.center) < self.consolidation_radius:
                    group.append(s2)
                    merged.add(j)

            if len(group) > 1:
                total_int = sum(s.intensity for s in group)
                new_center = sum(s.center * s.intensity for s in group) / total_int
                new_valence = sum(s.valence * s.intensity for s in group) / total_int
                consolidated = Splat(
                    center=new_center,
                    valence=new_valence,
                    intensity=min(20.0, total_int * 0.85),
                    radius=max(s.radius for s in group) * 1.05,
                    reinforcements=sum(s.reinforcements for s in group),
                )
                new_splats.append(consolidated)
            else:
                new_splats.append(s1)

        self.splats = new_splats

    # ------------------------------------------------------------------
    # Minimal "Dream Replay" style population (Physics of Friendship style)
    # ------------------------------------------------------------------
    def record_successful_trajectory(self, states: List[np.ndarray]):
        """Call this on good trajectories so dream can replay them."""
        self.successful_trajectories.append(list(states))

    def replay_dream(self, n_samples: int = 10):
        """Simple replay to reinforce good memories (minimal version of dream replay)."""
        if not self.successful_trajectories:
            return
        for _ in range(n_samples):
            traj = np.random.choice(list(self.successful_trajectories))
            for state in traj:
                # Re-store as strong positive (simplified)
                self.store_experience(state, action=None, energy_delta=0.2, success=True)

    def get_stats(self):
        if not self.splats:
            return {"count": 0}
        valences = [s.valence for s in self.splats]
        return {
            "count": len(self.splats),
            "pain": sum(1 for v in valences if v < 0),
            "pleasure": sum(1 for v in valences if v > 0),
            "mean_intensity": float(np.mean([s.intensity for s in self.splats])),
        }


# ------------------------------------------------------------------
# Tiny usage example
# ------------------------------------------------------------------
if __name__ == "__main__":
    mem = MinimalSplatMemory()

    # Simulate some experiences
    for i in range(200):
        state = np.random.randn(2) * 0.5
        delta = np.random.randn() * 0.2
        mem.store_experience(state, action=i % 3, energy_delta=delta, success=(delta > 0.15))

    # Dream a bit
    mem.dream_consolidate(steps=10)

    # Get steering
    query_state = np.array([0.1, 0.2])
    vec = mem.compute_steering_vector(query_state)
    print("Steering vector:", vec)
    print("Stats:", mem.get_stats())
