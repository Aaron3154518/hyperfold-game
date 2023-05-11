use std::{array, fs};

use regex::Regex;

use crate::{
    util::Catch,
    validate::constants::{DATA_FILE, SEP},
};

use super::dependency::Dependency;

#[derive(Copy, Clone, Debug)]
pub enum Data {
    Components,
    Globals,
    Events,
    Systems,
    Dependencies,
}

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; 5],
}

impl Decoder {
    pub fn new() -> Self {
        let data = fs::read_to_string(std::env::temp_dir().join(DATA_FILE))
            .expect("Could not read data file");
        let mut lines = data.split("\n");
        Self {
            data: array::from_fn(|i| {
                lines
                    .next()
                    .catch(format!("Could not get data line: {}", i))
                    .split(SEP)
                    .map(|s| s.to_string())
                    .collect()
            }),
        }
    }

    pub fn get_dependencies(&self, name: String) -> (usize, Vec<Dependency>) {
        let dep_regex =
            Regex::new(r"(?P<alias>\w+):(?P<idx>\d+)").expect("Could not crate dependency regex");
        self.data[Data::Dependencies as usize]
            .iter()
            .enumerate()
            .find_map(|(i, deps)| {
                deps.strip_prefix(format!("{}:", name).as_str())
                    .map(|deps| {
                        (
                            i,
                            deps.split(",")
                                .map(|dep| {
                                    dep_regex
                                        .captures(dep)
                                        .and_then(|c| c.name("alias").zip(c.name("idx")))
                                        .map(|(alias, idx)| Dependency {
                                            cr_idx: idx
                                                .as_str()
                                                .parse()
                                                .expect("Could not parse dependency index"),
                                            alias: alias.as_str().to_string(),
                                        })
                                        .catch(format!("Could not parse dependency: {}", dep))
                                })
                                .collect(),
                        )
                    })
            })
            .catch(format!("Could not locate dependency: {}", name))
    }
}
