use crate::{ast_crate::Crate, ast_mod::Mod};

pub trait Visited {
    fn visit<V>(&mut self, vis: &mut V)
    where
        V: Visitor + ?Sized;
}

pub trait Visitor {
    fn visit_crates(&mut self, v: &mut Vec<Crate>) {
        v.visit(self)
    }

    fn visit_crate(&mut self, v: &mut Crate) {
        v.visit(self)
    }

    fn visit_mod(&mut self, v: &mut Mod) {}
}
