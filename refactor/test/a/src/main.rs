#![allow(unused)]

use b as b2;
use c;

mod gay {
    use engine::game_crate;

    game_crate!();
}

pub mod _engine {
    pub use b;
    pub use c;
    pub use engine;
    pub trait Poopy:
        engine::AddComponent<crate::T6>
        + engine::AddComponent<crate::a2::a3::a::A>
        + engine::_engine::Poopy
        + b::_engine::Poopy
        + c::_engine::Poopy
    {
    }
    pub trait Peepee:
        engine::AddEvent<crate::t8::X>
        + engine::AddEvent<crate::t8::Y>
        + engine::AddEvent<crate::a1::X>
        + engine::AddEvent<crate::a1::Y>
        + engine::AddEvent<crate::a1::Z>
        + engine::_engine::Peepee
        + b::_engine::Peepee
        + c::_engine::Peepee
    {
    }
    pub struct GFoo {
        g0_0: crate::_engine::CFoo,
        g0_1: crate::_engine::EFoo,
        g1_0: engine::Entity,
        g1_1: engine::EntityTrash,
        g1_2: engine::Event,
        g1_3: engine::RenderSystem,
        g1_4: engine::Screen,
        g1_5: engine::Camera,
        g4_0: c::_engine::e::DC,
    }
    impl GFoo {
        pub fn new() -> Self {
            Self {
                g0_0: crate::_engine::CFoo::new(),
                g0_1: crate::_engine::EFoo::new(),
                g1_0: engine::Entity::new(),
                g1_1: engine::EntityTrash::new(),
                g1_2: engine::Event::new(),
                g1_3: engine::RenderSystem::new(),
                g1_4: engine::Screen::new(),
                g1_5: engine::Camera::new(),
                g4_0: c::_engine::e::DC::new(),
            }
        }
    }
    pub struct CFoo {
        eids: std::collections::HashSet<engine::Entity>,
        c0_0: std::collections::HashMap<engine::Entity, crate::T6>,
        c0_1: std::collections::HashMap<engine::Entity, crate::a2::a3::a::A>,
        c2_0: std::collections::HashMap<engine::Entity, b::S>,
    }
    impl CFoo {
        pub fn new() -> Self {
            Self {
                eids: std::collections::HashSet::new(),
                c0_0: std::collections::HashMap::new(),
                c0_1: std::collections::HashMap::new(),
                c2_0: std::collections::HashMap::new(),
            }
        }
        pub fn append(&mut self, cm: &mut Self) {
            self.eids.extend(cm.eids.drain());
            self.c0_0.extend(cm.c0_0.drain());
            self.c0_1.extend(cm.c0_1.drain());
            self.c2_0.extend(cm.c2_0.drain());
        }
        pub fn remove(&mut self, tr: &mut engine::EntityTrash) {
            for eid in tr.0.drain(..) {
                self.eids.remove(&eid);
                self.c0_0.remove(&eid);
                self.c0_1.remove(&eid);
                self.c2_0.remove(&eid);
            }
        }
    }
    impl engine::AddComponent<crate::T6> for CFoo {
        fn add_component(&mut self, e: engine::Entity, t: crate::T6) {
            self.c0_0.insert(e, t);
        }
    }
    impl engine::AddComponent<crate::a2::a3::a::A> for CFoo {
        fn add_component(&mut self, e: engine::Entity, t: crate::a2::a3::a::A) {
            self.c0_1.insert(e, t);
        }
    }
    impl engine::AddComponent<b::S> for CFoo {
        fn add_component(&mut self, e: engine::Entity, t: b::S) {
            self.c2_0.insert(e, t);
        }
    }
    impl engine::_engine::Poopy for CFoo {}

    impl b::_engine::Poopy for CFoo {}

    impl c::_engine::e::_engine::Poopy for CFoo {}

    impl c::_engine::Poopy for CFoo {}

    impl crate::_engine::Poopy for CFoo {}

    #[derive(Hash, Clone, Copy, Eq, PartialEq, Debug)]
    enum E {
        E0_0,
        E0_1,
        E0_2,
        E0_3,
        E0_4,
        E1_0,
        E1_1,
        E1_2,
        E2_0,
        E2_1,
    }
    pub const E_LEN: usize = 10usize;
    pub struct EFoo {
        e0_0: Vec<crate::t8::X>,
        e0_1: Vec<crate::t8::Y>,
        e0_2: Vec<crate::a1::X>,
        e0_3: Vec<crate::a1::Y>,
        e0_4: Vec<crate::a1::Z>,
        e1_0: Vec<engine::core_events::Update>,
        e1_1: Vec<engine::core_events::Events>,
        e1_2: Vec<engine::core_events::Render>,
        e2_0: Vec<b::l::o::l::Lol>,
        e2_1: Vec<b::l::o::l::Cat>,
        events: std::collections::VecDeque<(E, usize)>,
    }
    impl EFoo {
        pub fn new() -> Self {
            Self {
                e0_0: Vec::new(),
                e0_1: Vec::new(),
                e0_2: Vec::new(),
                e0_3: Vec::new(),
                e0_4: Vec::new(),
                e1_0: Vec::new(),
                e1_1: Vec::new(),
                e1_2: Vec::new(),
                e2_0: Vec::new(),
                e2_1: Vec::new(),
                events: std::collections::VecDeque::new(),
            }
        }
        pub fn has_events(&self) -> bool {
            !self.events.is_empty()
        }
        fn add_event(&mut self, e: E) {
            self.events.push_back((e, 0));
        }
        pub fn get_events(&mut self) -> std::collections::VecDeque<(E, usize)> {
            std::mem::replace(&mut self.events, std::collections::VecDeque::new())
        }
        pub fn append(&mut self, other: &mut Self) {
            other.e0_0.reverse();
            self.e0_0.append(&mut other.e0_0);
            other.e0_1.reverse();
            self.e0_1.append(&mut other.e0_1);
            other.e0_2.reverse();
            self.e0_2.append(&mut other.e0_2);
            other.e0_3.reverse();
            self.e0_3.append(&mut other.e0_3);
            other.e0_4.reverse();
            self.e0_4.append(&mut other.e0_4);
            other.e1_0.reverse();
            self.e1_0.append(&mut other.e1_0);
            other.e1_1.reverse();
            self.e1_1.append(&mut other.e1_1);
            other.e1_2.reverse();
            self.e1_2.append(&mut other.e1_2);
            other.e2_0.reverse();
            self.e2_0.append(&mut other.e2_0);
            other.e2_1.reverse();
            self.e2_1.append(&mut other.e2_1);
        }
        pub fn pop(&mut self, e: E) {
            match e {
                E::E0_0 => {
                    self.e0_0.pop();
                }
                E::E0_1 => {
                    self.e0_1.pop();
                }
                E::E0_2 => {
                    self.e0_2.pop();
                }
                E::E0_3 => {
                    self.e0_3.pop();
                }
                E::E0_4 => {
                    self.e0_4.pop();
                }
                E::E1_0 => {
                    self.e1_0.pop();
                }
                E::E1_1 => {
                    self.e1_1.pop();
                }
                E::E1_2 => {
                    self.e1_2.pop();
                }
                E::E2_0 => {
                    self.e2_0.pop();
                }
                E::E2_1 => {
                    self.e2_1.pop();
                }
            }
        }
    }
    impl engine::AddEvent<crate::t8::X> for EFoo {
        fn new_event(&mut self, t: crate::t8::X) {
            self.e0_0.push(t);
            self.add_event(E::E0_0);
        }
        fn get_event<'a>(&'a self) -> Option<&'a crate::t8::X> {
            self.e0_0.last()
        }
    }
    impl engine::AddEvent<crate::t8::Y> for EFoo {
        fn new_event(&mut self, t: crate::t8::Y) {
            self.e0_1.push(t);
            self.add_event(E::E0_1);
        }
        fn get_event<'a>(&'a self) -> Option<&'a crate::t8::Y> {
            self.e0_1.last()
        }
    }
    impl engine::AddEvent<crate::a1::X> for EFoo {
        fn new_event(&mut self, t: crate::a1::X) {
            self.e0_2.push(t);
            self.add_event(E::E0_2);
        }
        fn get_event<'a>(&'a self) -> Option<&'a crate::a1::X> {
            self.e0_2.last()
        }
    }
    impl engine::AddEvent<crate::a1::Y> for EFoo {
        fn new_event(&mut self, t: crate::a1::Y) {
            self.e0_3.push(t);
            self.add_event(E::E0_3);
        }
        fn get_event<'a>(&'a self) -> Option<&'a crate::a1::Y> {
            self.e0_3.last()
        }
    }
    impl engine::AddEvent<crate::a1::Z> for EFoo {
        fn new_event(&mut self, t: crate::a1::Z) {
            self.e0_4.push(t);
            self.add_event(E::E0_4);
        }
        fn get_event<'a>(&'a self) -> Option<&'a crate::a1::Z> {
            self.e0_4.last()
        }
    }
    impl engine::AddEvent<engine::core_events::Update> for EFoo {
        fn new_event(&mut self, t: engine::core_events::Update) {
            self.e1_0.push(t);
            self.add_event(E::E1_0);
        }
        fn get_event<'a>(&'a self) -> Option<&'a engine::core_events::Update> {
            self.e1_0.last()
        }
    }
    impl engine::AddEvent<engine::core_events::Events> for EFoo {
        fn new_event(&mut self, t: engine::core_events::Events) {
            self.e1_1.push(t);
            self.add_event(E::E1_1);
        }
        fn get_event<'a>(&'a self) -> Option<&'a engine::core_events::Events> {
            self.e1_1.last()
        }
    }
    impl engine::AddEvent<engine::core_events::Render> for EFoo {
        fn new_event(&mut self, t: engine::core_events::Render) {
            self.e1_2.push(t);
            self.add_event(E::E1_2);
        }
        fn get_event<'a>(&'a self) -> Option<&'a engine::core_events::Render> {
            self.e1_2.last()
        }
    }
    impl engine::AddEvent<b::l::o::l::Lol> for EFoo {
        fn new_event(&mut self, t: b::l::o::l::Lol) {
            self.e2_0.push(t);
            self.add_event(E::E2_0);
        }
        fn get_event<'a>(&'a self) -> Option<&'a b::l::o::l::Lol> {
            self.e2_0.last()
        }
    }
    impl engine::AddEvent<b::l::o::l::Cat> for EFoo {
        fn new_event(&mut self, t: b::l::o::l::Cat) {
            self.e2_1.push(t);
            self.add_event(E::E2_1);
        }
        fn get_event<'a>(&'a self) -> Option<&'a b::l::o::l::Cat> {
            self.e2_1.last()
        }
    }
    impl engine::_engine::Peepee for EFoo {}

    impl b::_engine::Peepee for EFoo {}

    impl c::_engine::e::_engine::Peepee for EFoo {}

    impl c::_engine::Peepee for EFoo {}

    impl crate::_engine::Peepee for EFoo {}

    pub struct SFoo {
        cfoo: CFoo,
        gfoo: GFoo,
        efoo: EFoo,
        stack: Vec<std::collections::VecDeque<(E, usize)>>,
        services: [Vec<Box<dyn Fn(&mut CFoo, &mut GFoo, &mut EFoo)>>; E_LEN],
    }
    impl SFoo {
        pub fn new() -> Self {
            let mut s = Self {
                cfoo: CFoo::new(),
                gfoo: GFoo::new(),
                efoo: EFoo::new(),
                stack: Vec::new(),
                services: std::array::from_fn(|_| Vec::new()),
            };
            s.init();
            s
        }
        fn init(&mut self) {
            (|cfoo: &mut CFoo, gfoo: &mut GFoo, efoo: &mut EFoo| crate::init(&mut gfoo.g0_1))(
                &mut self.cfoo,
                &mut self.gfoo,
                &mut self.efoo,
            );
            (|cfoo: &mut CFoo, gfoo: &mut GFoo, efoo: &mut EFoo| {
                c::_engine::e::bar(&mut gfoo.g4_0)
            })(&mut self.cfoo, &mut self.gfoo, &mut self.efoo);
            self.post_tick();
            self.add_systems();
        }
        fn add_system(&mut self, e: E, f: Box<dyn Fn(&mut CFoo, &mut GFoo, &mut EFoo)>) {
            self.services[e as usize].push(f);
        }
        fn add_systems(&mut self) {
            let f = |cfoo: &mut CFoo, gfoo: &mut GFoo, efoo: &mut EFoo| {
                if let Some(e) = engine::AddEvent::get_event(efoo) {
                    for eid in
                        engine::intersect::intersect_keys(&mut [engine::intersect::get_keys(
                            &cfoo.c0_0,
                        )])
                        .iter()
                    {
                        if let (Some(c0_0),) = (cfoo.c0_0.get_mut(eid),) {
                            if (!cfoo.c0_1.contains_key(eid)) {
                                crate::comp(
                                    e,
                                    c0_0,
                                    &mut gfoo.g4_0,
                                    std::marker::PhantomData,
                                    std::marker::PhantomData,
                                    &mut gfoo.g0_1,
                                )
                            }
                        }
                    }
                }
            };
            self.add_system(E::E0_0, Box::new(f));
            let f = |cfoo: &mut CFoo, gfoo: &mut GFoo, efoo: &mut EFoo| {
                if let Some(e) = engine::AddEvent::get_event(efoo) {
                    let mut v = cfoo
                        .c0_0
                        .iter_mut()
                        .map(|(k, v)| (k, (Some(v), None)))
                        .collect::<std::collections::HashMap<_, (_, Option<_>)>>();
                    let v = v
                        .into_iter()
                        .filter_map(|(eid, (v0, v1))| {
                            if let (Some(v0),) = (v0,) {
                                if (!cfoo.c0_1.contains_key(eid)) {
                                    return Some((v0, eid));
                                }
                            }
                            None
                        })
                        .collect::<Vec<_>>();
                    crate::vec(
                        e,
                        &mut gfoo.g4_0,
                        std::marker::PhantomData,
                        std::marker::PhantomData,
                        v,
                        &mut gfoo.g0_1,
                    );
                }
            };
            self.add_system(E::E0_1, Box::new(f));
            let f = |cfoo: &mut CFoo, gfoo: &mut GFoo, efoo: &mut EFoo| {
                if let Some(e) = engine::AddEvent::get_event(efoo) {
                    b::sys(e)
                }
            };
            self.add_system(E::E2_0, Box::new(f));
        }
        pub fn run(&mut self) {
            static FPS: u32 = 60;
            static FRAME_TIME: u32 = 1000 / FPS;
            let mut t = unsafe { crate::sdl2::SDL_GetTicks() };
            let mut dt;
            let mut tsum: u64 = 0;
            let mut tcnt: u64 = 0;
            while !self.gfoo.g1_2.quit {
                dt = unsafe { crate::sdl2::SDL_GetTicks() } - t;
                t += dt;
                self.tick(dt);
                dt = unsafe { crate::sdl2::SDL_GetTicks() } - t;
                tsum += dt as u64;
                tcnt += 1;
                if dt < FRAME_TIME {
                    unsafe { crate::sdl2::SDL_Delay(FRAME_TIME - dt) };
                }
            }
            println!("Average Frame Time: {}ms", tsum as f64 / tcnt as f64);
        }
        fn tick(&mut self, ts: u32) {
            self.gfoo
                .g1_2
                .update(ts, &self.gfoo.g1_5.0, &self.gfoo.g1_4.0);
            self.gfoo.g1_3.r.clear();
            self.add_events(self.init_events(ts));
            while !self.stack.is_empty() {
                if let Some((e, i, n)) = self
                    .stack
                    .last_mut()
                    .and_then(|queue| queue.front_mut())
                    .and_then(|(e, i)| {
                        let v_s = &self.services[*e as usize];
                        v_s.get(*i).map(|_| {
                            let vals = (e.clone(), i.clone(), v_s.len());
                            *i += 1;
                            vals
                        })
                    })
                {
                    if i + 1 >= n {
                        self.pop();
                    }
                    self.gfoo.g0_1 = EFoo::new();
                    if let Some(s) = self.services[e as usize].get(i) {
                        (s)(&mut self.cfoo, &mut self.gfoo, &mut self.efoo);
                    }
                    if i + 1 >= n {
                        self.efoo.pop(e);
                    }
                    let efoo = std::mem::replace(&mut self.gfoo.g0_1, EFoo::new());
                    self.add_events(efoo);
                } else {
                    self.pop();
                }
            }
            self.gfoo.g1_3.r.present();
            self.post_tick();
        }
        fn post_tick(&mut self) {
            self.cfoo.remove(&mut self.gfoo.g1_1);
            self.cfoo.append(&mut self.gfoo.g0_0);
        }
        fn init_events(&self, ts: u32) -> EFoo {
            let mut efoo = EFoo::new();
            engine::AddEvent::new_event(&mut efoo, engine::core_events::Events);
            engine::AddEvent::new_event(&mut efoo, engine::core_events::Update(ts));
            engine::AddEvent::new_event(&mut efoo, engine::core_events::Render);
            efoo
        }
        fn add_events(&mut self, mut em: EFoo) {
            if em.has_events() {
                self.efoo.append(&mut em);
                self.stack.push(em.get_events());
            }
        }
        fn pop(&mut self) {
            if self.stack.last_mut().is_some_and(|queue| {
                queue.pop_front();
                queue.is_empty()
            }) {
                self.stack.pop();
            }
        }
    }
}

pub const T1: u8 = 0;
static T2: u8 = 0;
pub trait T3 {}
type T4 = u8;
union T5 {
    i: u8,
}
#[engine::component]
pub struct T6;
pub fn T7() {}
// TODO: try toggling pub
pub mod t8 {
    #[engine::event]
    pub struct X;
    #[engine::event]
    pub struct Y(u8);
}

mod a1;
pub mod a2;

pub use a2 as a22;

pub use a2::a3::z::comp as component;
use engine::{game_crate, Container, Label, NorLabels};

#[engine::system]
fn comp(
    _e: &crate::t8::X,
    t6: &T6,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::a::A, a2::a3::a::A)>,
    t: &dyn crate::_engine::Peepee,
) {
}

#[engine::system]
fn vec(
    _e: &crate::t8::Y,
    dc: &c::e::DC,
    sc: Label<T6>,
    re: NorLabels<(a2::a3::a::A, a2::a3::a::A)>,
    vc: Container<(&mut T6, &engine::Entity)>,
    t: &dyn crate::_engine::Peepee,
) {
}

#[engine::system(Init)]
fn init(t: &dyn crate::_engine::Peepee) {}

fn main() {
    println!("Hello, world!");
}
