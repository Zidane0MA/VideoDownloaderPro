# React 19 Best Practices

Senior-level patterns for building robust React applications in VideoDownloaderPro.

## 1. Component Composition

-   **Compound Components**: Use for complex UI elements like Modals or Tabs to provide a flexible API.
-   **Render Props / Slots**: Prefer passing elements as props (or children) over large configuration objects.
-   **Separation of Concerns**: Keep business logic in custom hooks; keep components focused on presentation.

## 2. Performance Optimization

-   **Memoization Strategy**:
    -   Use `useMemo` for derived lists (e.g., sorting tasks in `useDownloadManager`).
    -   Apply `React.memo` to list items (e.g., `DownloadItem`) to prevent unnecessary re-renders in virtualized grids.
-   **Virtualization**: Always use `@virtuoso.dev/masonry` for the Wall or `react-virtuoso` for the Downloads List.
-   **Lazy Loading**: Use `loading="lazy"` and `decoding="async"` for thumbnails in grids.

## 3. Hook Orchestration

-   **Manager Hooks**: Use "Manager" hooks (e.g., `useDownloadManager`) to encapsulate complex logic involving multiple stores, queries, and IPC events.
-   **Event Cleanup**: Always return cleanup functions in `useEffect` when using Tauri `listen`.
-   **Ref usage**: Use `useRef` for DOM access and persisting values across renders WITHOUT triggering re-renders.

## 4. TypeScript Excellence

-   **Discriminated Unions**: Use for complex state or event types.
-   **Utility Types**: Leverage `Pick`, `Omit`, `Partial`, and `Record`.
-   **Strict Typing**: Avoid `any`. Define interfaces for all component props and state.
