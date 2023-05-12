use super::ast_resolve::Path;

#[macros::expand_enum]
#[derive(Clone, Copy, Debug)]
pub enum EnginePaths {
    // Trait paths
    AddComponent,
    AddEvent,
    // Struct paths
    Entity,
    EntityTrash,
    Container,
    Label,
    AndLabels,
    OrLabels,
    NandLabels,
    NorLabels,
    // Macro paths
    MacroComponent,
    MacroGlobal,
    MacroEvent,
    MacroSystem,
}

impl EnginePaths {
    pub fn as_str(&self) -> &str {
        match self {
            // Traits
            EnginePaths::AddComponent => "AddComponent",
            EnginePaths::AddEvent => "AddEvent",
            // Structs
            EnginePaths::Entity => "Entity",
            EnginePaths::EntityTrash => "EntityTrash",
            EnginePaths::Container => "Container",
            EnginePaths::Label => "Label",
            EnginePaths::AndLabels => "AndLabels",
            EnginePaths::OrLabels => "OrLabels",
            EnginePaths::NandLabels => "NandLabels",
            EnginePaths::NorLabels => "NorLabels",
            // Macros
            EnginePaths::MacroComponent => "component",
            EnginePaths::MacroGlobal => "global",
            EnginePaths::MacroEvent => "event",
            EnginePaths::MacroSystem => "system",
        }
    }

    pub fn get_path(&self) -> Vec<String> {
        vec!["crate", self.as_str()]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn to_paths(engine_cr_idx: usize) -> [Path; Self::len()] {
        Self::variants().map(|ep| Path {
            cr_idx: engine_cr_idx,
            path: ep.get_path(),
        })
    }
}

// Macros for defining subsets of engine paths
macro_rules! engine_paths {
    ($n: ident, $($vs: ident),*) => {
        #[macros::expand_enum]
        #[derive(Clone, Copy, Debug)]
        pub enum $n {
            $($vs),*
        }

        impl $n {
            pub fn map(&self) -> EnginePaths {
                match self {
                    $(Self::$vs => EnginePaths::$vs),*
                }
            }

            pub fn get_paths(paths: &[Path; EnginePaths::len()]) -> [Path; Self::len()] {
                Self::variants().map(|v| paths[v.map() as usize].to_owned())
            }

            pub fn to_paths(engine_cr_idx: usize) -> [Path; Self::len()] {
                Self::get_paths(&EnginePaths::to_paths(engine_cr_idx))
            }
        }
    };
}

// Paths needed by all crates
engine_paths!(CrateEnginePaths, AddComponent, AddEvent);

// Paths needed by only entry crate
engine_paths!(EntryEnginePaths, Entity, EntityTrash);
