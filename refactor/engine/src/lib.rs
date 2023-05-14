use std::marker::PhantomData;

// The path resolver can't find the macro paths in "macros" so they will be labelled under "engine"
pub use macros::*;

game_crate!();

// Important globals
#[global]
#[derive(PartialEq, Eq, Hash)]
pub struct Entity;

impl Entity {
    pub fn new() -> Self {
        Self
    }
}

#[global]
pub struct EntityTrash(Vec<Entity>);

impl EntityTrash {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

#[global]
pub struct Event;

impl Event {
    pub fn new() -> Self {
        Self
    }
}

#[global]
pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> Self {
        Self
    }
}

#[macros::global]
pub struct Screen;

impl Screen {
    pub fn new() -> Self {
        Self
    }
}

#[macros::global]
pub struct Camera;

impl Camera {
    pub fn new() -> Self {
        Self
    }
}

pub type Container<T> = Vec<T>;
pub type Label<T> = PhantomData<T>;
pub type AndLabels<T> = PhantomData<T>;
pub type OrLabels<T> = PhantomData<T>;
pub type NandLabels<T> = PhantomData<T>;
pub type NorLabels<T> = PhantomData<T>;

pub trait AddEvent<T> {
    fn new_event(&mut self, t: T);

    fn get_event<'a>(&'a self) -> Option<&'a T>;
}

pub trait AddComponent<T> {
    fn add_component(&mut self, e: Entity, t: T);
}
