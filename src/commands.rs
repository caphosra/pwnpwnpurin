use clap::{ArgAction, Parser, Subcommand};

use std::env::current_dir;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::str::FromStr;

use crate::builder::GLibCBuilder;

use crate::error::{InternalError, InternalResult};
use crate::file::FileManager;
use crate::log::LogSystem;
use crate::patching::Patching;
use crate::utils::TryToStr;

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
    #[command(about = "Remove glibc from the cache.")]
    Rm {
        #[arg(help = "A version of glibc.")]
        version: String,
    },
    #[command(about = "Delete all caches and configurations.")]
    Clean,
    #[command(about = "Install the specified glibc to the current directory.")]
    Install {
        #[arg(help = "A version of glibc.")]
        version: String,
        #[arg(short, long, help = "Force purin to rebuild glibc.")]
        force: bool,
        #[arg(long, help = "Force purin to rebuild the image.")]
        rebuild_image: bool,
        #[arg(
            short,
            long,
            help = "A directory to install libraries. The current directory is chosen by default."
        )]
        dir: Option<String>,
        #[arg(short, long, action = ArgAction::Append, help = "Install glibc libraries other than `lib.so.6` and `ld-linux-x86-64.so.2`.")]
        lib: Option<Vec<String>>,
    },
    #[command(about = "List all pre-built glibc.")]
    List,
    #[command(about = "Patch the executable with designated glibc.")]
    Patch {
        #[arg(help = "A version of glibc.")]
        version: String,
        #[arg(help = "An executable file to be patched.")]
        executable: String,
        #[arg(short, long, help = "Force purin to rebuild glibc.")]
        force: bool,
        #[arg(long, help = "Force purin to rebuild the image.")]
        rebuild_image: bool,
        #[arg(short, long, action = ArgAction::Append, help = "Install glibc libraries other than `lib.so.6` and `ld-linux-x86-64.so.2`.")]
        lib: Option<Vec<String>>,
    },
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
            PurinSubCommand::Rm { version } => self.exec_rm(version).await,
            PurinSubCommand::Clean => self.exec_clean().await,
            PurinSubCommand::Install {
                version,
                force,
                rebuild_image,
                dir,
                lib,
            } => {
                let lib = lib.clone().unwrap_or(Vec::new());
                let mut lib = lib
                    .iter()
                    .map(|lib_name| lib_name.as_str())
                    .collect::<Vec<_>>();
                self.exec_install(version, *force, *rebuild_image, dir.as_deref(), &mut lib)
                    .await
            }
            PurinSubCommand::List => self.exec_list().await,
            PurinSubCommand::Patch {
                version,
                executable,
                force,
                rebuild_image,
                lib,
            } => {
                let lib = lib.clone().unwrap_or(Vec::new());
                let mut lib = lib
                    .iter()
                    .map(|lib_name| lib_name.as_str())
                    .collect::<Vec<_>>();
                self.exec_patch(version, executable, *force, *rebuild_image, &mut lib)
                    .await
            }
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

    pub async fn exec_rm(&self, version: &str) -> InternalResult<()> {
        if self.file_manager.exists(version) {
            self.file_manager.clean_glibc_dir(version)?;

            LogSystem::log("Deleted cached libraries.".to_string());
        } else {
            LogSystem::err(format!("Glibc {} is not found.", version));
        }

        Ok(())
    }

    pub async fn exec_clean(&self) -> InternalResult<()> {
        self.file_manager.clean()?;

        LogSystem::log("Deleted all cached libraries.".to_string());

        Ok(())
    }

    pub async fn exec_install(
        &self,
        version: &str,
        force: bool,
        rebuild_image: bool,
        dir: Option<&str>,
        lib: &mut Vec<&str>,
    ) -> InternalResult<()> {
        self.exec_build(version, force, rebuild_image).await?;

        let dest_dir = if let Some(dir) = dir {
            let dest_dir = PathBuf::from_str(dir).map_err(|_| {
                InternalError::Common(format!("Failed to interpret \"{}\" as a directory.", dir))
            })?;
            if !dest_dir.is_dir() {
                Err(InternalError::Common(format!(
                    "The directory \"{}\" does not exist.",
                    dir
                )))?;
            }
            canonicalize(dest_dir)?
        } else {
            current_dir()?
        };
        self.file_manager.copy_to(&version, &dest_dir, lib)?;

        LogSystem::success(format!(
            "Installed glibc to {}.",
            dest_dir.as_path().try_to_str()?
        ));

        Ok(())
    }

    pub async fn exec_list(&self) -> InternalResult<()> {
        let versions = self.file_manager.get_glibc_list()?;
        for version in versions {
            LogSystem::log(format!("glibc-{}", version));
        }
        Ok(())
    }

    pub async fn exec_patch(
        &self,
        version: &str,
        executable: &str,
        force: bool,
        rebuild_image: bool,
        lib: &mut Vec<&str>,
    ) -> InternalResult<()> {
        let exec_path = PathBuf::from_str(executable).map_err(|_| {
            InternalError::Common(format!("Failed to interpret \"{}\" as a path.", executable))
        })?;
        let exec_path = canonicalize(exec_path)?;
        if exec_path.is_file() && !exec_path.is_symlink() {
            let parent = exec_path.parent().ok_or(InternalError::Common(format!(
                "Failed to get the parent of \"{}\".",
                executable
            )))?;
            let parent_str = parent.try_to_str()?;
            self.exec_install(version, force, rebuild_image, Some(parent_str), lib)
                .await?;

            let patcher = Patching::new();
            patcher.patch(parent, &exec_path).await?;

            LogSystem::log(format!("Patched {} with glibc {}.", executable, version));

            Ok(())
        } else {
            Err(InternalError::Common(format!(
                "\"{}\" is not a file.",
                executable
            )))
        }
    }
}
