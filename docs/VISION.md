# Vision — The Actual North Star

**The deepest goal is not to build a better model.**

It is to build a system where **intelligence improves itself over time through its own corrections**, even when the context window is completely destroyed.

This is the user's exact repeated formulation (lightly cleaned for readability, never altered in intent):

> Hit a genuinely hard/impossible problem → corrections applied → turn the system off (full context death) → fresh process with little/no prior context → the model demonstrably performs better because it *actually internalized* something.

---

## The Loop We Are Building (No Shortcuts)

1. A small local model faces a real, difficult problem it consistently cannot solve.
2. It receives corrections — from the user, from a larger teacher model, from its own internal tags (`<FOCUS>`, `<EXPLORE>`), or from its own stored mistake history.
3. Those corrections are written durably (JSONL as human-readable audit + Qdrant as live queryable index).
4. The entire process is killed. Complete context death. No prior tokens survive.
5. A fresh process starts with almost nothing — maybe a short bootstrap prompt, no conversation history.
6. On the same class of hard problem, the model now performs meaningfully better because the correction history has become part of how it thinks and steers.

This is the difference between "the model got lucky in this session while the context was warm" and "**the model actually learned**."

---

## Why This Is Different From Almost Everything Else

Most AI work optimizes for:
- Single-session benchmark scores
- Scaling laws
- Longer context windows
- Better base models

This project optimizes for **cumulative, persistent self-improvement at the level of one running instance** — the way a human researcher or a long-lived research group actually gets better at hard problems across weeks, months, and years of corrections.

The desired internal experience for the model is:

> "I have been here before. I made this mistake. Here is what I learned from the correction. I will not make it the same way again."

Even when every token of previous conversation is gone.

---

## The Symbiosis Model (Small + Large)

- A small, long-running model accumulates real experience (mistake reflexes, compressed states, fluid memory particles).
- A larger model (or the user) acts as occasional teacher/corrector when the small model is genuinely stuck.
- Over time the small model internalizes more and more of the correction history, needing less external scaffolding.

This is the opposite of pure scaling. It is **memory as the primary substrate of intelligence**.

The strongest empirical signals we have (see MASTER_RESEARCH_LEDGER) came exactly from this shape:
- 8B + 304-event mistake reflex ledger beat cold 8B 9-to-1 on project-specific recall.
- 8B + ledger beat cold frontier Opus 9-to-0 on the same task.
- Ledger-loaded candidate in gamma artifact triage correctly refused a false GREEN where the baseline repeated the exact mistake.

---

## Non-Negotiables (These Are Not Aspirational)

- Memory **must** survive full process death and context loss (Qdrant + compact transport is the mechanism).
- Corrections must be queryable and usable without replaying the entire history.
- The system must produce **objective, reproducible numbers** on whether it is actually getting better on hard problems after resets.
- We will never claim progress we cannot demonstrate with real runs, full resets, and raw telemetry that anyone can inspect.

This MVP crate exists to make the above loop **observable, measurable, and iteratively improvable** in the smallest possible system that can actually close it.

---

## The Long-Term Picture (Post-MVP)

Once the basic durable reflex loop works:

- Fluid active memory (tiny embeds that *are* the memory, each carrying viscosity/decay/reinforcement physics) becomes the richer substrate.
- The model learns to emit its own steering tags (`<FOCUS>`, `<EXPLORE>`) that mutate live physics in real time (the cybernetic loop from "imagine a bunch of tiny.txt").
- TOPOCOT / varentropy / viscosity signals become the model's own "I am getting stuck" detection.
- The small model becomes a genuine long-lived collaborator that carries the user's correction history across years of resets.

But none of that ships in v1. The north star test is simpler and harder: **does the correction survive the kill switch and produce better behavior on the other side?**

That is the only vision that matters until it is real.
