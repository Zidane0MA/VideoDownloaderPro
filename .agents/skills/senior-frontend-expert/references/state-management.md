# State Management Strategy

Orchestrating client and server state effectively.

## 1. Client State (Zustand)

-   **Store Granularity**: Separate queue state (`useDownloadStore`) from persistent feature state.
-   **Selectors**: Use reactive selectors to minimize renders: `const tasks = useStore(s => s.tasks);`.

## 2. Server State (TanStack Query)

-   **Invalidation**: Use `queryClient.invalidateQueries({ queryKey: ['posts'] })` after mutations (delete, restore) to keep the Gallery fresh.
-   **Optimistic Updates**: Implement optimistic state changes in Manager hooks for immediate UI feedback before the backend confirms.

## 3. State Source of Truth

-   **Authoritative Backend**: Treat backend events as the final word on task status. The UI should "follow" backend state, not try to predict it perfectly.

## 4. Derived State

-   **Avoid Redundancy**: If a value can be computed from existing state/props, do NOT store it in `useState` or a store.
-   **Computations**: Handle heavy derivations inside `useMemo`.

## 4. Error Handling

-   **Error Boundaries**: Wrap major UI sections to prevent full-app crashes.
-   **Feedback Messaging**: Use a global toast system or inline alerts to communicate errors clearly to the user.
