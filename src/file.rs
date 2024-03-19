use home::home_dir;
use std::fs::{copy, create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use crate::error::{InternalError, InternalResult};
use crate::log::LogSystem;

include!(concat!(env!("OUT_DIR"), "/docker_file.rs"));

pub struct FileManager {
    config_dir: PathBuf,
}

impl FileManager {
    pub fn new() -> InternalResult<Self> {
        let mut home_dir = home_dir().ok_or(InternalError::Common(
            "Failed to get the home directory.".to_string(),
        ))?;
        home_dir.push(".config");
        home_dir.push("purin");

        if !home_dir.exists() {
            create_dir_all(&home_dir)?;
        }

        Ok(FileManager {
            config_dir: home_dir,
        })
    }

    pub fn get_glibc_dir(&self, version: &str) -> PathBuf {
        let mut glibc_dir = self.config_dir.clone();
        glibc_dir.push(format!("glibc-{}", version));
        glibc_dir
    }

    pub fn create_glibc_dir(&self, version: &str) -> InternalResult<()> {
        let glibc_dir = self.get_glibc_dir(version);
        create_dir_all(&glibc_dir)?;
        Ok(())
    }

    pub fn clean_glibc_dir(&self, version: &str) -> InternalResult<()> {
        let glibc_dir = self.get_glibc_dir(version);
        remove_dir_all(glibc_dir)?;
        Ok(())
    }

    pub fn exists(&self, version: &str) -> bool {
        self.get_glibc_dir(version).exists()
    }

    pub fn get_docker_file_path(&self) -> PathBuf {
        let mut docker_file_path = self.config_dir.clone();
        docker_file_path.push("Dockerfile");
        docker_file_path
    }

    pub fn create_docker_file(&self) -> InternalResult<PathBuf> {
        let docker_file_path = self.get_docker_file_path();
        if !docker_file_path.exists() {
            let mut source = File::create(&docker_file_path)?;
            source.write_all(DOCKER_FILE.as_bytes())?;

            LogSystem::log("Created a docker file.".to_string());
        }
        Ok(docker_file_path)
    }

    pub fn copy_to(&self, version: &str, dest: &PathBuf) -> InternalResult<()> {
        if !dest.exists() {
            Err(InternalError::IOError(format!(
                "A destination of the binary of glibc {} does not exist.",
                version
            )))?
        }

        let glibc_dir = self.get_glibc_dir(version);

        for lib_name in vec![
            &format!("libc-{}.so", version),
            &format!("ld-{}.so", version),
        ] {
            let mut src_path = glibc_dir.clone();
            src_path.push(lib_name);
            let mut dest_path = dest.clone();
            dest_path.push(lib_name);

            copy(src_path, &dest_path)?;
        }

        Ok(())
    }
}
