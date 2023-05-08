use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::util::Expect;

#[derive(Debug)]
struct Symbol {
    ident: String,
    alias: Vec<String>,
}

#[derive(Debug)]
pub struct Mod {
    path: Vec<String>,
    mods: Vec<Mod>,
    neighbors: Vec<String>,
    pub_symbols: Vec<Symbol>,
    pri_symbols: Vec<Symbol>,
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

    pub fn get_neighbors(&self) -> Vec<String> {
        self.neighbors.to_vec()
    }

    // File/items
    pub fn visit_file(&mut self, i: &syn::File) {
        self.visit_items(&i.items)
    }

    fn visit_items(&mut self, i: &Vec<syn::Item>) {
        i.iter().for_each(|i| self.visit_item(i))
    }

    fn visit_item(&mut self, i: &syn::Item) {
        // Match once to add to symbol table
        if let Some((ident, vis)) = match i {
            // Add to symbol table
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
            // Use statements need to be parsed
            syn::Item::Use(i) => None,
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
            .push(Symbol {
                ident: ident.to_string(),
                alias: Vec::new(),
            })
        }

        // Match again to parse
        match i {
            syn::Item::Mod(i) => self.visit_item_mod(i),
            syn::Item::Use(i) => self.visit_item_use(i),
            syn::Item::Fn(i) => (),
            syn::Item::Enum(i) => (),
            syn::Item::Struct(i) => (),
            _ => (),
        }
    }

    // Mod
    fn visit_item_mod(&mut self, i: &syn::ItemMod) {
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

    // Use paths
    fn visit_item_use(&mut self, i: &syn::ItemUse) {
        let mut uses = self.visit_use_tree(&i.tree, &mut Vec::new(), Vec::new());
        match i.vis {
            syn::Visibility::Public(_) => &mut self.pub_symbols,
            syn::Visibility::Restricted(_) | syn::Visibility::Inherited => &mut self.pri_symbols,
        }
        .append(&mut uses);
    }

    fn visit_use_tree(
        &mut self,
        i: &syn::UseTree,
        path: &mut Vec<String>,
        items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        match &i {
            syn::UseTree::Path(i) => self.visit_use_path(i, path, items),
            syn::UseTree::Name(i) => self.visit_use_name(i, path, items),
            syn::UseTree::Rename(i) => self.visit_use_rename(i, path, items),
            syn::UseTree::Glob(i) => self.visit_use_glob(i, path, items),
            syn::UseTree::Group(i) => self.visit_use_group(i, path, items),
        }
    }

    fn visit_use_path(
        &mut self,
        i: &syn::UsePath,
        path: &mut Vec<String>,
        items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        path.push(i.ident.to_string());
        self.visit_use_tree(&i.tree, path, items)
    }

    fn visit_use_name(
        &mut self,
        i: &syn::UseName,
        path: &mut Vec<String>,
        mut items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        items.push(Symbol {
            ident: i.ident.to_string(),
            alias: [path.to_owned(), vec![i.ident.to_string()]].concat(),
        });
        items
    }

    fn visit_use_rename(
        &mut self,
        i: &syn::UseRename,
        path: &mut Vec<String>,
        mut items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        items.push(Symbol {
            ident: i.rename.to_string(),
            alias: [path.to_owned(), vec![i.ident.to_string()]].concat(),
        });
        items
    }

    fn visit_use_glob(
        &mut self,
        i: &syn::UseGlob,
        path: &mut Vec<String>,
        mut items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        // TODO: wtf
        items.push(Symbol {
            ident: "*".to_string(),
            alias: [path.to_owned(), vec!["*".to_string()]].concat(),
        });
        items
    }

    fn visit_use_group(
        &mut self,
        i: &syn::UseGroup,
        path: &mut Vec<String>,
        items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        i.items
            .iter()
            .fold(items, |items, i| self.visit_use_tree(i, path, items))
    }
}
