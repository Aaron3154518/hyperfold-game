use std::{fs, path::PathBuf};

use crate::{ast_file::DirType, ast_mod::Mod, util::Expect};

#[derive(Debug)]
pub struct Crate {
    pub name: String,
    pub dir: PathBuf,
    pub main: Mod,
}

impl Crate {
    pub fn new(dir: PathBuf, is_entry: bool) -> Self {
        let err_dir = dir.to_owned();
        let dir: PathBuf = fs::canonicalize(dir).catch(format!(
            "Could not canonicalize path: {}",
            err_dir.display()
        ));

        Self {
            name: if is_entry {
                "crate".to_string()
            } else {
                dir.file_name()
                    .catch(format!("Could not parse file name: {}", err_dir.display()))
                    .to_string_lossy()
                    .to_string()
            },
            dir: dir.to_owned(),
            main: Mod::parse_dir(
                dir.join("src"),
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
