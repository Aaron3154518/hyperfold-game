use std::{fs, path::PathBuf};

use syn::visit::Visit;

use crate::util::{end, Expect};

#[derive(Debug)]
pub struct Symbol {
    pub ident: String,
    pub path: Vec<String>,
    pub public: bool,
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
}

impl Mod {
    pub fn new(dir: PathBuf, path: Vec<String>, ty: ModType) -> Self {
        Self {
            ty,
            dir,
            path,
            mods: Vec::new(),
            symbols: Vec::new(),
            uses: Vec::new(),
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

    // File/items
    pub fn visit_file(&mut self, i: &syn::File) {
        self.visit_items(&i.items);
    }

    fn visit_items(&mut self, i: &Vec<syn::Item>) {
        i.iter().for_each(|i| self.visit_item(i))
    }

    fn visit_item(&mut self, i: &syn::Item) {
        // Match once to add to symbol table
        if let Some((ident, vis)) = match i {
            // Use statements need to be parsed
            syn::Item::Use(i) => None,
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

            // Ignore completely
            syn::Item::ForeignMod(..)
            | syn::Item::Impl(..)
            | syn::Item::Macro(..)
            | syn::Item::Verbatim(..)
            | _ => None,
        } {
            // Add to the symbol table
            self.symbols.push(Symbol {
                ident: ident.to_string(),
                path: [self.path.to_owned(), vec![ident.to_string()]].concat(),
                public: match vis {
                    syn::Visibility::Public(_) => true,
                    syn::Visibility::Restricted(_) | syn::Visibility::Inherited => false,
                },
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

    // Use paths
    fn visit_item_use(&mut self, i: &syn::ItemUse) {
        let mut uses = self.visit_use_tree(&i.tree, &mut Vec::new(), Vec::new());
        let public = match i.vis {
            syn::Visibility::Public(_) => true,
            syn::Visibility::Restricted(_) | syn::Visibility::Inherited => false,
        };
        uses.iter_mut().for_each(|u| u.public = public);
        self.uses.append(&mut uses);
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
        i.items
            .iter()
            .fold(items, |items, i| self.visit_use_tree(i, path, items))
    }
}
