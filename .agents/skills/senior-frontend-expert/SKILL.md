---
name: senior-frontend-expert
description: Specialized guidance for senior-level frontend engineering in VideoDownloaderPro. Use when building or refactoring UI components, managing state with Zustand/TanStack Query, or integrating React with Tauri v2. Triggers on React, TypeScript, Tailwind, Zustand, TanStack Query, and UI/UX design tasks.
---

# Senior Frontend Expert

This skill provides expert-level guidance for building modern, high-performance, and beautifully designed web applications within the VideoDownloaderPro ecosystem.

## Core Pillars

1.  **Architecture & Composition**: Leveraging React 19 features and clean component patterns.
2.  **State Management**: Efficiently balancing server state (TanStack Query) and client state (Zustand).
3.  **Visual Excellence**: Implementing premium UI/UX via Tailwind CSS and professional design principles.
4.  **Desktop Integration**: Seamless communication between Rust and React using Tauri v2.

## Project-Specific Patterns

This skill is pre-configured for VideoDownloaderPro's specific architecture:
-   **Design System**: Uses the `surface-NNN` and `brand-NNN` color scales.
-   **State Hierarchy**: Zustand for ephemeral UI and the download queue; TanStack Query for the Gallery (Wall) and persistent metadata.
-   **Tauri Bridge**: Authoritative status logic (Rust emits events, React updates stores).

## Reference Guides

To keep this skill lean, detailed guidance is split into modular reference files. Read these when diving into specific areas:

-   **[React Best Practices](references/react-best-practices.md)**: Composition, memoization, and custom hook orchestration.
-   **[Styling & Design](references/styling-and-design.md)**: Premium aesthetics, surface palette, and tabular numbers.
-   **[State Management](references/state-management.md)**: Zustand/Query split and optimistic updates.
-   **[Tauri Integration](references/tauri-integration.md)**: Event cleanup and authoritative status patterns.

## Senior Mindset

A Senior Frontend Engineer doesn't just "make it work." They:
-   **Anticipate Failures**: Implement robust error boundaries and loading states.
-   **Optimize for Maintenance**: Write self-documenting code and follow project-wide patterns.
-   **Prioritize Performance**: Minimize re-renders and bridge overhead.
-   **Obsess over UX**: Ensure interactions are fluid, accessible, and "wowed" at first glance.
