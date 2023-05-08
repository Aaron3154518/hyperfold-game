use std::{fs, path::PathBuf};

use crate::{
    ast_file::{DirType, FileType},
    util::Expect,
};

#[derive(Debug)]
pub struct Crate {
    dir: PathBuf,
    file: FileType,
}

impl Crate {
    pub fn new(dir: PathBuf, is_entry: bool) -> Self {
        let err_dir = dir.to_owned();
        let dir: PathBuf = fs::canonicalize(dir).catch(format!(
            "Could not canonicalize path: {}",
            err_dir.display()
        ));

        Self {
            dir: dir.to_owned(),
            file: FileType::parse_dir(
                dir,
                &vec!["crate".to_string()],
                if is_entry {
                    DirType::Main
                } else {
                    DirType::Lib
                },
            ),
        }
    }
}
