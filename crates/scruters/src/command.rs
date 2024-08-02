use color_eyre::{
    eyre::{anyhow, Context},
    Result,
};
use std::process::Stdio;
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

/// Run a command and return the stdout and stderr streams.
/// The command is spawned onto a background task.
pub(crate) fn spawn_command(
    mut command: Command,
    cancellation_token: CancellationToken,
) -> Result<(ChildStdout, ChildStderr)> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .wrap_err("failed to spawn command")?;

    let stdout = Option::take(&mut child.stdout)
        .ok_or_else(|| {
            anyhow!("failed to get stdout from command")
        })?;

    let stderr = Option::take(&mut child.stderr)
        .ok_or_else(|| {
            anyhow!("failed to get stderr from command")
        })?;

    tokio::spawn(async move {
        tokio::select! {
            () = cancellation_token.cancelled() => {
                debug!("Command was cancelled");
                if let Err(error) = child.kill().await {
                    error!(?error, "Failed to kill command");
                }
            }
            result = child.wait() => match result {
                Ok(exit_status) => {
                    if !exit_status.success() {
                        error!(?exit_status, "Command returned non-success exit status");
                    }
                }
                Err(error) => {
                    error!(?error, "Failed to wait on command");
                }
            }
        }
    });

    Ok((stdout, stderr))
}
