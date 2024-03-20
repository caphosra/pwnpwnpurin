use clap::{Parser, Subcommand};

use std::env::current_dir;

use crate::builder::GLibCBuilder;

use crate::error::InternalResult;
use crate::file::FileManager;
use crate::log::LogSystem;

#[derive(Debug, Parser)]
pub struct PurinArgs {
    #[command(subcommand)]
    pub subcommand: PurinSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum PurinSubCommand {
    #[command(about = "Build the specified glibc.")]
    Build {
        #[arg(help = "A version of glibc.")]
        version: String,
        #[arg(short, long, help = "Force purin to rebuild glibc.")]
        force: bool,
        #[arg(long, help = "Force purin to rebuild the image.")]
        rebuild_image: bool,
    },
    #[command(about = "Put the specified glibc to the current directory.")]
    Install {
        #[arg(help = "A version of glibc.")]
        version: String,
        #[arg(short, long, action)]
        force: bool,
        #[arg(long, help = "Force purin to rebuild the image.")]
        rebuild_image: bool,
    },
    #[command(about = "List all prebuilt glibc.")]
    List,
}

pub struct CommandExec {
    file_manager: FileManager,
}

impl CommandExec {
    pub fn new() -> InternalResult<Self> {
        let file_manager = FileManager::new()?;
        Ok(CommandExec { file_manager })
    }

    pub async fn exec_command(&self, com: &PurinSubCommand) -> InternalResult<()> {
        match com {
            PurinSubCommand::Build {
                version,
                force,
                rebuild_image,
            } => self.exec_build(version, *force, *rebuild_image).await,
            PurinSubCommand::Install {
                version,
                force,
                rebuild_image,
            } => self.exec_install(version, *force, *rebuild_image).await,
            PurinSubCommand::List => self.exec_list().await,
        }
    }

    pub async fn exec_build(
        &self,
        version: &str,
        force: bool,
        rebuild_image: bool,
    ) -> InternalResult<()> {
        let builder = GLibCBuilder::new();
        builder.check_source(&version).await?;

        if force && self.file_manager.exists(version) {
            self.file_manager.clean_glibc_dir(version)?;

            LogSystem::log("Deleted cached libraries.".to_string());
        }
        if force || !self.file_manager.exists(version) {
            builder.check_docker_installed().await?;
            builder
                .build_docker_image(rebuild_image, &self.file_manager)
                .await?;

            builder.build(&version, &self.file_manager).await?;
        } else {
            LogSystem::log("Already built.".to_string());
        }

        Ok(())
    }

    pub async fn exec_install(
        &self,
        version: &str,
        force: bool,
        rebuild_image: bool,
    ) -> InternalResult<()> {
        self.exec_build(version, force, rebuild_image).await?;

        let dest_dir = current_dir()?;
        self.file_manager.copy_to(&version, &dest_dir)?;

        LogSystem::success("Installed glibc to the directory.".to_string());

        Ok(())
    }

    pub async fn exec_list(&self) -> InternalResult<()> {
        Ok(())
    }
}
