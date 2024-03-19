use clap::Parser;

use crate::commands::{CommandExec, PurinArgs};
use crate::log::LogSystem;

mod builder;
mod commands;
mod error;
mod file;
mod log;
mod utils;

#[tokio::main]
async fn main() {
    let args = PurinArgs::parse();
    match CommandExec::new() {
        Ok(exec) => {
            if let Err(err) = exec.exec_command(&args.subcommand).await {
                LogSystem::err(err.to_string());
            }
        }
        Err(err) => {
            LogSystem::err(err.to_string());
        }
    }
}
