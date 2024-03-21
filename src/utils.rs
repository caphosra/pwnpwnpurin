use std::process::Stdio;
use std::path::Path;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::error::{InternalError, InternalResult};
use crate::log::LogSystem;

pub async fn execute_command<F>(program: &str, command_fn: F) -> InternalResult<Vec<String>>
where
    F: FnOnce(&mut Command) -> &mut Command,
{
    let mut command = Command::new(program);
    let mut command_with_pipe = command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command_fn(&mut command_with_pipe).spawn()?;
    let stdout = child.stdout.take().ok_or(InternalError::Common(
        "Failed to capture stdout.".to_string(),
    ))?;
    let stderr = child.stderr.take().ok_or(InternalError::Common(
        "Failed to capture stderr.".to_string(),
    ))?;

    let mut lines = Vec::new();
    let mut stdout_reader = BufReader::new(stdout).lines();
    while let Some(line) = stdout_reader.next_line().await? {
        LogSystem::subprocess(format!(">> {}", line));
        lines.push(line);
    }

    let mut stderr_reader = BufReader::new(stderr).lines();
    while let Some(line) = stderr_reader.next_line().await? {
        LogSystem::warn(format!(">> {}", line));
    }

    let status = child.wait().await?;
    if !status.success() {
        Err(InternalError::Common(format!(
            "The program \"{}\" finished with a status {}.",
            program,
            status.to_string()
        )))
    } else {
        Ok(lines)
    }
}

#[macro_export]
macro_rules! exec_com {
    ($program: expr, $($arg: expr),*; $err: expr) => {{
        crate::utils::execute_command($program, |command| {
            command $(.arg($arg))*
        })
        .await
        .map_err(|err| {
            crate::error::InternalError::Multiple(
                Box::new(crate::error::InternalError::Common($err)),
                Box::new(err)
            )
        })?
    }};
}

pub trait TryToStr<'a> {
    fn try_to_str(&'a self) -> InternalResult<&'a str>;
}

impl<'a> TryToStr<'a> for Path {
    fn try_to_str(&'a self) -> InternalResult<&'a str> {
        self.to_str().ok_or(InternalError::Common(format!("Failed to interpret \"{}\" as a path.", self.display())))
    }
}
