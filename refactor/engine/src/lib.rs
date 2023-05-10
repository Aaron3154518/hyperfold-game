use std::marker::PhantomData;

// The path resolver can't find the macro paths in "macros" so they will be labelled under "engine"
pub use macros::*;

pub struct Entity;

pub type Container<T> = PhantomData<T>;
pub type Label<T> = PhantomData<T>;
pub type AndLabels<T> = PhantomData<T>;
pub type OrLabels<T> = PhantomData<T>;
pub type NandLabels<T> = PhantomData<T>;
pub type NorLabels<T> = PhantomData<T>;
