---
description: Start and execute a new feature or phase based on the Memory Bank roadmap.
---

# Feature Implementation Process

Use this workflow when starting a new development task from the roadmap.

## 1. Context & Selection
1. **Read** `memory-bank/activeContext.md` to confirm the current phase and focus.
2. **Read** `memory-bank/progress.md` to pick the next unchecked item(s).
3. **Check** `memory-bank/systemPatterns.md` for architectural constraints relevant to the task (e.g., "Use SQLite", "IPC patterns").
4. **CRITICAL: Validate against `/docs`**: The `memory-bank` is a summary. You **MUST** read the detailed specification in `docs/` (e.g., `04_development_roadmap.md`) to ensure no requirements are missed.

## 2. Planning
1. **Create/Update** `implementation_plan.md` (Artifact).
    - **Goal**: Define *what* we are building.
    - **Proposed Changes**: List detailed file edits/creations.
    - **Verification**: How will we test it?
2. **Review**: `notify_user` to get approval on the plan.

## 3. Execution
1. **Step-by-Step**: Execute the plan's checklist.
2. **Tests**: Verify changes as you go (compile, run dev, test).

## 4. Completion Loop
1. **Verify**: Ensure the feature works as intended.
2. **Documentation**: Execute the **Update Memory Bank** workflow to mark progress and update context.