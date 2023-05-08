use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::util::Expect;

#[derive(Debug)]
pub struct Mod {
    path: Vec<String>,
    mods: Vec<Mod>,
    neighbors: Vec<String>,
    pub_symbols: Vec<String>,
    pri_symbols: Vec<String>,
}

impl Mod {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            mods: Vec::new(),
            neighbors: Vec::new(),
            pub_symbols: Vec::new(),
            pri_symbols: Vec::new(),
        }
    }

    pub fn visit_items(&mut self, i: &Vec<syn::Item>) {
        i.iter().for_each(|i| self.visit_item(i))
    }

    pub fn get_neighbors(&self) -> Vec<String> {
        self.neighbors.to_vec()
    }
}

impl<'ast> syn::visit::Visit<'ast> for Mod {
    fn visit_file(&mut self, i: &'ast syn::File) {
        self.visit_items(&i.items)
    }

    fn visit_item(&mut self, i: &'ast syn::Item) {
        // Match once to add to symbol table
        if let Some((ident, vis)) = match i {
            // TODO: uses
            syn::Item::Use(i) => None,
            syn::Item::Fn(i) => Some((&i.sig.ident, &i.vis)),
            syn::Item::Mod(syn::ItemMod { ident, vis, .. })
            | syn::Item::Enum(syn::ItemEnum { ident, vis, .. })
            | syn::Item::Struct(syn::ItemStruct { ident, vis, .. })
            | syn::Item::Const(syn::ItemConst { ident, vis, .. })
            | syn::Item::ExternCrate(syn::ItemExternCrate { ident, vis, .. })
            | syn::Item::Static(syn::ItemStatic { ident, vis, .. })
            | syn::Item::Trait(syn::ItemTrait { ident, vis, .. })
            | syn::Item::TraitAlias(syn::ItemTraitAlias { ident, vis, .. })
            | syn::Item::Type(syn::ItemType { ident, vis, .. })
            | syn::Item::Union(syn::ItemUnion { ident, vis, .. }) => Some((ident, vis)),
            // Ignore completely
            syn::Item::ForeignMod(..)
            | syn::Item::Impl(..)
            | syn::Item::Macro(..)
            | syn::Item::Verbatim(..)
            | _ => None,
        } {
            // Add to the symbol table
            match vis {
                syn::Visibility::Public(_) => &mut self.pub_symbols,
                syn::Visibility::Restricted(_) | syn::Visibility::Inherited => {
                    &mut self.pri_symbols
                }
            }
            .push(ident.to_string())
        }

        // Match again to parse
        match i {
            syn::Item::Mod(i) => self.visit_item_mod(i),
            syn::Item::Use(i) => (),
            syn::Item::Fn(i) => (),
            syn::Item::Enum(i) => (),
            syn::Item::Struct(i) => (),
            _ => (),
        }
    }

    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        match &i.content {
            // Parse inner mod
            Some((_, items)) => {
                let mut new_mod = Mod::new();
                new_mod.visit_items(items);
                self.mods.push(new_mod);
            }
            // Parse file mod
            None => {
                self.neighbors.push(i.ident.to_string());
            }
        }
    }
}
