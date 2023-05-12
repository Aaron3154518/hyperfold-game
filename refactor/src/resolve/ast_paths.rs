use super::ast_resolve::Path;

#[derive(Clone, Copy, Debug)]
pub enum EnginePaths {
    AddComponent,
    AddEvent,
    // TODO: Only need path from crate
    Entity,
}

pub const NUM_ENGINE_PATHS: usize = 3;

impl EnginePaths {
    pub fn get_variants() -> [EnginePaths; NUM_ENGINE_PATHS] {
        [
            EnginePaths::AddComponent,
            EnginePaths::AddEvent,
            EnginePaths::Entity,
        ]
    }

    pub fn get_type(&self) -> String {
        match self {
            EnginePaths::AddComponent => "AddComponent",
            EnginePaths::AddEvent => "AddEvent",
            EnginePaths::Entity => "Entity",
        }
        .to_string()
    }

    pub fn get_path(&self) -> Vec<String> {
        vec!["crate".to_string(), self.get_type()]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

// TODO: easy way to declare hardcoded
pub struct Paths {
    // Macro paths
    pub component: Path,
    pub global: Path,
    pub event: Path,
    pub system: Path,
    // Struct paths
    pub eid: Path,
    pub container: Path,
    pub label: Path,
    pub and_labels: Path,
    pub or_labels: Path,
    pub nand_labels: Path,
    pub nor_labels: Path,
    // Trait paths
    pub engine_paths: [Path; NUM_ENGINE_PATHS],
}

impl Paths {
    pub fn new(engine_cr_idx: usize) -> Self {
        Self {
            component: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "component".to_string()],
            },
            global: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "global".to_string()],
            },
            event: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "event".to_string()],
            },
            system: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "system".to_string()],
            },
            eid: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "Entity".to_string()],
            },
            container: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "Container".to_string()],
            },
            label: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "Label".to_string()],
            },
            and_labels: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "AndLabels".to_string()],
            },
            or_labels: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "OrLabels".to_string()],
            },
            nand_labels: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "NandLabels".to_string()],
            },
            nor_labels: Path {
                cr_idx: engine_cr_idx,
                path: vec!["crate".to_string(), "NorLabels".to_string()],
            },
            engine_paths: EnginePaths::get_variants().map(|ep| Path {
                cr_idx: engine_cr_idx,
                path: ep.get_path(),
            }),
        }
    }
}
