use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::util::Expect;

#[derive(Debug)]
pub struct Mod {
    path: Vec<String>,
    mods: Vec<Mod>,
    pub_symbols: Vec<String>,
    pri_symbols: Vec<String>,
}

impl Mod {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            mods: Vec::new(),
            pub_symbols: Vec::new(),
            pri_symbols: Vec::new(),
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for Mod {
    fn visit_file(&mut self, i: &'ast syn::File) {
        // Iter once to add to symbol table
        for i in i.items.iter() {
            match i {
                syn::Item::Mod(i) => {}
                syn::Item::Use(i) => (),
                syn::Item::Fn(i) => (),
                syn::Item::Enum(i) => (),
                syn::Item::Struct(i) => (),
                // Add to the symbol table and ignore
                syn::Item::Const(syn::ItemConst { ident, vis, .. })
                | syn::Item::ExternCrate(syn::ItemExternCrate { ident, vis, .. })
                | syn::Item::Static(syn::ItemStatic { ident, vis, .. })
                | syn::Item::Trait(syn::ItemTrait { ident, vis, .. })
                | syn::Item::TraitAlias(syn::ItemTraitAlias { ident, vis, .. })
                | syn::Item::Type(syn::ItemType { ident, vis, .. })
                | syn::Item::Union(syn::ItemUnion { ident, vis, .. }) => match vis {
                    syn::Visibility::Public(_) => &mut self.pub_symbols,
                    syn::Visibility::Restricted(_) | syn::Visibility::Inherited => {
                        &mut self.pri_symbols
                    }
                }
                .push(ident.to_string()),
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
