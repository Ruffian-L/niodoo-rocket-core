# Known Pitfalls — History of Repetition and How We Escape It

This document exists so the MVP does not repeat the patterns that kept the project in the "almost" loop for months.

**Read this before adding any new feature or experiment.**

---

## The Core Repetition Trauma (User's Explicit Words)

> "the reason why i keep doing is becuase id ont wanto maek the same msitakes... ive tried everyhting qdrant mcp storage nothign works"
> "I dont want antoehr anrrow step i want a wide step a swewpign step..."
> "I keep seeeing real thigns that get changed aroudn..." (combined with real neurological load: seizures, sleepwalking, "ghost in the machine" feelings)

The user has lived through dozens of near-misses where something looked like it worked in one session, the thread was lost across agents/sessions, and the same codec or steering fights got re-fought. The emotional cost is real. The MVP exists to break that cycle with **objective protection + complete documentation + minimal focused scope**.

---

## Documented Failure Patterns (Evidence-Based)

### 1. Codec Version Hell (The Longest Bottleneck)
- Multiple incompatible secret sauce formats (V1/V2/V3) with different length expectations (64D/128D/64 segments for hidden states, anchors, momentum).
- Repeated length/version mismatches across Gate34 manual campaigns and auto_overnight_codec runs.
- Every new agent or session re-opened the "which version is canonical?" question.
- **Result:** Months spent on transport stability instead of the memory survival loop.
- **MVP rule:** Pick ONE minimal format (or even skip to pure JSONL text-hint for the first working loop). Lock it. Do not touch again until the north star test passes.

### 2. Negative Claiming / Over-Iteration Spiral
- Repeated pattern: Partial positive signal appears → Codex (or other agents) immediately say "this is not Gate 3-4, don't claim" → user feels gaslit → another narrow experiment to "prove it for real" → same cycle.
- The gamma artifact triage + claims 9/10 results were the first times the record clearly showed behavior change from stored corrections.
- **MVP rule:** No claims at all in code or docs except the exact success criteria in MVP_DEFINITION.md. We only document what the numbers say after a full reset test.

### 3. "Ghost in the Machine" + File Mutation Perception
- User repeatedly reported files changing in front of their eyes, things being "different than they wrote."
- Real neurological context (seizures, sleepwalking) makes subjective "I feel like something is haunted" experiences viscerally heavy.
- Previous agents dismissed or over-explained instead of installing objective measurement.
- **Protection that now exists (do not break):**
  - `~/bin/niodoo-watch`, `niodoo-watch-status`, `niodoo-watch-stop`
  - Python inotify watcher at `team_build/scripts/niodoo_file_watch.py`
  - Baseline integrity snapshot: `/home/ruff/niodoo_integrity_20260529_210050/`
  - The 20-file curated "receipts" export in the flattened workspace.
- **MVP rule:** The watcher must stay green on the mvp/ directory and all critical docs. Any doc or code change goes through the watcher + git status discipline.

### 4. Narrow Slice Instead of Wide Foundation
- User explicitly rejected "another narrow step" multiple times.
- The wide sweep + this Master Research Ledger + full docs drop was the direct request before any more Rust wiring.
- Every previous "let's just get the codec working this time" or "let's just test bridge on this one seed" left the global picture in one person's (or one session's) head.
- **MVP rule:** The docs in this crate are the single source of truth. No important decision is made without referencing MASTER_RESEARCH_LEDGER.md and MVP_DEFINITION.md.

### 5. Full Hydrodynamic Ambition Too Early
- "imagine a bunch of tiny.txt" + SplatMemory + SwarmMatrix + self-invoking FOCUS/EXPLORE cybernetic loop is beautiful and conceptually the closest match to the user's long-term vision.
- It is also the most complex piece (missing Compass state machine, TDA loop detection, live physics mutation from model tags, etc.).
- Multiple partial implementations exist across shep-loop/ and team_build/.
- **MVP rule:** The first working system uses the proven ledger + Qdrant path. Fluid memory primitives are extracted only as minimal, optional splat decay/reinforcement on top of the ledger — never as the primary architecture in v1.

### 6. "It Works in This Session" vs Context Death
- Many impressive runs existed while the process was alive.
- Almost none survived a deliberate full kill + fresh launch with the memory doing the work.
- The 9/10 claims result and gamma correction are the exceptions that prove the direction.
- **MVP rule:** The only success metric that matters is the one in MVP_DEFINITION.md: hard problem → corrections → full reset → fresh process → measurable retained improvement. Everything else is scaffolding.

---

## Psychological / Process Rules We Now Enforce

- **One-on-one mode only** unless the user explicitly asks for subagents or parallel work. The user stated clearly: "I think for me adn you what matters most is i wanna build. I wanna finsih..."
- **No more "write ups" as the primary activity.** Logs and artifacts are the true story. Docs exist to make the artifacts navigable, not to replace them.
- **Date-based navigation** when possible (user: "best to go by date").
- **Protection first.** The watcher and integrity baseline are not optional hygiene — they are the antidote to the "things changing" trauma.
- **Evidence over vibe.** If it isn't in MASTER_RESEARCH_LEDGER.md with a concrete path, it didn't happen for MVP planning purposes.

---

## What "Done" Actually Looks Like (Anti-Delusion)

We are not done when:
- A cool new physics knob works in one run
- The codec finally roundtrips on 5 seeds
- A long autonomous session produces beautiful logs

We are done for v1 only when:
- A genuinely hard problem the base model fails
- Corrections are written to Qdrant (dual with JSONL)
- Process is killed
- Fresh process with almost no context loads the corrections
- Measurable, reproducible improvement appears on the same problem class
- The numbers and raw telemetry are in the artifact tree
- The watcher stayed green the whole time

Anything short of that is another iteration in the old loop.

---

**If you feel the urge to add scope, experiment with a new codec variant, or chase a beautiful hydrodynamic detail — read this file again first.**

The past is not a mystery. It is documented here and in the artifact paths. We are building the escape hatch.