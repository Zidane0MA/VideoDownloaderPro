use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_process_cancellation_windows() {
    // 1. Spawn a long-running process (ping localhost)
    let mut cmd = Command::new("ping");
    cmd.arg("-t").arg("127.0.0.1");

    // Windows-specific creation flags to hide window (mimic worker.rs)
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().expect("Failed to spawn process");
    let pid = child.id().expect("Failed to get PID");
    println!("Spawned process with PID: {}", pid);

    // 2. Setup cancellation
    let cancel_token = CancellationToken::new();
    let token_clone = cancel_token.clone();

    // 3. Spawn a task that cancels after 2 seconds
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("Cancelling token...");
        token_clone.cancel();
    });

    // 4. Run the select loop (mimic worker.rs)
    let join_handle = tokio::spawn(async move {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                println!("Token cancelled. Killing process...");
                let _ = child.kill().await;
                // child.wait().await.expect("Failed to wait"); // worker.rs does this
                println!("Process killed.");
            }
            _ = child.wait() => {
                println!("Process exited on its own (unexpected).");
            }
        }

        // Return PID so we can check if it's still alive
        pid
    });

    let pid = join_handle.await.expect("Task failed");

    // 5. Verify process is truly dead using system tools
    tokio::time::sleep(Duration::from_secs(1)).await;

    let check = std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid))
        .output()
        .expect("Failed to run tasklist");

    let output = String::from_utf8_lossy(&check.stdout);
    println!("Tasklist output:\n{}", output);

    // If PID is found, tasklist output usually contains the image name.
    // If not found, it says "No tasks are running..."
    let is_alive = output.contains(&pid.to_string());

    assert!(
        !is_alive,
        "Process {} is still running after cancellation!",
        pid
    );
}
