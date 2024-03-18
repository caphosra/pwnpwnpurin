use regex::Regex;
use reqwest::{Client, StatusCode};
use std::fmt::format;
use std::process::{Command, Output};

use crate::error::{InternalError, InternalResult};
use crate::file::FileManager;
use crate::log::LogSystem;

pub struct GLibCBuilder {
    image_name: String,
}

impl GLibCBuilder {
    pub fn new() -> Self {
        GLibCBuilder {
            image_name: "purin:latest".to_string(),
        }
    }

    pub fn get_glibc_source_url(&self, version: &str) -> String {
        format!("https://ftp.gnu.org/gnu/glibc/glibc-{}.tar.xz", version)
    }

    pub fn get_glibc_source_name(&self, version: &str) -> String {
        format!("glibc-{}.tar.xz", version)
    }

    pub async fn check_source(&self, version: &str) -> InternalResult<()> {
        let regex = Regex::new(r"[0-9]+\.[0-9]+").unwrap();
        if !regex.is_match(&version) {
            Err(InternalError::Common(
                "A version of glibc must follow \"[0-9]+\\.[0-9]+\".".to_string(),
            ))?;
        }

        let glibc_url = self.get_glibc_source_url(&version);

        LogSystem::log(format!("Accessing {}.", glibc_url));

        let res = Client::new().get(&glibc_url).send().await?;
        let status = res.status();
        if status != StatusCode::OK {
            Err(InternalError::Common(format!(
                "The request failed with a status \"{} {}\".",
                status.as_u16(),
                status.canonical_reason().unwrap()
            )))?;
        }

        LogSystem::success(format!("Glibc {} is available.", version));

        Ok(())
    }

    pub fn check_docker_installed(&self) -> InternalResult<()> {
        let _ = Command::new("docker").output().map_err(|_| {
            InternalError::Common(
                "Docker is not found. Please make sure Docker is ready.".to_string(),
            )
        })?;

        LogSystem::log("Found Docker.".to_string());

        Ok(())
    }

    pub fn build_docker_image(&self, force: bool) -> InternalResult<()> {
        let docker_instance = Command::new("docker")
            .arg("images")
            .arg("-q")
            .arg(&self.image_name)
            .output()
            .map_err(|_| {
                InternalError::Common(
                    "Failed to figure out whether an image exists or not.".to_string(),
                )
            })?;

        let output = String::from_utf8(docker_instance.stdout).map_err(|_| {
            InternalError::Common("An output of Docker is not encoded with UTF8.".to_string())
        })?;

        if let Some(output) = output.lines().next() {
            LogSystem::log(format!("Found {} ({}).", self.image_name, output));
            if force {
                Command::new("docker")
                    .arg("rmi")
                    .arg("-f")
                    .arg(&self.image_name)
                    .output()
                    .map_err(|_| {
                        InternalError::Common(format!(
                            "Failed to delete the old {}.",
                            self.image_name
                        ))
                    })?;

                LogSystem::log(format!("Deleted {} ({}).", self.image_name, output));
            } else {
                return Ok(());
            }
        }

        LogSystem::log(format!("An image named {} is not found.", self.image_name));

        LogSystem::log(format!(
            "Building {}. It may take a long time.",
            self.image_name
        ));

        Command::new("docker")
            .arg("build")
            .arg(".")
            .arg("-t")
            .arg(&self.image_name)
            .arg("-f")
            .arg("purin.dockerfile")
            .output()
            .map_err(|_| InternalError::Common("Failed to build an image.".to_string()))?;

        LogSystem::log("Built the image.".to_string());

        Ok(())
    }

    pub fn exec_command_inside_docker<F>(
        &self,
        build_dir: &str,
        command_fn: F,
    ) -> InternalResult<Output>
    where
        F: FnOnce(&mut Command) -> &mut Command,
    {
        let mut command = Command::new("docker");
        let command_with_args = command
            .arg("run")
            .arg("--volume")
            .arg(format!("{}:/build", build_dir))
            .arg("--workdir")
            .arg("/build")
            .arg(&self.image_name);
        command_fn(command_with_args).output().map_err(|e| e.into())
    }

    pub fn build(&self, version: &str, file_manager: &FileManager) -> InternalResult<()> {
        let build_dir = file_manager.init_build_dir()?.to_string_lossy().to_string();

        self.exec_command_inside_docker(&build_dir, |command| {
            let src_url = self.get_glibc_source_url(version);
            command.arg("wget").arg(src_url)
        })
        .map_err(|_| InternalError::Common("Failed to download a source.".to_string()))?;

        self.exec_command_inside_docker(&build_dir, |command| {
            command
                .arg("tar")
                .arg("-xvf")
                .arg(self.get_glibc_source_name(version))
        })
        .map_err(|_| InternalError::Common("Failed to extract a source.".to_string()))?;

        Ok(())
    }
}
