use regex::Regex;
use reqwest::{Client, StatusCode};
use tokio::process::Command;

use crate::error::{InternalError, InternalResult};
use crate::exec_com;
use crate::file::FileManager;
use crate::log::LogSystem;
use crate::utils::TryToStr;

pub struct GLibCBuilder {
    image_name: String,
    container_name: String,
}

///
/// Executes a command inside the container.
/// This macro should be used in a body of a function which returns [InternalResult](crate::error::InternalResult).
///
macro_rules! exec_com_inside {
    ($($arg: expr),*; $container_name: expr, $work_dir: expr; $err: expr) => {{
        exec_com!(
            "docker", "container", "exec", "--tty", "--workdir", $work_dir, $container_name, $($arg),*;
            $err
        )
    }};
}

impl GLibCBuilder {
    pub fn new() -> Self {
        GLibCBuilder {
            image_name: "purin:latest".to_string(),
            container_name: "purin-exec".to_string(),
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

    pub async fn check_docker_installed(&self) -> InternalResult<()> {
        let _ = Command::new("docker").output().await.map_err(|_| {
            InternalError::Common(
                "Docker is not found. Please make sure Docker is ready.".to_string(),
            )
        })?;

        LogSystem::log("Found Docker.".to_string());

        Ok(())
    }

    pub async fn build_docker_image(
        &self,
        rebuild_image: bool,
        file_manager: &FileManager,
    ) -> InternalResult<()> {
        let output = exec_com!(
            "docker", "images", "-q", &self.image_name;
            "Failed to figure out whether an image exists or not.".to_string()
        );

        if let Some(image_hash) = output.first() {
            LogSystem::log(format!("Found {} ({}).", self.image_name, image_hash));
            if rebuild_image {
                exec_com!(
                    "docker", "rmi", "-f", &self.image_name;
                    format!(
                        "Failed to delete the old {}.",
                        self.image_name
                    )
                );

                LogSystem::log(format!("Deleted {} ({}).", self.image_name, image_hash));
            } else {
                return Ok(());
            }
        }

        LogSystem::log(format!("An image named {} is not found.", self.image_name));

        LogSystem::log(format!(
            "Building {}. It may take a long time.",
            self.image_name
        ));

        let docker_file_path = file_manager.create_docker_file()?;
        let docker_dir = docker_file_path.parent().ok_or(InternalError::Common(
            "Failed to get the directory of docker file path.".to_string(),
        ))?;
        let docker_dir_str = docker_dir.try_to_str()?;
        exec_com!(
            "docker", "build", docker_dir_str, "--tag", &self.image_name, "--progress", "plain";
            "Failed to build an image.".to_string()
        );

        LogSystem::log("Built the image.".to_string());

        Ok(())
    }

    pub async fn stop_container(&self) -> InternalResult<()> {
        exec_com!(
            "docker", "container", "stop", &self.container_name;
            "Failed to stop the running container.".to_string()
        );

        exec_com!(
            "docker", "container", "rm", &self.container_name;
            "Failed to remove the stopped container.".to_string()
        );

        LogSystem::log("Stopped and removed the container.".to_string());

        Ok(())
    }

    pub async fn build(&self, version: &str, file_manager: &FileManager) -> InternalResult<()> {
        let output = exec_com!(
            "docker", "container", "ls", "--all", "--filter", format!("name={}", &self.container_name), "--quiet";
            "Failed to find the running container.".to_string()
        );
        if output.len() > 0 {
            LogSystem::log("Found a running container of purin. Going to stop it.".to_string());

            self.stop_container().await?;
        }

        exec_com!(
            "docker", "container", "run", "--detach", "--tty", "--name", &self.container_name, &self.image_name, "sh";
            "Failed to spawn a new container.".to_string()
        );

        LogSystem::log("Spawned a container to build glibc.".to_string());

        exec_com_inside!(
            "mkdir", "-p", "/build/dest";
            &self.container_name, "/";
            "Failed to create folders.".to_string()
        );

        exec_com_inside!(
            "mkdir", "-p", "/build/out";
            &self.container_name, "/";
            "Failed to create folders.".to_string()
        );

        exec_com_inside!(
            "wget", self.get_glibc_source_url(version);
            &self.container_name, "/build";
            "Failed to download a source.".to_string()
        );

        LogSystem::log(format!("Downloaded the source of glibc {}.", version));

        exec_com_inside!(
            "tar", "-xf", self.get_glibc_source_name(version);
            &self.container_name, "/build";
            "Failed to extract a source.".to_string()
        );

        LogSystem::log(format!("Extracted the source of glibc {}.", version));

        exec_com_inside!(
            "bash", "-c", format!("../glibc-{}/configure --disable-werror --prefix=/build/out CC=\"gcc -m64\" CXX=\"g++ -m64\"", version);
            &self.container_name, "/build/dest";
            "Failed to configure the glibc.".to_string()
        );

        LogSystem::log(format!("Configured glibc {}.", version));

        exec_com_inside!(
            "bash", "-c", "make -j `nproc`";
            &self.container_name, "/build/dest";
            "Failed to build the glibc.".to_string()
        );

        LogSystem::success(format!("Built glibc {}.", version));

        exec_com_inside!(
            "bash", "-c", "make install";
            &self.container_name, "/build/dest";
            "Failed to install the glibc to the container.".to_string()
        );

        file_manager.create_glibc_dir(version)?;
        let dir_path = file_manager.get_glibc_dir(version);
        let dir_path = dir_path.try_to_str()?;

        exec_com!(
            "docker", "container", "cp",
                format!("{}:/build/out/.", self.container_name),
                dir_path;
            "Failed to copy libraries.".to_string()
        );

        LogSystem::log("Extracted libraries from the container.".to_string());

        self.stop_container().await?;

        LogSystem::log("Stopped the container.".to_string());

        Ok(())
    }
}
