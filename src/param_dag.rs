use std::{
    any::TypeId,
    collections::{hash_map::DefaultHasher, VecDeque},
    hash::{Hash, Hasher},
    iter::{once, Rev},
    marker::PhantomData,
    slice::Iter,
    vec::IntoIter,
};

use hyperfold_engine::utils::{number::Number, util::get_time};
use itertools::Itertools;

// Node Structs
type NodeId = u64;

struct RootValue<T> {
    id: NodeId,
    value: T,
    updated: u32,
}

impl<T: 'static> RootValue<T> {
    pub fn new(id: NodeId, value: T) -> Self {
        Self {
            id,
            value,
            updated: get_time(),
        }
    }
}

pub trait UpdateFunc<T, const A: usize, const B: usize> = Fn([&T; A], [&T; B]) -> T;

struct UpdateData<T, const A: usize, const B: usize, F: UpdateFunc<T, A, B>> {
    roots: [usize; A],
    nodes: [usize; B],
    func: F,
    pd: PhantomData<T>,
}

pub trait Update<T> {
    fn update(&self, dag: &Dag<T>, updated: u32) -> Option<T>;

    fn get_roots(&self) -> Iter<usize>;

    fn get_nodes(&self) -> Iter<usize>;

    fn get_nodes_desc(&self) -> Rev<IntoIter<usize>> {
        self.get_nodes().copied().sorted().rev()
    }

    fn update_nodes(&mut self, idxs: &Vec<usize>);
}

impl<T, const A: usize, const B: usize, F: UpdateFunc<T, A, B>> Update<T>
    for UpdateData<T, A, B, F>
{
    fn update(&self, dag: &Dag<T>, updated: u32) -> Option<T> {
        let mut update = false;
        let roots = self.roots.clone().map(|i| {
            let r = &dag.roots[i];
            update = update || r.updated > updated;
            &r.value
        });
        let nodes = self.nodes.clone().map(|i| {
            let n = &dag.nodes[i];
            update = update || n.updated > updated;
            &n.value
        });
        update.then(|| (self.func)(roots, nodes))
    }

    fn get_roots(&self) -> Iter<usize> {
        self.roots.iter()
    }

    fn get_nodes(&self) -> Iter<usize> {
        self.nodes.iter()
    }

    fn update_nodes(&mut self, idxs: &Vec<usize>) {
        for i in &mut self.nodes {
            *i = idxs[*i];
        }
    }
}

struct NodeValue<T> {
    id: NodeId,
    depth: usize,
    value: T,
    update: Box<dyn Update<T>>,
    updated: u32,
}

impl<T: Default + 'static> NodeValue<T> {
    // Node should be default constructed if depended on (but not if searched for)
    pub fn default(id: NodeId) -> Self {
        Self {
            id,
            depth: 0,
            value: Default::default(),
            update: Box::new(UpdateData {
                roots: [],
                nodes: [],
                func: |[], []| unimplemented!(),
                pd: PhantomData,
            }),
            updated: 0,
        }
    }
}

// Node Traits
fn id(type_id: TypeId, idx: u8) -> NodeId {
    let mut hasher = DefaultHasher::new();
    type_id.hash(&mut hasher);
    idx.hash(&mut hasher);
    hasher.finish()
}

// NodeTrait + Node or Root are the only traits that need to be implemented
pub trait NodeTrait: 'static {
    fn idx(&self) -> u8;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn id(&self) -> NodeId {
        id(self.type_id(), self.idx())
    }
}

pub trait Node<T>: NodeTrait {}

pub trait Root<T>: NodeTrait {
    fn default(&self) -> T;
}

// Used to distinctify NodeDefault and Observe traits
pub struct NodeMarker;
pub struct RootMarker;

// Used to convert the Node/Root into an Optional default
pub trait NodeDefault<T, U>: NodeTrait {
    fn default(&self) -> Option<T> {
        None
    }
}

impl<T, U: Node<T>> NodeDefault<T, NodeMarker> for U {}

impl<T, U: Root<T>> NodeDefault<T, RootMarker> for U {
    fn default(&self) -> Option<T> {
        Some(Root::<T>::default(self))
    }
}

// Dag
pub struct Dag<T> {
    roots: Vec<RootValue<T>>,
    nodes: Vec<NodeValue<T>>,
}

impl<T: 'static> Dag<T> {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn find_node(&self, id: NodeId) -> Option<(usize, &NodeValue<T>)> {
        self.nodes.iter().enumerate().find(|(_, n)| n.id == id)
    }

    fn find_root(&self, id: NodeId) -> Option<(usize, &RootValue<T>)> {
        self.roots.iter().enumerate().find(|(_, r)| r.id == id)
    }

    fn get_node_impl(&mut self, id: NodeId) -> &NodeValue<T> {
        // Collect all indices in descending order
        let (idx, _) = self.find_node(id).expect("Node does not exist");
        let mut idxs: VecDeque<_> = once(idx).collect();
        let mut i = 0;
        while i < idxs.len() {
            let n_i = idxs[i];
            let mut j = 0;
            // Add this node's dependencies
            for n2_i in self.nodes[n_i].update.get_nodes_desc() {
                while j < idxs.len() && idxs[j] > n2_i {
                    j += 1;
                }
                if j >= idxs.len() || idxs[j] != n2_i {
                    idxs.push_back(n2_i);
                }
            }
            i += 1;
        }

        // Iterate over dependencies and update if necessary
        let t = get_time();
        for i in idxs.iter().rev() {
            let node = &self.nodes[*i];
            if let Some(val) = node.update.update(self, node.updated) {
                self.nodes[*i].value = val;
            }
            self.nodes[*i].updated = t;
        }

        &self.nodes[idx]
    }

    fn get_root_impl(&mut self, id: NodeId, default: T) -> &RootValue<T> {
        let idx = match self.roots.iter().position(|r| r.id == id) {
            Some(i) => i,
            None => {
                self.roots.push(RootValue {
                    id,
                    value: default,
                    updated: get_time(),
                });
                self.roots.len() - 1
            }
        };
        &self.roots[idx]
    }

    pub fn get<U>(&mut self, n: impl NodeDefault<T, U>) -> &T {
        match n.default() {
            Some(t) => &self.get_root_impl(n.id(), t).value,
            None => &self.get_node_impl(n.id()).value,
        }
    }

    pub fn set(&mut self, r: impl Root<T>, value: T) {
        let id = r.id();
        match self.roots.iter_mut().find(|r| r.id == id) {
            Some(r) => {
                r.value = value;
                r.updated = get_time();
            }
            None => self.roots.push(RootValue {
                id,
                value,
                updated: get_time(),
            }),
        }
    }

    pub fn update(&mut self, r: impl Root<T>, f: impl FnOnce(&T) -> T) {
        let val = f(&self.get_root_impl(r.id(), r.default()).value);
        self.set(r, val)
    }

    pub fn add_root(&mut self, r: impl Root<T>) {
        let value = r.default();
        self.set(r, value)
    }
}

impl<T: Default + 'static> Dag<T> {
    pub fn add_node<const A: usize, const B: usize>(
        &mut self,
        n: impl Node<T>,
        roots: [&dyn Root<T>; A],
        nodes: [&dyn Node<T>; B],
        func: impl UpdateFunc<T, A, B> + 'static,
    ) {
        let roots = roots.map(|r| match self.find_root(r.id()) {
            Some((i, _)) => i,
            None => {
                self.roots.push(RootValue::new(r.id(), r.default()));
                self.roots.len() - 1
            }
        });
        let nodes = nodes.map(|n| match self.find_node(n.id()) {
            Some((i, _)) => i,
            None => {
                self.nodes.push(NodeValue::default(n.id()));
                self.nodes.len() - 1
            }
        });

        // Insert/update the node
        let id = n.id();
        let depth = nodes.iter().fold(0, |m, idx| m.max(self.nodes[*idx].depth));
        let update = Box::new(UpdateData {
            roots,
            nodes,
            func,
            pd: PhantomData,
        });
        let prev_depth = match self.nodes.iter_mut().find(|n| n.id == id) {
            Some(n) => {
                n.update = update;
                n.updated = 0;
                std::mem::replace(&mut n.depth, depth)
            }
            None => {
                self.nodes.push(NodeValue {
                    id,
                    depth,
                    value: Default::default(),
                    update,
                    updated: 0,
                });
                0
            }
        };

        // Update node depth and re-sort by depth
        let mut depths: Vec<_> = self.nodes.iter().map(|n| n.depth).collect();
        for (i, n) in self
            .nodes
            .iter_mut()
            .enumerate()
            .filter(|(_, n)| n.depth > prev_depth)
        {
            let nodes = n.update.get_nodes();
            let depth = nodes.fold(0, |m, idx| m.max(depths[*idx]));
            if depth != n.depth {
                n.depth = depth;
                depths[i] = depth;
            }
        }
        // Get new indices for each node
        let idxs: Vec<_> = (self.nodes.iter().enumerate())
            .map(|(i, n)| (i, (n.depth, n.id)))
            .sorted_by_key(|(_, key)| *key)
            .map(|(i, _)| i)
            .collect();
        // Update stored node indices
        self.nodes.sort_by_key(|n| (n.depth, n.id));
        for n in &mut self.nodes {
            n.update.update_nodes(&idxs);
        }
    }
}

#[hyperfold_engine::global]
struct NumDag(pub Dag<Number>);

impl NumDag {
    pub fn new() -> Self {
        Self(Dag::new())
    }
}

#[macro_export]
macro_rules! parameters {
    (@def $name: ident) => {
        #[derive(Copy, Clone)]
        pub struct $name;

        impl NodeTrait for $name {
            fn idx(&self) -> u32 {
                0
            }
        }
    };

    (@def $name: ident ($($v: ident),+)) => {
        #[derive(Copy, Clone)]
        pub enum $name {
            $($v),*
        }

        impl NodeTrait for $name {
            fn idx(&self) -> u8 {
                *self as u8
            }
        }
    };

    ($name: ident) => {
        $crate::parameters!(@def $name);
        impl Node<Number> for $name {}
    };

    ($name: ident ($($v: ident),+)) => {
        $crate::parameters!(@def $name ($($v),*));
        impl Node<Number> for $name {}
    };

    ($name: ident = $v: literal) => {
        $crate::parameters!(@def $name);
        impl Root<Number> for $name {
            fn default(&self) -> Number {
                $v.into()
            }
        }
    };

    ($name: ident ($($v: ident = $n: literal),+)) => {
        $crate::parameters!(@def $name ($($v),*));
        impl Root<Number> for $name {
            fn default(&self) -> Number {
                match self {
                    $($name::$v => $n),*
                }.into()
            }
        }
    };
}

// Observers
pub struct Observer<T> {
    checked: u32,
    id: NodeId,
    // Some = root, None = node
    default: Option<T>,
}

impl<T: 'static> Observer<T> {
    fn new(id: NodeId, default: Option<T>) -> Self {
        Self {
            checked: 0,
            id,
            default,
        }
    }

    pub fn check(&mut self, dag: &mut Dag<T>, f: impl FnOnce(&T))
    where
        T: Clone,
    {
        let (updated, value) = match &self.default {
            Some(t) => {
                let r = dag.get_root_impl(self.id, t.clone());
                (r.updated, &r.value)
            }
            None => {
                let n = dag.get_node_impl(self.id);
                (n.updated, &n.value)
            }
        };
        if self.checked < updated {
            self.checked = get_time();
            f(value);
        }
    }
}

pub trait Observe<T, U> {
    fn observe(&self) -> Observer<T>;
}

impl<T: 'static, N: Node<T>> Observe<T, NodeMarker> for N {
    fn observe(&self) -> Observer<T> {
        Observer::new(self.id(), None)
    }
}

impl<T: 'static, R: Root<T>> Observe<T, RootMarker> for R {
    fn observe(&self) -> Observer<T> {
        Observer::new(self.id(), Some(self.default()))
    }
}

#[macro_export]
macro_rules! observers {
    ($name: ident <$ty: ty> { $($var: ident = $node: expr),* }) => {
        pub struct $name {
            $(pub $var: Observer<$ty>),*
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $($var: $node.observe()),*
                }
            }
        }
    };
}
