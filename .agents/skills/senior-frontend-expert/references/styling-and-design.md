# Styling & Design Excellence

Principles for creating "Wowed" UI experiences in VideoDownloaderPro.

## 1. Visual Design Principles

-   **Premium Aesthetics**:
    -   **Surface Palette**: Use `bg-surface-900` for main backgrounds, `bg-surface-800` for cards/panels, and `border-surface-700` for dividers.
    -   **Glassmorphism**: Combine `bg-surface-800/60` with `backdrop-blur-md` for overlays.
    -   **Glow Effects**: Use subtle shadows (e.g., `shadow-[0_0_10px_rgba(59,130,246,0.5)]` on progress bars).
-   **Typography**:
    -   **Tabular Numbers**: Use `style={{ fontVariantNumeric: 'tabular-nums' }}` for metrics (speed, size, progress) to prevent layout jitter.
    -   Maintain a clear visual hierarchy using `text-surface-100` for titles and `text-surface-400` for secondary info.

## 2. Tailwind CSS Patterns

-   **Utility-First**: Leverage Tailwind exclusively. Avoid ad-hoc CSS unless strictly necessary.
-   **Dynamic Classes**: Use the `cn` utility (tailwind-merge + clsx) for conditional styling.
-   **Hover & Transitions**: Always add `transition-all` or specific transition utilities to interactive elements. Use subtle `hover:scale-[1.02]` or `hover:brightness-110`.

## 3. Accessibility (A11y)

-   **Semantic HTML**: Use `<header>`, `<main>`, `<section>`, `<nav>`, `<button>` correctly.
-   **Aria Labels**: Provide `aria-label` for icon-only buttons.
-   **Focus States**: Ensure focus rings are visible and aesthetically pleasing (`focus-visible:ring-2`).
-   **Keyboard Navigation**: All interactive elements must be reachable and usable via keyboard.

## 4. Responsive Design

-   **Mobile First**: Design for small screens and scale up.
-   **Grid & Flexbox**: Use `grid` for layout skeletons and `flex` for content alignment.
-   **Safe Areas**: Respect OS-specific safe areas (especially relevant for desktop/mobile hybrids).
