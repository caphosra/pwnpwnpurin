use clap::{Parser, Subcommand};
use regex::Regex;
use reqwest::{Client, StatusCode};
use std::process::Command;
use std::env::current_dir;

use crate::builder::GLibCBuilder;
use crate::error::{InternalError, InternalResult};
use crate::file::FileManager;
use crate::log::LogSystem;

mod builder;
mod error;
mod file;
mod log;

#[derive(Debug, Parser)]
struct PurinArgs {
    #[command(subcommand)]
    subcommand: PurinSubCommand,
}

#[derive(Debug, Subcommand)]
enum PurinSubCommand {
    #[command(about = "Put the specified glibc to the current directory.")]
    Pick {
        #[arg(short, long, required = true)]
        version: String,
        #[arg(short, long, action)]
        force: bool
    },
    #[command(about = "List all prebuilt glibc.")]
    List,
}

fn get_glibc_source_url(version: &str) -> String {
    format!("https://ftp.gnu.org/gnu/glibc/glibc-{}.tar.xz", version)
}

async fn run_pick(version: String, force: bool) -> InternalResult<()> {
    let builder = GLibCBuilder::new();
    builder.check_source(&version).await?;

    let file_manager = FileManager::new()?;
    if file_manager.exists(&version) {
        let dest_dir = current_dir()?;
        file_manager.copy_to(&version, &dest_dir)?;

        LogSystem::log("Copied the cached library.".to_string());
        return Ok(());
    }

    LogSystem::log("There are no cached library.".to_string());

    builder.check_docker_installed()?;
    builder.build_docker_image(force)?;

    builder.build(&version, &file_manager)?;

    Ok(())
}

fn run_list() -> InternalResult<()> {
    let file_manager = FileManager::new()?;
    let glibc_list = file_manager.get_glibc_list()?;

    for version in glibc_list {
        LogSystem::log(format!("- glibc {}", version));
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = PurinArgs::parse();
    match args.subcommand {
        PurinSubCommand::Pick { version, force } => {
            if let Err(err) = run_pick(version, force).await {
                LogSystem::err(err.to_string());
            }
        }
        PurinSubCommand::List => {
            if let Err(err) = run_list() {
                LogSystem::err(err.to_string());
            }
        }
    }
}
