---
description: Handle critical changes or deviations from the planned documentation (/docs).
---

# Handling Critical Specification Changes

Use this workflow when you discover that the plan in `/docs` is technically infeasible, incorrect, or sub-optimal during implementation.

## 1. Stop and Assess
1. **Pause** the current coding task.
2. **Identify** the divergence:
    - What does `/docs` say?
    - What is the reality/requirement?
    - Why is the change necessary?

## 2. Update Source of Truth (/docs)
1. **Modify** the relevant file in `docs/` (e.g., `04_development_roadmap.md`, `07_ipc_api_contract.md`).
    - Mark the old section as ~~strikethrough~~ or remove it.
    - Add the new requirement/design clearly.
    - Add a `> [!NOTE]` explaining *why* it changed (optional but helpful).
2. **Commit** these changes (conceptually) as the new "Plan".

## 3. Update Project Context (Memory Bank)
1. **Update** `memory-bank/systemPatterns.md` if the change affects architecture.
2. **Update** `memory-bank/activeContext.md` to note the change in direction.

## 4. Resume Implementation
1. Return to your coding task.
2. Reference the *new* updated documentation in your `implementation_plan.md`.
3. Proceed with confidence that code and docs are aligned.
