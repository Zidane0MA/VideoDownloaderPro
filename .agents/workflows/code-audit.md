---
description: Audit a file or module to identify bad practices. Produces a report artifact. No code changes.
---

# `/code-audit` — Identificar Malas Prácticas

Use this workflow to scan a specific area of code and produce a prioritized findings report.
**CRITICAL: This workflow makes ZERO code changes. Read-only.**

## 1. Scope

1. Ask the user (or read from their message) which file or module to audit.
2. Read ONLY the specified file(s). Do not load extra context unless a dependency is strictly required to understand a finding.
3. Do NOT load Memory Bank, docs, or skills at this stage — conserve tokens.

## 2. Scan

Scan the loaded code for the following bad practices. Be precise: record file path + line number for each hit.

### Rust (Backend)
- `.unwrap()` or `.expect()` with no justification comment
- `let _ =` silently ignoring a `Result` or `Option`
- `println!` / `eprintln!` / `dbg!` used for debugging (not structured logging)
- Functions or methods exceeding **60 lines**
- Unnecessary `.clone()` inside loops or hot paths
- Blocking calls (`std::thread::sleep`, synchronous I/O) inside `async fn`

### TypeScript / React (Frontend)
- `console.log` / `console.error` left in production code
- Explicit `any` type annotations
- `useEffect` with no dependency array (runs on every render)
- Components exceeding **150 lines**
- Prop drilling deeper than **2 levels**
- Missing `useMemo` / `useCallback` on expensive computations passed as props

### Tauri / IPC
- `invoke()` calls with no `.catch()` or error handling in frontend
- Capabilities in `capabilities/*.json` granting broader permissions than the command requires

### General
- Commented-out ("zombie") code blocks
- `TODO` / `FIXME` comments with no date or owner
- Unused imports or variables
- Magic numbers (raw numeric literals not assigned to a named constant)
- Spaghetti code and bad architecture

## 3. Report

1. **Create an artifact** at the path `<appDataDir>/brain/<conversation-id>/audit_report.md` with the following structure:

```
# Audit Report — <file or module name> — <date>

## Summary
X findings: N High / N Medium / N Low

## Findings

### [HIGH] <Category>
- **File:** path/to/file.rs  **Line:** 42
- **Issue:** `.unwrap()` called on DB query result with no justification.
- **Snippet:** `let item = db.find(id).unwrap();`

### [MEDIUM] ...
### [LOW] ...
```

2. Use severity:
   - **HIGH** = Runtime crash risk, security issue, or silent data loss
   - **MEDIUM** = Code smell that degrades maintainability or performance
   - **LOW** = Style/convention violation

3. `notify_user` with a brief summary of findings and a link to the report artifact.
4. Recommend running `/code-fix` to address the findings.