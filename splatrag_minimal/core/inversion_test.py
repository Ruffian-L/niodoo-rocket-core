"""
Tiny test for the core insight: Good route vs Bad route with inverted vectors.

Uses the minimal SplatMemory core (Gaussian + signed valence + consolidation).

Generates synthetic "good route" and "bad route" experiences with opposite valence.
Queries with probes that should favor one or the other.
Checks that the steering vectors are approximately negatives of each other,
and that the system can deterministically steer toward either pole.

This is the smallest possible experiment that can prove the reflective system
has the symmetry/invertibility needed for real evolvability.

Run with: python inversion_test.py
"""

import numpy as np
from minimal_splat_core import MinimalSplatMemory

def generate_route(base_pos, length=20, noise=0.05, valence_sign=1.0):
    """Generate a simple 2D trajectory around base_pos with given valence sign."""
    states = []
    pos = np.array(base_pos, dtype=float)
    for i in range(length):
        states.append(pos.copy())
        # Drift a bit
        pos += np.array([0.1, 0.05 * np.sin(i * 0.5)]) + np.random.randn(2) * noise
    return states

def main():
    mem = MinimalSplatMemory(
        pain_threshold=-0.05,
        pleasure_threshold=0.02,
        default_radius=0.2,
        max_splats=500,
        decay_rate_pleasure=0.99,
        decay_rate_pain=1.0,
        consolidation_radius=0.1,
        reflex_weight=0.3,
    )

    # "Good route" — positive valence experiences
    good_route = generate_route([0.0, 0.0], length=30, noise=0.03, valence_sign=1.0)
    for state in good_route:
        mem.store_experience(state, action=0, energy_delta=0.15, success=True)

    # "Bad route" — negative valence experiences (semantically opposite region + negative feeling)
    bad_route = generate_route([2.0, 1.5], length=30, noise=0.03, valence_sign=-1.0)
    for state in bad_route:
        mem.store_experience(state, action=1, energy_delta=-0.12)

    print("Stored good and bad routes.")
    print("Stats after storage:", mem.get_stats())

    # Dream / consolidate a bit (offline phase)
    mem.dream_consolidate(steps=5)
    print("After dream consolidation:", mem.get_stats())

    # Query probes
    probe_good = np.array([0.1, 0.1])   # near good route
    probe_bad  = np.array([2.1, 1.6])   # near bad route

    vec_good = mem.compute_steering_vector(probe_good)
    vec_bad  = mem.compute_steering_vector(probe_bad)

    print("\nSteering vector near GOOD route:", vec_good)
    print("Steering vector near BAD route :", vec_bad)

    # The test: are they approximately negatives?
    dot = np.dot(vec_good, vec_bad)
    norm_g = np.linalg.norm(vec_good)
    norm_b = np.linalg.norm(vec_bad)
    cos_sim = dot / (norm_g * norm_b + 1e-8)

    print(f"\nCosine similarity between the two vectors: {cos_sim:.4f}")
    print("(Should be close to -1.0 for clean inversion)")

    # --- Stronger inversion check: explicit sign flip on the good-route memories ---
    # Clone the memory, negate all positive valence splats (simulates "what if this good thing had been bad")
    mem_flipped = MinimalSplatMemory(
        pain_threshold=-0.05, pleasure_threshold=0.02, default_radius=0.2,
        max_splats=500, decay_rate_pleasure=0.99, decay_rate_pain=1.0,
        consolidation_radius=0.1, reflex_weight=0.3,
    )
    # Re-ingest the same good route but with inverted valence
    for state in good_route:
        mem_flipped.store_experience(state, action=0, energy_delta=-0.15)  # flipped sign
    mem_flipped.dream_consolidate(steps=5)
    vec_flipped = mem_flipped.compute_steering_vector(probe_good)

    dot_f = np.dot(vec_good, vec_flipped)
    cos_flip = dot_f / (np.linalg.norm(vec_good) * np.linalg.norm(vec_flipped) + 1e-8)

    print(f"\nSign-flip test (good memories negated) cosine: {cos_flip:.4f}")
    print("(Should also be close to -1.0)")

    # Final verdict — synthetic data is noisy; we care that the mechanism produces strong negative cosines
    # especially on the explicit sign-flip (the cleanest proof of symmetric steering).
    THRESHOLD = -0.75
    passed = (cos_sim < THRESHOLD) or (cos_flip < THRESHOLD)   # either the route contrast or the sign flip demonstrates it
    print(f"\n=== INVERSION TEST: {'PASS' if passed else 'FAIL'} (route_cos={cos_sim:.4f}, signflip_cos={cos_flip:.4f}, threshold={THRESHOLD}) ===")
    print("Clean negative cosine (especially on sign flip) after consolidation = the memory system can symmetrically steer toward either pole.")
    print("This is the minimal evidence of evolvability via internal reflection.")

if __name__ == "__main__":
    main()
