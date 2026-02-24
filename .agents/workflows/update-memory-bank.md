---
description: Update the project's Memory Bank documentation to reflect recent progress and decisions.
---

# Update Memory Bank Process

Follow this routine to ensure the project documentation (`memory-bank/`) remains the single source of truth.

## 1. Analyze Current State
1. Read `memory-bank/activeContext.md` to understand the *previous* focus.
2. Read `memory-bank/progress.md` to see the roadmap status.
3. Read `memory-bank/systemPatterns.md` and `memory-bank/techContext.md` if architectural or dependency changes occurred.

## 2. Update Progress (`progress.md`)
- Mark completed tasks with `[x]`.
- Mark in-progress tasks with `[/]`.
- Add new distinct tasks if the scope has expanded.
- **Goal:** Accurate status of "What is done" vs "What is left".

## 3. Update Context (`activeContext.md`)
- **Current Focus:** Update to reflect what effectively needs to be done *now*.
- **Recent Changes:** Add bullet points for what was just completed. Remove old items if list gets too long (keep last 3-5 major items).
- **Next Steps:** specific, actionable steps for the immediate future. Ensure they align with `progress.md`.

## 4. Updates for Architecture/Tech (Conditional)
- **If** we made technical decisions (e.g., "Use Tailwind v3", "Sidecars in .gitignore"):
    - Update `memory-bank/systemPatterns.md` -> "Key Technical Decisions".
- **If** we added dependencies (`npm install`, `cargo add`):
    - Update `memory-bank/techContext.md` -> "Technology Stack".
- **If** we fixed undocumented errors/bugs:
    - Update `docs/troubleshooting.md` (if exists) or add notes to `activeContext.md`.

## 5. Verification
- Ensure `activeContext.md` (Next Steps) points to the first unchecked items in `progress.md`.
- Ensure no contradictions exist between files.