use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::{
    parse::ast_attrs::{get_attributes_if_active, Attribute, EcsAttribute},
    util::{end, Catch},
};

use super::ast_fn_arg::FnArg;

#[derive(Debug)]
pub enum MarkType {
    Struct,
    Fn { args: Vec<FnArg> },
}

// TODO: Cfg features
#[derive(Debug)]
pub struct MarkedItem {
    pub ty: MarkType,
    pub sym: Symbol,
    pub attrs: Vec<(Vec<String>, Vec<String>)>,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub ident: String,
    // Use stmts: Includes ident/alias
    // Other items: Doesn't include ident
    pub path: Vec<String>,
    pub public: bool,
}

impl Symbol {
    pub fn from(mut path: Vec<String>, ident: &syn::Ident, vis: &syn::Visibility) -> Self {
        path.push(ident.to_string());
        // Add to the symbol table
        Self {
            ident: ident.to_string(),
            path,
            public: match vis {
                syn::Visibility::Public(_) => true,
                syn::Visibility::Restricted(_) | syn::Visibility::Inherited => false,
            },
        }
    }
}

#[derive(Debug)]
pub enum ModType {
    Main,
    Lib,
    Mod,
    File,
    Internal,
}

#[derive(Debug)]
pub struct Mod {
    pub ty: ModType,
    pub dir: PathBuf,
    pub path: Vec<String>,
    pub mods: Vec<Mod>,
    pub symbols: Vec<Symbol>,
    pub uses: Vec<Symbol>,
    pub marked: Vec<MarkedItem>,
}

// TODO: ignore private mods
// Pass 1: parsing
impl Mod {
    pub fn new(dir: PathBuf, path: Vec<String>, ty: ModType) -> Self {
        Self {
            ty,
            dir,
            path,
            mods: Vec::new(),
            symbols: Vec::new(),
            uses: Vec::new(),
            marked: Vec::new(),
        }
    }

    pub fn parse(&mut self, path: PathBuf) {
        let file_contents = fs::read_to_string(path.to_owned())
            .catch(format!("Failed to read file: {}", path.display()));
        let ast = syn::parse_file(&file_contents).catch(format!(
            "Failed to parse file contents of: {}",
            path.display()
        ));
        self.visit_file(&ast);
    }
}

// File/items
impl Mod {
    pub fn visit_file(&mut self, i: &syn::File) {
        self.visit_items(&i.items);
    }

    fn visit_items(&mut self, i: &Vec<syn::Item>) {
        i.iter().for_each(|i| self.visit_item(i))
    }

    fn visit_item(&mut self, i: &syn::Item) {
        // Match once to add to symbol table
        match i {
            // Use statements need to be parsed
            syn::Item::Use(i) => None,
            // Add to symbol table
            syn::Item::Fn(syn::ItemFn {
                sig: syn::Signature { ident, .. },
                vis,
                ..
            })
            | syn::Item::Mod(syn::ItemMod { ident, vis, .. })
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
        }
        .map(|(ident, vis)| {
            self.symbols
                .push(Symbol::from(self.path.to_vec(), ident, vis))
        });

        // Match again to parse
        match i {
            syn::Item::Mod(i) => self.visit_item_mod(i),
            syn::Item::Use(i) => self.visit_item_use(i),
            syn::Item::Fn(i) => self.visit_item_fn(i),
            syn::Item::Struct(i) => self.visit_item_struct(i),
            _ => (),
        }
    }

    // Mod
    fn visit_item_mod(&mut self, i: &syn::ItemMod) {
        if let Some(attrs) = get_attributes_if_active(&i.attrs, &self.path, &Vec::new()) {
            match &i.content {
                // Parse inner mod
                Some((_, items)) => {
                    let mut new_mod = Self::new(
                        self.dir.to_owned(),
                        [self.path.to_vec(), vec![i.ident.to_string()]].concat(),
                        ModType::Internal,
                    );
                    new_mod.visit_items(items);
                    self.mods.push(new_mod);
                }
                // Parse file mod
                None => {
                    self.mods.push(Self::parse_mod(
                        self.dir.join(i.ident.to_string()),
                        &[self.path.to_vec(), vec![i.ident.to_string()]].concat(),
                    ));
                }
            }
        }
    }

    // Components
    fn visit_item_struct(&mut self, i: &syn::ItemStruct) {
        if let Some(attrs) = get_attributes_if_active(&i.attrs, &self.path, &Vec::new()) {
            if !attrs.is_empty() {
                self.marked.push(MarkedItem {
                    ty: MarkType::Struct,
                    sym: Symbol::from(self.path.to_vec(), &i.ident, &i.vis),
                    attrs,
                });
            }
        }
    }

    // Systems
    fn visit_item_fn(&mut self, i: &syn::ItemFn) {
        if let Some(attrs) = get_attributes_if_active(&i.attrs, &self.path, &Vec::new()) {
            if !attrs.is_empty() {
                self.marked.push(MarkedItem {
                    ty: MarkType::Fn {
                        args: i
                            .sig
                            .inputs
                            .iter()
                            .filter_map(|arg| match arg {
                                syn::FnArg::Typed(t) => {
                                    let mut sys_arg = FnArg::new();
                                    sys_arg.parse_arg(0, &self.path, &t);
                                    Some(sys_arg)
                                }
                                syn::FnArg::Receiver(_) => None,
                            })
                            .collect(),
                    },
                    sym: Symbol::from(self.path.to_vec(), &i.sig.ident, &i.vis),
                    attrs,
                });
            }
        }
    }
}

// TODO: Ambiguous paths, :: paths

// Use paths
impl Mod {
    fn visit_item_use(&mut self, i: &syn::ItemUse) {
        if let Some(attrs) = get_attributes_if_active(&i.attrs, &self.path, &Vec::new()) {
            let mut uses = self.visit_use_tree(&i.tree, &mut Vec::new(), Vec::new());
            let public = match i.vis {
                syn::Visibility::Public(_) => true,
                syn::Visibility::Restricted(_) | syn::Visibility::Inherited => false,
            };
            uses.iter_mut().for_each(|u| u.public = public);
            self.uses.append(&mut uses);
        }
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
        if i.ident == "super" {
            if path.is_empty() {
                *path = self.path[..end(&self.path, 1)].to_vec()
            } else {
                path.pop();
            }
        } else {
            path.push(i.ident.to_string())
        }
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
            path: [path.to_owned(), vec![i.ident.to_string()]].concat(),
            public: false,
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
            path: [path.to_owned(), vec![i.ident.to_string()]].concat(),
            public: false,
        });
        items
    }

    fn visit_use_glob(
        &mut self,
        i: &syn::UseGlob,
        path: &mut Vec<String>,
        mut items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        items.push(Symbol {
            ident: "*".to_string(),
            path: path.to_owned(),
            public: false,
        });
        items
    }

    fn visit_use_group(
        &mut self,
        i: &syn::UseGroup,
        path: &mut Vec<String>,
        items: Vec<Symbol>,
    ) -> Vec<Symbol> {
        i.items.iter().fold(items, |items, i| {
            self.visit_use_tree(i, &mut path.to_vec(), items)
        })
    }
}
