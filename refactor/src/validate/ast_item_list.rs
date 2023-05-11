use crate::resolve::ast_items::{Component, Event, Global, ItemsCrate};

// This struct is used to modify the lists of items
// Traits: hardcodes traits
#[derive(Debug)]
pub struct ItemList<'a> {
    pub components: Vec<Vec<&'a Component>>,
    pub globals: Vec<Vec<&'a Global>>,
    pub events: Vec<Vec<&'a Event>>,
    pub traits: Vec<&'a Global>,
}

impl<'a> ItemList<'a> {
    pub fn from(crates: &'a Vec<ItemsCrate>, traits: &'a Vec<Global>) -> Self {
        let (mut components, mut globals, mut events) = (Vec::new(), Vec::new(), Vec::new());
        for cr in crates.iter() {
            components.push(cr.components.iter().collect());
            if cr.cr_idx == 0 {
                let mut v: Vec<&Global> = traits.iter().collect();
                v.extend(cr.globals.iter());
                globals.push(v);
            } else {
                globals.push(cr.globals.iter().collect());
            }
            events.push(cr.events.iter().collect());
        }

        Self {
            components,
            globals,
            events,
            traits: traits.iter().collect(),
        }
    }
}
