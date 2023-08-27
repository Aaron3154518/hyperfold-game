use std::{
    any::TypeId,
    collections::{hash_map::DefaultHasher, VecDeque},
    hash::{Hash, Hasher},
    iter::{once, Rev},
    slice::Iter,
    vec::IntoIter,
};

use hyperfold_engine::utils::util::get_time;
use itertools::Itertools;

// Node Structs
type NodeId = u64;

struct RootValue<T> {
    id: NodeId,
    value: T,
    updated: u32,
}

impl<T: 'static> RootValue<T> {
    pub fn default(r: &dyn Root<T>) -> Self {
        Self {
            id: r.id(),
            value: r.default(),
            updated: get_time(),
        }
    }
}

pub trait UpdateFunc<const A: usize, const B: usize> = Fn([u32; A], [u32; B]) -> u32;

struct UpdateData<const A: usize, const B: usize, F: UpdateFunc<A, B>> {
    roots: [usize; A],
    nodes: [usize; B],
    func: F,
}

pub trait Update {
    fn update(&self, dag: &Dag, updated: u32) -> Option<u32>;

    fn get_roots(&self) -> Iter<usize>;

    fn get_nodes(&self) -> Iter<usize>;

    fn get_nodes_desc(&self) -> Rev<IntoIter<usize>> {
        self.get_nodes().copied().sorted().rev()
    }

    fn update_nodes(&mut self, idxs: &Vec<usize>);
}

impl<const A: usize, const B: usize, F: UpdateFunc<A, B>> Update for UpdateData<A, B, F> {
    fn update(&self, dag: &Dag, updated: u32) -> Option<u32> {
        let mut update = false;
        let roots = self.roots.clone().map(|i| {
            let r = &dag.roots[i];
            update = update || r.updated > updated;
            r.value
        });
        let nodes = self.nodes.clone().map(|i| {
            let n = &dag.nodes[i];
            update = update || n.updated > updated;
            n.value
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
    update: Box<dyn Update>,
    updated: u32,
}

impl<T> NodeValue<T> {
    pub fn default(n: &dyn Node, value: T) -> Self {
        Self {
            id: n.id(),
            depth: 0,
            value,
            update: Box::new(UpdateData {
                roots: [],
                nodes: [],
                func: |[], []| unimplemented!(),
            }),
            updated: 0,
        }
    }
}

// Node Traits
pub trait NodeTrait: 'static {
    fn idx(&self) -> u8;

    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn id(&self) -> NodeId {
        let mut hasher = DefaultHasher::new();
        self.type_id().hash(&mut hasher);
        self.idx().hash(&mut hasher);
        hasher.finish()
    }
}

pub trait Root<T>: NodeTrait {
    fn default(&self) -> T;
}

pub trait Node: NodeTrait {}

// Dag
#[hyperfold_engine::global]
struct Dag {
    roots: Vec<RootValue<u32>>,
    nodes: Vec<NodeValue<u32>>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn find_node(&self, id: NodeId) -> Option<(usize, &NodeValue<u32>)> {
        self.nodes.iter().enumerate().find(|(_, n)| n.id == id)
    }

    fn find_root(&self, id: NodeId) -> Option<(usize, &RootValue<u32>)> {
        self.roots.iter().enumerate().find(|(_, r)| r.id == id)
    }

    fn get_node_impl(&mut self, id: NodeId) -> &NodeValue<u32> {
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

    pub fn get_node<N: Node>(&mut self, n: N) -> u32 {
        self.get_node_impl(n.id()).value
    }

    pub fn add_node<N: Node, const A: usize, const B: usize>(
        &mut self,
        n: N,
        roots: [&dyn Root<u32>; A],
        nodes: [&dyn Node; B],
        func: impl UpdateFunc<A, B> + 'static,
    ) {
        let roots = roots.map(|r| match self.find_root(r.id()) {
            Some((i, _)) => i,
            None => {
                self.roots.push(RootValue::default(r));
                self.roots.len() - 1
            }
        });
        let nodes = nodes.map(|n| match self.find_node(n.id()) {
            Some((i, _)) => i,
            None => {
                self.nodes.push(NodeValue::default(n, 0));
                self.nodes.len() - 1
            }
        });

        // Insert/update the node
        let id = n.id();
        let depth = nodes.iter().fold(0, |m, idx| m.max(self.nodes[*idx].depth));
        let update = Box::new(UpdateData { roots, nodes, func });
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
                    value: 0,
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

    fn get_root_impl(&mut self, id: NodeId, default: u32) -> &RootValue<u32> {
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

    pub fn get_root<R: Root<u32>>(&mut self, r: R) -> u32 {
        self.get_root_impl(r.id(), r.default()).value
    }

    pub fn set<R: Root<u32>>(&mut self, r: R, value: u32) {
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

    pub fn update<R: Root<u32>>(&mut self, r: R, f: impl FnOnce(u32) -> u32) {
        let val = f(self.get_root_impl(r.id(), r.default()).value);
        self.set(r, val)
    }

    pub fn add_root<R: Root<u32>>(&mut self, r: R) {
        let value = r.default();
        self.set(r, value)
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
        impl Node for $name {}
    };

    ($name: ident ($($v: ident),+)) => {
        $crate::parameters!(@def $name ($($v),*));
        impl Node for $name {}
    };

    ($name: ident = $v: literal) => {
        $crate::parameters!(@def $name);
        impl Root for $name {
            fn default(&self) -> u32 {
                $v
            }
        }
    };

    ($name: ident ($($v: ident = $n: literal),+)) => {
        $crate::parameters!(@def $name ($($v),*));
        impl Root<u32> for $name {
            fn default(&self) -> u32 {
                match self {
                    $($name::$v => $n),*
                }
            }
        }
    };
}

// Observers
pub struct RootObserver<T> {
    checked: u32,
    id: NodeId,
    default: T,
}

impl RootObserver<u32> {
    pub fn check(&mut self, dag: &mut Dag, f: impl FnOnce(u32)) {
        let r = dag.get_root_impl(self.id, self.default);
        if self.checked < r.updated {
            f(r.value);
            self.checked = get_time();
        }
    }
}

impl<R: Root<u32>> From<R> for RootObserver<u32> {
    fn from(root: R) -> Self {
        Self {
            checked: 0,
            id: root.id(),
            default: root.default(),
        }
    }
}

pub struct NodeObserver {
    checked: u32,
    id: NodeId,
}

impl NodeObserver {
    pub fn check(&mut self, dag: &mut Dag, f: impl FnOnce(u32)) {
        let n = dag.get_node_impl(self.id);
        if self.checked < n.updated {
            f(n.value);
            self.checked = get_time();
        }
    }
}

impl<N: Node> From<N> for NodeObserver {
    fn from(node: N) -> Self {
        Self {
            checked: 0,
            id: node.id(),
        }
    }
}
