use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    iter::Map,
    path::PathBuf,
};

use crate::{ast_file::DirType, ast_mod::Mod, util::Expect};

#[derive(Debug)]
pub struct Crate {
    pub name: String,
    pub dir: PathBuf,
    pub main: Mod,
    pub deps: HashMap<String, String>,
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
            deps: Self::parse_cargo_toml(dir),
        }
    }

    fn parse_cargo_toml(dir: PathBuf) -> HashMap<String, String> {
        // Get the path to the `Cargo.toml` file
        let cargo_toml_path = dir.join("Cargo.toml");

        // Read the `Cargo.toml` file into a string
        let mut cargo_toml = String::new();
        let mut file = File::open(&cargo_toml_path).catch(format!(
            "Could not open Cargo.toml: {}",
            cargo_toml_path.display()
        ));
        file.read_to_string(&mut cargo_toml).catch(format!(
            "Could not read Cargo.toml: {}",
            cargo_toml_path.display()
        ));

        // Parse the `Cargo.toml` file as TOML
        let cargo_toml = cargo_toml.parse::<toml::Value>().catch(format!(
            "Could not parse Cargo.toml: {}",
            cargo_toml_path.display()
        ));

        // Extract the list of dependencies from the `Cargo.toml` file
        let deps = cargo_toml
            .get("dependencies")
            .expect("Could not find 'dependencies' section in Cargo.toml")
            .as_table()
            .expect("Could not convert 'dependencies' section to a table");
        deps.into_iter()
            .filter_map(|(k, v)| match v {
                toml::Value::Table(t) => match t.get("path") {
                    Some(toml::Value::String(p)) => Some((k.to_string(), p.to_string())),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}
