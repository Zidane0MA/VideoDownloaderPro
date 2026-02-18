# Auth System: Performance & Scaling Recommendations

This document outlines performance considerations and recommended strategies for scaling the authentication system, particularly as the number of supported platforms increases.

## Current Architecture

The current system uses a **WebView-based Cookie Extraction (L1)** approach for platforms that require complex login flows (e.g., YouTube).
- **Format**: Hidden `AppWindow` instances are spawned to navigate to the provider, extract cookies via `ICoreWebView2CookieManager`, and then close.
- **Concurrency**: Currently, extractions are triggered manually one by one.

## Performance Bottlenecks

1.  **Memory Consumption (RAM)**
    *   Each WebView instance (Edge/Chromium process) consumes significant memory (approx. 50MB - 150MB+ per instance).
    *   **Risk**: Simultaneous extraction for 10+ platforms could consume 1GB+ RAM, potentially crashing the app or slowing down the OS.

2.  **CPU Usage**
    *   Initializing a WebView process is CPU-intensive.
    *   **Risk**: Spawning multiple windows simultaneously causes CPU spikes, leading to UI freezes.

## Scaling Strategies (Future Implementation)

### 1. Sequential Queue System (Recommended)
Instead of allowing simultaneous checks, implement a queue:
- **Mechanism**: A `CheckoutQueue` that processes one platform at a time.
- **UX**: The user clicks "Update All". The UI shows a progress bar (1/10, 2/10...).
- **Benefit**: Keeps RAM usage constant (only 1 extra WebView open at a time).

### 2. Worker Window Reuse
Avoid destroying and creating windows repeatedly.
- **Mechanism**:
    1.  Create a single hidden `auth_worker` window at startup (or on first demand).
    2.  Send instructions to it: `navigate(url) -> wait_ready -> extract_cookies`.
    3.  Clear data/cache if necessary between different platforms (though separate partitions might be safer).
- **Benefit**: Eliminates process initialization overhead. Much faster "time-to-first-byte" for cookie extraction.

### 3. Hierarchical Auth Methods
Prioritize lighter methods before falling back to WebView.
- **L2 (Browser Import)**: Reading cookies from Chrome/Firefox via `yt-dlp` or direct SQLite reading is instant and uses negligible resources.
    *   *Action*: Make "Import from Browser" the default or bulk action.
    *   **Limitation**: Chromium-based browsers (Chrome v127+, Edge) now use **App-Bound Encryption**, often blocking external tools (like `yt-dlp`) from reading cookies. Firefox is currently more reliable for this method.
- **L3 (Manual)**: Zero resource cost, but higher user friction.

### 4. Headless alternatives (Advanced)
Investigate if lighter-weight HTTP clients can handle the refresh logic for certain platforms without a full browser engine (e.g., using refresh tokens where available).

## Implementation Roadmap (Draft)

- [ ] **Phase 1**: Implement `L2` (Browser Import) as a bulk operation.
- [ ] **Phase 2**: Create a `JobQueue` in Rust for `check_status` commands to enforce concurrency limit (Max 1).
- [ ] **Phase 3**: Refactor `auth.rs` to use a persistent `WorkerWindow` if delays become noticeable.
