use std::path::{Path, PathBuf};

use crate::{error::InternalResult, exec_com, utils::TryToStr};

pub struct Patching;

impl Patching {
    pub fn new() -> Self { Self { } }

    pub async fn patch(&self, parent: &Path, exec_path: &Path) -> InternalResult<()> {
        let mut ld_path: PathBuf = parent.into();
        ld_path.push( "ld-linux-x86-64.so.2");

        let parent_path = parent.try_to_str()?;
        let ld_path = ld_path.as_path().try_to_str()?;
        let exec_path = exec_path.try_to_str()?;

        exec_com!(
            "patchelf", "--set-interpreter", ld_path, "--set-rpath", parent_path, exec_path;
            format!("Failed to patch {}. Please make sure patchelf is installed properly.", exec_path)
        );

        Ok(())
    }
}
