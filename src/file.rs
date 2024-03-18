use home::home_dir;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::path::PathBuf;

use crate::error::{InternalError, InternalResult};
use crate::log::LogSystem;

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

    pub fn get_glibc_name(&self, version: &str) -> String {
        format!("glibc-{}.so", version)
    }

    pub fn exists(&self, version: &str) -> bool {
        self.get_glibc_path(version).exists()
    }

    pub fn get_glibc_path(&self, version: &str) -> PathBuf {
        let mut target = self.config_dir.clone();
        target.push(self.get_glibc_name(version));

        target
    }

    pub fn get_build_dir(&self) -> PathBuf {
        let mut build_dir = self.config_dir.clone();
        build_dir.push("build");
        build_dir
    }

    pub fn init_build_dir(&self) -> InternalResult<PathBuf> {
        let build_dir = self.get_build_dir();
        if build_dir.exists() {
            remove_dir_all(&build_dir)?;
            LogSystem::log("Deleted artifacts in the build dir.".to_string())
        }
        create_dir_all(&build_dir)?;
        LogSystem::log("Created a new build dir.".to_string());

        Ok(build_dir)
    }

    pub fn get_glibc_list(&self) -> InternalResult<Vec<String>> {
        let mut glibc_list = Vec::new();
        for entry in read_dir(&self.config_dir)? {
            let dir_entry = entry?;
            dir_entry.file_type()?;
        }
        Ok(glibc_list)
    }

    pub fn copy_to(&self, version: &str, dest: &PathBuf) -> InternalResult<()> {
        if !dest.exists() {
            Err(InternalError::IOError(format!(
                "A destination of the binary of glibc {} does not exist.",
                version
            )))?
        }

        let lib = self.get_glibc_path(&version);
        if !lib.exists() {
            Err(InternalError::IOError(format!(
                "A binary of glibc {} does not exist.",
                version
            )))?
        }

        let mut dest = dest.clone();
        dest.push(self.get_glibc_name(version));

        copy(lib, dest)?;

        Ok(())
    }
}
