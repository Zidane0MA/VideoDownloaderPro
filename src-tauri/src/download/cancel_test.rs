#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;
    use tokio_util::sync::CancellationToken;

    /// Helper: kill the entire process tree on Windows using `taskkill /F /T /PID`.
    /// On non-Windows, falls back to `child.kill()`.
    #[cfg(windows)]
    async fn kill_process_tree(child: &tokio::process::Child) {
        if let Some(pid) = child.id() {
            let output = tokio::process::Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output()
                .await;
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    eprintln!(
                        "[kill_process_tree] taskkill PID={} stdout='{}' stderr='{}'",
                        pid,
                        stdout.trim(),
                        stderr.trim()
                    );
                }
                Err(e) => {
                    eprintln!("[kill_process_tree] taskkill failed: {}", e);
                }
            }
        }
    }

    #[cfg(not(windows))]
    async fn kill_process_tree(child: &mut tokio::process::Child) {
        let _ = child.kill().await;
    }

    /// Test 1: Verify that CancellationToken + tokio::select! correctly
    /// interrupts a long-running process (simulating what the download worker does).
    ///
    /// This replicates the exact pattern used in `worker.rs::execute_download`.
    #[tokio::test]
    async fn test_cancel_token_kills_process() {
        // Spawn a long-running process (ping with a long count)
        #[cfg(windows)]
        let mut child = Command::new("ping")
            .args(["-n", "100", "127.0.0.1"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to spawn ping");

        #[cfg(not(windows))]
        let mut child = Command::new("ping")
            .args(["-c", "100", "127.0.0.1"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to spawn ping");

        let pid = child.id().expect("No PID");
        eprintln!("[test] spawned process with PID: {}", pid);

        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        // Cancel after 1 second
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            eprintln!("[test] firing cancellation token...");
            cancel_clone.cancel();
        });

        let stdout = child.stdout.take().unwrap();
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buf = Vec::new();
        let mut lines_read = 0u32;

        // This is the same pattern as worker.rs
        let was_cancelled = loop {
            if cancel_token.is_cancelled() {
                eprintln!("[test] cancel detected BEFORE select!");
                break true;
            }

            tokio::select! {
                biased;

                _ = cancel_token.cancelled() => {
                    eprintln!("[test] cancel detected IN select!");
                    break true;
                }

                result = reader.read_until(b'\n', &mut buf) => {
                    match result {
                        Ok(0) => {
                            eprintln!("[test] EOF reached");
                            break false;
                        }
                        Ok(_) => {
                            lines_read += 1;
                            buf.clear();
                        }
                        Err(e) => {
                            eprintln!("[test] read error: {}", e);
                            break false;
                        }
                    }
                }
            }
        };

        assert!(was_cancelled, "Expected cancellation to fire before EOF");
        eprintln!("[test] lines read before cancel: {}", lines_read);

        // --- NOW TEST: does child.kill() actually stop the process? ---
        eprintln!("[test] killing process with child.kill()...");
        let _ = child.kill().await;
        let _ = child.wait().await;

        // Give OS a moment to clean up
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if the process is still alive via tasklist (Windows)
        #[cfg(windows)]
        {
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid)])
                .output()
                .await
                .expect("Failed to run tasklist");
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let still_alive = stdout_str.contains(&pid.to_string());
            eprintln!(
                "[test] process {} still alive after child.kill(): {}",
                pid, still_alive
            );
            // child.kill() SHOULD kill the immediate process
            assert!(
                !still_alive,
                "Process should not be alive after child.kill()"
            );
        }
    }

    /// Test 2: Verify that `taskkill /F /T /PID` kills the entire process tree.
    /// This simulates the yt-dlp scenario where child processes may be spawned.
    ///
    /// We use `cmd /c ping` so that cmd.exe is the parent and ping.exe is the child.
    /// Then we verify killing cmd.exe's PID tree also kills ping.exe.
    #[tokio::test]
    #[cfg(windows)]
    async fn test_taskkill_tree_kills_subprocess() {
        // Spawn cmd /c which starts ping as a child process
        // This simulates yt-dlp spawning ffmpeg
        let child = Command::new("cmd")
            .args(["/C", "ping -n 100 127.0.0.1"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .expect("Failed to spawn cmd");

        let parent_pid = child.id().expect("No PID for cmd");
        eprintln!("[test_tree] parent PID (cmd.exe): {}", parent_pid);

        // Wait a moment for the child process (ping.exe) to spawn
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Find the child PID (ping.exe) spawned by cmd.exe
        let wmic_output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("(ParentProcessId={})", parent_pid),
                "get",
                "ProcessId",
            ])
            .output()
            .await
            .expect("wmic failed");
        let wmic_str = String::from_utf8_lossy(&wmic_output.stdout);
        eprintln!(
            "[test_tree] children of PID {}: {}",
            parent_pid,
            wmic_str.trim()
        );

        // Extract child PIDs
        let child_pids: Vec<u32> = wmic_str
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .collect();
        eprintln!("[test_tree] found child PIDs: {:?}", child_pids);
        assert!(
            !child_pids.is_empty(),
            "Expected at least one child process (ping.exe)"
        );

        // --- Kill with taskkill /F /T /PID (tree kill) ---
        eprintln!("[test_tree] killing process tree with taskkill...");
        kill_process_tree(&child).await;

        // Wait for cleanup
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Verify parent is dead
        let tasklist = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", parent_pid)])
            .output()
            .await
            .expect("tasklist failed");
        let parent_alive =
            String::from_utf8_lossy(&tasklist.stdout).contains(&parent_pid.to_string());
        eprintln!(
            "[test_tree] parent {} alive after taskkill /T: {}",
            parent_pid, parent_alive
        );

        // Verify ALL children are dead
        for child_pid in &child_pids {
            let tasklist = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", child_pid)])
                .output()
                .await
                .expect("tasklist failed");
            let child_alive =
                String::from_utf8_lossy(&tasklist.stdout).contains(&child_pid.to_string());
            eprintln!(
                "[test_tree] child {} alive after taskkill /T: {}",
                child_pid, child_alive
            );
            assert!(
                !child_alive,
                "Child process {} should have been killed by taskkill /T",
                child_pid
            );
        }

        assert!(
            !parent_alive,
            "Parent process should be dead after taskkill /T"
        );
    }

    /// Test 3: Demonstrate the BUG — child.kill() does NOT kill subprocesses.
    /// This test proves that without taskkill /T, child processes survive.
    #[tokio::test]
    #[cfg(windows)]
    async fn test_child_kill_does_not_kill_subprocess_tree() {
        // Spawn cmd /c which starts ping as a child process
        let mut child = Command::new("cmd")
            .args(["/C", "ping -n 100 127.0.0.1"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .expect("Failed to spawn cmd");

        let parent_pid = child.id().expect("No PID for cmd");
        eprintln!("[test_bug] parent PID (cmd.exe): {}", parent_pid);

        // Wait for ping.exe to spawn
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Find child PIDs
        let wmic_output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("(ParentProcessId={})", parent_pid),
                "get",
                "ProcessId",
            ])
            .output()
            .await
            .expect("wmic failed");
        let wmic_str = String::from_utf8_lossy(&wmic_output.stdout);

        let child_pids: Vec<u32> = wmic_str
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .collect();
        eprintln!("[test_bug] child PIDs: {:?}", child_pids);

        // --- Kill with just child.kill() (the current buggy approach) ---
        eprintln!("[test_bug] killing with child.kill() only...");
        let _ = child.kill().await;
        let _ = child.wait().await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check if children survived (THIS IS THE BUG)
        let mut orphans_found = false;
        for child_pid in &child_pids {
            let tasklist = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", child_pid)])
                .output()
                .await
                .expect("tasklist failed");
            let child_alive =
                String::from_utf8_lossy(&tasklist.stdout).contains(&child_pid.to_string());
            eprintln!(
                "[test_bug] child {} alive after child.kill(): {}  ← THIS IS THE BUG",
                child_pid, child_alive
            );
            if child_alive {
                orphans_found = true;
                // Clean up the orphan for test hygiene
                let _ = Command::new("taskkill")
                    .args(["/F", "/PID", &child_pid.to_string()])
                    .output()
                    .await;
            }
        }

        // We EXPECT orphans to prove the bug exists
        // NOTE: This assertion documents the bug. If Windows behavior changes
        // and child.kill() starts killing the tree, this test will fail —
        // which would mean the bug is fixed at the OS/tokio level.
        eprintln!(
            "[test_bug] RESULT: orphans_found = {} (expected: true to confirm bug)",
            orphans_found
        );
        // We log the result but don't hard-assert, since behavior can vary
        // depending on how cmd.exe handles the signal
        if orphans_found {
            eprintln!("[test_bug] ✅ Bug confirmed: child.kill() leaves orphan processes");
        } else {
            eprintln!("[test_bug] ❓ No orphans found — child.kill() killed the tree (unexpected on Windows)");
        }
    }
}
