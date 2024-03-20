use home::home_dir;
use regex::Regex;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use crate::error::{InternalError, InternalResult};
use crate::log::LogSystem;

include!(concat!(env!("OUT_DIR"), "/docker_file.rs"));

pub struct FileManager {
    config_dir: PathBuf,
    glibc_re: Regex,
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
            glibc_re: Regex::new(r"glibc-([0-9]+)\.([0-9]+)").unwrap(),
        })
    }

    pub fn clean(&self) -> InternalResult<()> {
        if self.config_dir.exists() {
            remove_dir_all(&self.config_dir)?;
            Ok(())
        }
        else {
            Err(InternalError::Common("Already cleaned.".to_string()))
        }
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

    pub fn get_glibc_list(&self) -> InternalResult<Vec<String>> {
        let mut versions = Vec::new();
        let entries = read_dir(&self.config_dir)?;
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                let name = entry.file_name();
                let name = name.to_str().ok_or(InternalError::Common(
                    "Failed to parse the directory name.".to_string(),
                ))?;
                if let Some(cap) = self.glibc_re.captures(name) {
                    let major_version = cap.get(1).unwrap().as_str().parse::<u32>().unwrap();
                    let minor_version = cap.get(2).unwrap().as_str().parse::<u32>().unwrap();

                    versions.push((major_version, minor_version));
                }
            }
        }
        versions.sort_by(|a, b| {
            if a.0 == b.0 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0)
            }
        });

        let versions = versions
            .iter()
            .map(|(major, minor)| format!("{}.{}", major, minor))
            .collect::<Vec<_>>();
        Ok(versions)
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

    pub fn possible_names(&self, version: &str) -> Vec<Vec<String>> {
        vec![
            vec![format!("libc-{}.so", version), "libc.so.6".to_string()],
            vec![
                format!("ld-{}.so", version),
                "ld-linux-x86-64.so.2".to_string(),
            ],
        ]
    }

    pub fn copy_to(&self, version: &str, dest: &PathBuf) -> InternalResult<()> {
        if !dest.exists() {
            Err(InternalError::IOError(format!(
                "A destination of the binary of glibc {} does not exist.",
                version
            )))?
        }

        let glibc_dir = self.get_glibc_dir(version);

        for lib_names in &self.possible_names(version) {
            let default_name = lib_names.first().unwrap();
            let mut found = false;
            for name in lib_names {
                let mut src_path = glibc_dir.clone();
                src_path.push("lib");
                src_path.push(name);
                if src_path.exists() {
                    let mut dest_path = dest.clone();
                    dest_path.push(default_name);

                    copy(src_path, &dest_path)?;

                    LogSystem::log(format!(
                        "Copied {} to the designated directory.",
                        default_name
                    ));

                    found = true;
                    break;
                }
            }
            if !found {
                Err(InternalError::Common(format!(
                    "Failed to find {}.",
                    default_name
                )))?;
            }
        }

        Ok(())
    }
}
