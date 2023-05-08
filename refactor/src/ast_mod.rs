use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::util::Expect;

#[derive(Debug)]
pub struct Mod {
    path: Vec<String>,
    mods: Vec<Mod>,
    symbols: Vec<String>,
}

impl Mod {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            mods: Vec::new(),
            symbols: Vec::new(),
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for Mod {
    fn visit_file(&mut self, i: &'ast syn::File) {
        for i in i.items.iter() {
            match i {
                syn::Item::Mod(i) => (),
                syn::Item::Use(i) => (),
                syn::Item::Fn(i) => (),
                syn::Item::Enum(i) => (),
                syn::Item::Struct(i) => (),
                // Add to the symbol table and ignore
                syn::Item::Const(syn::ItemConst { ident, .. })
                | syn::Item::ExternCrate(syn::ItemExternCrate { ident, .. })
                | syn::Item::Static(syn::ItemStatic { ident, .. })
                | syn::Item::Trait(syn::ItemTrait { ident, .. })
                | syn::Item::TraitAlias(syn::ItemTraitAlias { ident, .. })
                | syn::Item::Type(syn::ItemType { ident, .. })
                | syn::Item::Union(syn::ItemUnion { ident, .. }) => {
                    self.symbols.push(ident.to_string())
                }
                // Ignore completely
                syn::Item::ForeignMod(..)
                | syn::Item::Impl(..)
                | syn::Item::Macro(..)
                | syn::Item::Verbatim(..)
                | _ => (),
            }
        }
    }
}
