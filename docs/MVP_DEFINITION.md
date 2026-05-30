# MVP Definition — Exact Success Criteria (No Ambiguity)

This is the **only** definition of done for the first version of this crate. Everything else is scaffolding or future work.

**North Star Test (verbatim from the user's repeated formulation):**

> Hit a genuinely hard/impossible problem → corrections applied → turn the system off (full context death) → fresh process with little/no prior context → the model demonstrably performs better because it *actually internalized* something.

When this loop is real, observable, and produces numbers on a hard problem, the MVP is complete. Not before.

---

## Mandatory Success Criteria (All Must Be True)

1. **A genuinely hard problem exists** for the base model (8B or whatever is used in the harness) that it consistently fails at without the memory system. This must be documented with raw failing runs.

2. **Corrections are applied** during the run (from user, stronger model, or self-generated via tags) and are turned into mistake-reflex events.

3. **Durable dual-write occurs**:
   - JSONL file written as human-readable, git-trackable audit trail (source of truth).
   - Qdrant collection receives the same events (feature-gated `--features qdrant`).
   - The events are queryable by the runtime on subsequent launches.

4. **Full context death**:
   - The process is deliberately killed (not graceful shutdown with warm state).
   - No prior conversation tokens or in-memory session state survive.
   - A fresh invocation starts with only a minimal bootstrap (no hand-crafted "remember this" prompt that leaks the answer).

5. **Fresh process loads the memory**:
   - The new process starts with almost no context.
   - It loads the stored reflexes (via `--mistake-reflex-path` or automatic startup load in later phases).
   - The reflexes are actually active (influence mode or equivalent changes behavior).

6. **Measurable, reproducible retained improvement** on the same problem class:
   - Clear difference in success rate, error patterns avoided, or specific judgment quality.
   - Raw telemetry, stdout, and artifact files captured for both the failing baseline and the post-reset improved runs.
   - The improvement is attributable to the loaded corrections (not to longer thinking, different seed luck, or prompt engineering).

7. **Objective protection remained green** throughout:
   - File watcher logged no unexplained mutations on the mvp tree or critical artifacts.
   - Git state + integrity baseline captured at start and end.

---

## What Counts as "Evidence" (Strict)

- Raw `telemetry.jsonl` + `stdout.txt` + `stderr.txt` from both sides of the reset.
- The exact ledger JSONL that was written and later loaded.
- Qdrant point samples or query results showing the correction events existed before the kill.
- A dated artifact directory with a one-page `PROOF.md` or `NORTH_STAR_RUN.md` that a third party can follow to reproduce the core claim.
- The watcher log and git diff showing the run happened under protection.

Vibes, internal monitors firing, or "it felt different this time" do not count.

---

## Explicit Out of Scope for v1 (Do Not Ship These as Requirements)

- Full hydrodynamic / fluid active memory swarm as the primary system (the beautiful vision in `shep-loop/hydrodynamic-swarm/imagine a bunch of tiny.txt` is Phase 4+).
- Multiple secret sauce codec versions or a perfect 256× niodv4 transport (one locked minimal roundtrip at most; gamma/claims wins happened without it).
- Self-invoking cybernetic loop where the model emits `<FOCUS>`/`<EXPLORE>` that mutates live physics (aspirational and historically destabilizing when attempted too early).
- General capability claims or comparisons on MMLU/GPQA/MATH-style benchmarks.
- Heavy GPU topology, lophat, massive swarms, or production-grade reliability beyond "the loop closes once with numbers."

---

## Anti-Goals (These Are How We Got Stuck Before)

- Another giant research monolith (this crate must stay small enough that one human + one AI can hold the whole picture).
- Overclaiming based on partial results or in-session behavior only.
- Repeating any historical experiment that already exists in the artifact tree (use the 304-event ledger, gamma runs, and Qdrant history instead of re-collecting).
- Scope creep justified by "this is cool" or "this will make the next phase easier" before Phase 2 numbers exist.

---

## Definition of v1 Complete

We can run the full north star sequence on at least one hard problem class, produce the numbers, capture the artifacts under protection, and point to them with a clear, non-hyped writeup.

At that moment the MVP is done. The rest of the vision becomes engineering on a working foundation instead of another search for the missing piece.

**Everything in this crate exists to make that moment arrive as quickly and cleanly as possible.**
