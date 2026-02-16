# Product Context

## Why this project exists
The market for video downloaders is saturated with utility-focused, often clunky or ad-ridden software. **Video Downloader Pro** aims to be the "Linear" or "Raycast" of downloaders—a tool that feels professional, fast, and aesthetically pleasing. It shifts the mental model from "downloading a file" to "collecting content" via the "Wall" interface.

## User Experience Goals
*   **"It Just Works":** The user pastes a link, and the app handles the complexity (metadata, format selection, thumbnails, auth retries).
*   **Visual First:** The downloaded content is presented as a beautiful gallery, not a file explorer list.
*   **Trust & Privacy:** Explicit handling of cookies and credentials. No "black box" data transmission.
*   **Power User Friendly:** Keyboard shortcuts, advanced filtering, and granular control over download formats and paths.

## User Flows
1.  **Quick Download:** User copies a URL -> App detects clipboard or User pastes -> Metadata preview -> One-click Download.
2.  **The "Wall" Browsing:** User scrolls through thousands of downloaded items without lag. Items are grouped by Creator.
3.  **Authentication:** User encounters a "Login Required" error -> Click "Login" -> App opens a secure WebView -> User logs in -> App captures session and resumes download automatically.
4.  **Export/Portable:** User can take the "downloads" folder and walk away. The app enriches the folder structure but doesn't lock content inside a proprietary database blob.
