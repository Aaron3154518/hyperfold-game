use std::path::PathBuf;

use parser::{
    parse::ast_crate::Crate, resolve::ast_items::ItemsCrate, util::end,
    validate::ast_validate::ItemData,
};

pub fn main() {
    // TODO: hardcoded
    let (crates, paths) = Crate::parse(PathBuf::from("../test/a"));

    let mut items = crates[..end(&crates, 1)]
        .iter()
        .map(|cr| {
            let mut ic = ItemsCrate::new();
            ic.parse_crate(cr, &paths, &crates);
            // Remove macros crate as crate dependency
            if let Some(i) = ic
                .dependencies
                .iter()
                .position(|d| d.cr_idx == crates.len() - 1)
            {
                ic.dependencies.swap_remove(i);
            }
            ic
        })
        .collect::<Vec<_>>();

    let data = ItemData::validate(&paths, &mut items);

    eprintln!("{data:#?}");

    data.write_to_file();
}
