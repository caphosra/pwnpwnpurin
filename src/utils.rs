use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

use crate::log::LogSystem;
use crate::error::{InternalError, InternalResult};

pub async fn execute_command<F>(program: &str, command_fn: F) -> InternalResult<()> where F: FnOnce(&mut Command) -> &mut Command {
    let mut command = Command::new(program);
    let mut child = command_fn(&mut command).spawn()?;
    let stdout = child.stdout.take().ok_or(InternalError::Common(
        "Failed to capture stdout.".to_string(),
    ))?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    while let Some(line) = stdout_reader.next_line().await? {
        LogSystem::log(format!(">> {}", line))
    }

    child.wait().await?;

    Ok(())
}
