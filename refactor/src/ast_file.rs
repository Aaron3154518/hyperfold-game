use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::{ast_mod::Mod, util::Expect};

#[derive(Debug)]
pub enum DirType {
    Main,
    Lib,
    Mod,
}

impl DirType {
    fn to_string(&self) -> &str {
        match self {
            Self::Main => "src/main.rs",
            Self::Lib => "src/lib.rs",
            Self::Mod => "mod.rs",
        }
    }
}

#[derive(Debug)]
pub enum FileType {
    File {
        path: PathBuf,
        file_mod: Mod,
    },
    Dir {
        dir: PathBuf,
        children: Vec<FileType>,
    },
}

impl FileType {
    pub fn parse(path: PathBuf, mods: &Vec<String>) -> Self {
        if path.is_dir() {
            Self::parse_dir(path, mods, DirType::Mod)
        } else if path.is_file() {
            Self::parse_file(path, mods)
        } else {
            panic!("File does not exist: {}", path.display())
        }
    }

    pub fn parse_file(path: PathBuf, mods: &Vec<String>) -> Self {
        let file_contents = fs::read_to_string(path.to_owned())
            .catch(format!("Failed to read file: {}", path.display()));
        let ast = syn::parse_file(&file_contents).catch(format!(
            "Failed to parse file contents of: {}",
            path.display()
        ));
        let mut file_mod = Mod::new();
        file_mod.visit_file(&ast);
        Self::File { path, file_mod }
    }

    pub fn parse_dir(path: PathBuf, mods: &Vec<String>, ty: DirType) -> Self {
        let mod_rs = Self::parse_file(path.join(ty.to_string()), mods);
        // TODO: get siblings from mod declarations
        Self::Dir {
            dir: path,
            children: vec![mod_rs],
        }
    }
}
