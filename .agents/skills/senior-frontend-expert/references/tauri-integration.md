# Tauri v2 Integration Patterns

High-performance communication between Rust and React.

## 1. Commands (Invoke)

-   **Typed Commands**: Use TypeScript interfaces that strictly match Rust `struct`s.
-   **Error Handling**: Wrap `invoke` calls in `try/catch`. Expect errors from the Rust side and handle them gracefully in the UI.
-   **Payload Serialization**: Be mindful of large payloads; let the bridge handle JSON conversion but don't over-fetch.

## 2. Events (Listen/Emit)

-   **Global Listeners**: Encapsulate in `useEffect` within Manager hooks.
-   **Unlisten Pattern**: Always await the unlisten promise and call it inside the effect cleanup:
    ```typescript
    useEffect(() => {
      const setup = async () => {
        const unlisten = await listen('event', (e) => { ... });
        return unlisten;
      };
      const unlistenPromise = setup();
      return () => { unlistenPromise.then(u => u()); };
    }, []);
    ```
-   **Progress Streaming**: Use events for streaming real-time data like download progress or log updates.

## 3. File System & Media

-   **`convertFileSrc`**: Use for displaying local media (thumbnails, videos) without security issues.
-   **Sidecar Coordination**: Manage `yt-dlp` or `ffmpeg` lifecycle through Rust commands, reporting status back to React.

## 4. State Sync

-   **Zustand + Tauri**: Sync persistent settings between Rust's `tauri-plugin-store` and Zustand stores.
-   **Hydration**: Ensure the frontend "hydrates" its state from the backend on startup.
