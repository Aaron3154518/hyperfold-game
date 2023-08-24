use std::{any::TypeId, collections::HashMap};

use hyperfold_engine::utils::util::get_time;

struct RootValue<T> {
    value: T,
    updated: u32,
}

pub trait UpdateFunc<const A: usize, const B: usize> = Fn([u32; A], [u32; B]) -> u32;

struct UpdateData<const A: usize, const B: usize, F: UpdateFunc<A, B>> {
    roots: [(DagIdx, u32); A],
    nodes: [DagIdx; B],
    func: F,
}

pub trait Update {
    fn get(&self, dag: &mut Dag) -> u32;
}

impl<const A: usize, const B: usize, F: UpdateFunc<A, B>> Update for UpdateData<A, B, F> {
    fn get(&self, dag: &mut Dag) -> u32 {
        (self.func)(
            self.roots
                .each_ref()
                .map(|(idx, def)| dag.get_root_impl(*idx, *def)),
            self.nodes.each_ref().map(|idx| dag.get_node_impl(*idx)),
        )
    }
}

struct NodeValue<T> {
    value: T,
    update: Box<dyn Update>,
    updated: u32,
}

pub trait NodeTrait: 'static {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn idx(&self) -> u32;
}

pub trait Root: NodeTrait {
    fn default(&self) -> u32;
}

pub trait Node: NodeTrait {}

pub type DagIdx = (TypeId, u32);

#[hyperfold_engine::global]
struct Dag {
    roots: Vec<(DagIdx, RootValue<u32>)>,
    nodes: HashMap<DagIdx, NodeValue<u32>>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            nodes: HashMap::new(),
        }
    }

    // fn get_time() -> u64 {
    //     SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .map(|d| d.as_secs() * 1000 + d.subsec_nanos() as u64 / 1_000_000)
    //         .expect("Failed to get time")
    // }

    pub fn get_node_impl(&mut self, idx: DagIdx) -> u32 {
        0
    }

    pub fn get_node<N: Node>(&mut self, n: N) -> u32 {
        self.get_node_impl((n.type_id(), n.idx()))
    }

    pub fn add_node<N: Node, const A: usize, const B: usize>(
        &mut self,
        n: N,
        roots: [&dyn Root; A],
        nodes: [&dyn Node; B],
        func: impl UpdateFunc<A, B> + 'static,
    ) {
        let roots = roots.map(|r| ((r.type_id(), r.idx()), r.default()));
        let nodes = nodes.map(|n| (n.type_id(), n.idx()));

        let idx = (n.type_id(), n.idx());
        let node = NodeValue {
            value: 0,
            update: Box::new(UpdateData { roots, nodes, func }),
            updated: 0,
        };
        self.nodes.insert(idx, node);
    }

    fn get_root_impl(&mut self, idx: DagIdx, default: u32) -> u32 {
        match self.roots.iter().find(|r| r.0 == idx) {
            Some(r) => r.1.value,
            None => {
                self.roots.push((
                    idx,
                    RootValue {
                        value: default,
                        updated: get_time(),
                    },
                ));
                default
            }
        }
    }

    pub fn get_root<R: Root>(&mut self, r: R) -> u32 {
        self.get_root_impl((r.type_id(), r.idx()), r.default())
    }

    pub fn set_root<R: Root>(&mut self, r: R, value: u32) {
        let idx = (r.type_id(), r.idx());
        let value = RootValue {
            value,
            updated: get_time(),
        };
        match self.roots.iter_mut().find(|r| r.0 == idx) {
            Some(r) => r.1 = value,
            None => self.roots.push((idx, value)),
        }
    }

    pub fn add_root<R: Root>(&mut self, r: R) {
        let value = r.default();
        self.set_root(r, value)
    }
}

#[macro_export]
macro_rules! roots {
    ($name: ident = $v: literal) => {
        #[derive(Copy, Clone)]
        pub struct $name;

        impl NodeTrait for $name {
            fn idx(&self) -> u32 {
                0
            }
        }

        impl Root for $name {
            fn default(&self) -> u32 {
                $v
            }
        }
    };

    ($name: ident ($($v: ident = $n: literal),+)) => {
        #[derive(Copy, Clone)]
        pub enum $name {
            $($v),*
        }

        impl NodeTrait for $name {
            fn idx(&self) -> u32 {
                *self as u32
            }
        }

        impl Root for $name {
            fn default(&self) -> u32 {
                match self {
                    $($name::$v => $n),*
                }
            }
        }
    };
}

roots!(DagVals(A = 1, B = 2, C = 3));

#[hyperfold_engine::system(Init)]
fn init_dag(dag: &mut Dag) {
    dag.add_root(DagVals::A);
}
