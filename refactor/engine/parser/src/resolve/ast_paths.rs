use crate::{codegen::idents::Idents, util::JoinMap};

use super::ast_resolve::Path;

pub trait ExpandEnum<const N: usize>
where
    Self: Sized,
{
    const LEN: usize = N;
    const VARIANTS: [Self; N];
}

pub trait GetPaths<const N: usize>: ExpandEnum<N> {
    fn as_str(&self) -> &str;

    fn get_path(&self) -> Vec<String> {
        vec!["crate", self.as_str()].map_vec(|s| s.to_string())
    }

    fn to_paths(cr_idx: usize) -> [Path; N] {
        Self::VARIANTS.map(|v| Path {
            cr_idx,
            path: v.get_path(),
        })
    }
}

#[shared::macros::expand_enum]
pub enum MacroPaths {
    Component,
    Global,
    Event,
    System,
}

impl GetPaths<{ Self::LEN }> for MacroPaths {
    fn as_str(&self) -> &str {
        match self {
            MacroPaths::Component => "component",
            MacroPaths::Global => "global",
            MacroPaths::Event => "event",
            MacroPaths::System => "system",
        }
    }
}

#[shared::macros::expand_enum]
pub enum EngineTraits {
    AddComponent,
    AddEvent,
}

impl GetPaths<{ Self::LEN }> for EngineTraits {
    fn as_str(&self) -> &str {
        match self {
            EngineTraits::AddComponent => "AddComponent",
            EngineTraits::AddEvent => "AddEvent",
        }
    }
}

#[shared::macros::expand_enum]
pub enum EngineGlobals {
    CFoo,
    EFoo,
    Entity,
    EntityTrash,
    Event,
    RenderSystem,
    Camera,
    Screen,
}

impl GetPaths<{ Self::LEN }> for EngineGlobals {
    fn as_str(&self) -> &str {
        match self {
            EngineGlobals::CFoo => Idents::CFoo.as_str(),
            EngineGlobals::EFoo => Idents::EFoo.as_str(),
            EngineGlobals::Entity => "Entity",
            EngineGlobals::EntityTrash => "EntityTrash",
            EngineGlobals::Event => "Event",
            EngineGlobals::RenderSystem => "RenderSystem",
            EngineGlobals::Camera => "Camera",
            EngineGlobals::Screen => "Screen",
        }
    }

    fn to_paths(cr_idx: usize) -> [Path; Self::LEN] {
        Self::VARIANTS.map(|v| Path {
            cr_idx: match v {
                EngineGlobals::CFoo | EngineGlobals::EFoo => 0,
                _ => cr_idx,
            },
            path: v.get_path(),
        })
    }
}

#[shared::macros::expand_enum]
pub enum EngineContainers {
    Container,
    Label,
    AndLabels,
    OrLabels,
    NandLabels,
    NorLabels,
}

impl GetPaths<{ Self::LEN }> for EngineContainers {
    fn as_str(&self) -> &str {
        match self {
            EngineContainers::Container => "Container",
            EngineContainers::Label => "Label",
            EngineContainers::AndLabels => "AndLabels",
            EngineContainers::OrLabels => "OrLabels",
            EngineContainers::NandLabels => "NandLabels",
            EngineContainers::NorLabels => "NorLabels",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Paths {
    pub macros: [Path; MacroPaths::LEN],
    pub traits: [Path; EngineTraits::LEN],
    pub globals: [Path; EngineGlobals::LEN],
    pub containers: [Path; EngineContainers::LEN],
}

impl Paths {
    pub fn new(engine_cr_idx: usize, macros_cr_idx: usize) -> Self {
        Self {
            macros: MacroPaths::to_paths(macros_cr_idx),
            traits: EngineTraits::to_paths(engine_cr_idx),
            globals: EngineGlobals::to_paths(engine_cr_idx),
            containers: EngineContainers::to_paths(engine_cr_idx),
        }
    }

    pub fn get_macro(&self, i: MacroPaths) -> &Path {
        &self.macros[i as usize]
    }

    pub fn get_trait(&self, i: EngineTraits) -> &Path {
        &self.traits[i as usize]
    }

    pub fn get_global(&self, i: EngineGlobals) -> &Path {
        &self.globals[i as usize]
    }

    pub fn get_container(&self, i: EngineContainers) -> &Path {
        &self.containers[i as usize]
    }
}
