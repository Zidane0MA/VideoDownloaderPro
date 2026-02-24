---
description: Workflow to find and implement username extraction for new platforms using local DB cookies.
---

# Debug Username Extraction

This workflow helps you identify the correct cookie names to use for extracting usernames for platforms like YouTube, TikTok, X, etc., by using real authenticated sessions from your local database.

## Prerequisites

1.  **Login**: Ensure you have logged into the target platform in the app (WebView, Browser Import, or Manual).
2.  **State**: The session status must be `ACTIVE`.

## Steps

### 1. Run the Debug Test

Execute the integration test that prints all cookies for active sessions.

```powershell
cd src-tauri
cargo test auth::cookie_tests::tests::test_extract_from_local_db -- --ignored --nocapture
```

### 2. Analyze the Output

Look for the "Cookie Dump" section for your target platform.

```text
Processing session for platform: tiktok
  -> Decrypted cookies successfully. Length: 1234
  -> Cookie Dump for tiktok:
     - .tiktok.com | sessionid = abcde...
     - .tiktok.com | csrf_session = 12345...
  -> Attempting TikTok API fetch (no ID required)...
  -> API Fetched Username: None
```

### 3. Identify the API Endpoint and Parsing

If the username is `None`, you need to implement or fix the extraction logic in `src-tauri/src/auth/api.rs`:
- Look at the `Cookie Dump` to understand what auth tokens are available.
- Use tools like Postman or browser DevTools to find the platform's API endpoint that returns user info (e.g., `/passport/web/account/info/` for TikTok).

### 4. Update `api.rs`

Open `src-tauri/src/auth/api.rs` and update or add the corresponding `fetch_*_username` function.
For example, for a new platform:

```rust
pub async fn fetch_newplatform_username(cookies: &str) -> Option<(String, Option<String>)> {
    // 1. Create reqwest Client
    // 2. Add Cookie header
    // 3. Send GET/POST request to platform API
    // 4. Parse JSON/HTML response to extract handle and avatar
    // 5. Return Some((handle, avatar))
}
```

Then hook it up in the `fetch_profile` matching block!

### 5. Verify

Re-run the test to confirm the username is now correctly extracted via the API.

```powershell
cargo test auth::cookie_tests::tests::test_extract_from_local_db -- --ignored --nocapture
```

Output should now show the API successfully fetching it:
```text
  -> API Fetched Username: Some(("my_awesome_handle", Some("https://...")))
```

## Troubleshooting / Retry

If the process stalls, context gets confusing, or you make too many experimental changes:

1.  **Stop**: Don't keep digging if you're blocked.
2.  **New Chat**: Start a fresh chat session with the AI.
3.  **Provide Context**: Mention "I am debugging username extraction for [Platform]" and point it to this workflow file.
4.  **Re-run**: Run the debug test again in the new chat to get a clean slate of logs.

## Platform Support Status

Track which platforms have verified username extraction:

- [x] **TikTok** (via Passport `account/info` API)
- [x] **X (Twitter)** (found `User ID`, API fetch pending/best-effort)
- [ ] **Instagram** (Pending verification)
- [x] **YouTube** (via InnerTube `account_menu` API using `SAPISIDHASH`)
- [ ] **Facebook** (Pending)
- [ ] **Twitch** (Pending)
