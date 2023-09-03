use std::{
    any::TypeId,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use either::Either::{Left, Right};
use hyperfold_engine::{
    add_components, components,
    ecs::{
        entities::{Entity, NewEntity},
        events::core::Update,
    },
    f32,
    framework::{
        event_system::mouse::{Drag, DragStart, DragState, DragTrigger},
        physics::Position,
        render_system::{
            drawable::Canvas,
            render_data::RenderTexture,
            shapes::{Rectangle, ShapeTrait},
            RenderComponent, RenderOpts, Renderer, Screen, Texture,
        },
    },
    utils::{
        colors::{BLUE, GRAY, GREEN, RED},
        rect::{Align, Rect},
        traits::Id,
        util::FloatMath,
    },
};
use itertools::Itertools;

use crate::{
    _engine::{Components, Events},
    utils::elevations::Elevations,
};

// Upgrade box
#[hyperfold_engine::component(Singleton)]
struct UpgradeBox {
    v_scroll: f32,
    scroll: f32,
    min_scroll: f32,
    max_scroll: f32,
    back_upgrades: Vec<(Rect, usize)>,
    front_upgrades: Vec<(Rect, usize)>,
    curr_id: Option<TypeId>,
}

impl Default for UpgradeBox {
    fn default() -> Self {
        Self {
            v_scroll: 0.0,
            scroll: 0.0,
            min_scroll: 0.0,
            max_scroll: 0.0,
            back_upgrades: Vec::new(),
            front_upgrades: Vec::new(),
            curr_id: None,
        }
    }
}

#[hyperfold_engine::system(Init)]
fn init_upgrades(entities: &mut dyn Components, screen: &Screen, r: &Renderer) {
    let (w, h) = (screen.0.w as f32, screen.0.h as f32);
    let rect = Rect::from(
        w / 2.0,
        0.0,
        w / 2.0,
        h / 5.0,
        Align::Center,
        Align::TopLeft,
    );
    let tex = Texture::new(r, rect.w_i32() as u32, rect.h_i32() as u32, GRAY);

    let e = Entity::new();
    add_components!(
        entities,
        e,
        UpgradeBox::default(),
        DragTrigger::OnMove,
        Position(rect),
        RenderOpts::new(Elevations::Upgrades as u8)
            .absolute()
            .is_visible(false),
        RenderComponent::new(RenderTexture::new(Some(tex)))
    );
}

// Upgrades
#[hyperfold_engine::component]
struct Upgrade {
    // Upgrades with the same ids get shown together
    id: TypeId,
    idx: usize,
}

impl Upgrade {
    pub fn new(t: impl Id, idx: usize) -> Self {
        Self {
            id: t.type_id(),
            idx,
        }
    }
}

// Switch upgrade list
#[hyperfold_engine::event]
struct OpenUpgrades {
    id: TypeId,
}

impl OpenUpgrades {
    pub fn new(t: impl Id) -> Self {
        Self { id: t.type_id() }
    }
}

components!(Upgrades, up: &'a Upgrade);
components!(UpgradeBoxPos, up_box: &'a mut UpgradeBox, pos: &'a Position, opts: &'a mut RenderOpts, tex: &'a mut RenderComponent);

// Returns Upgrades with the given Id, sorted by index
pub fn filter_upgrades(upgrades: Vec<Upgrades>, id: TypeId) -> Vec<Upgrades> {
    upgrades
        .into_iter()
        .filter(|u| u.up.id == id)
        .sorted_by_key(|u| u.up.idx)
        .collect()
}

#[hyperfold_engine::system]
fn open_upgrades(
    OpenUpgrades { id }: &OpenUpgrades,
    upgrades: Vec<Upgrades>,
    UpgradeBoxPos {
        pos,
        up_box,
        opts,
        tex,
        ..
    }: UpgradeBoxPos,
    r: &Renderer,
) {
    up_box.curr_id = Some(*id);

    let upgrades = filter_upgrades(upgrades, *id);
    let n = upgrades.len();

    up_box.max_scroll = match pos.0.empty() {
        true => 0.0,
        false => pos.0.w() * f32!(n.max(1) - 1) / (pos.0.w() * 2.0 / pos.0.h()).floor(),
    };
    // mMaxScroll = pos.empty()
    //                  ? 0
    //                  : pos.w() * (mUpgrades ? mUpgrades->size() - 1 : 0) /
    //                        floor(pos.w() * 2 / pos.h());

    // Largest width of an upgrade icon
    // Also the scroll distance between each upgrade
    let w = pos.0.h() / 2.0;
    // x/y radii of the upgrade ellipse
    let (rx, ry) = ((pos.0.w() - w) / 2.0, (pos.0.h() - w) / 2.0);

    // Position of central image inside ellipse
    let img_rect = Rect::from(
        pos.0.cx(),
        pos.0.cy(),
        w * 2.0,
        w * 2.0,
        Align::Center,
        Align::Center,
    );
    // TODO: render image
    // mImg.setDest(imgR);

    // Number of upgrades per quadrant
    // TODO: double in back
    let num_steps = (pos.0.w() / w).floor_i32();
    let step = PI / f32!(num_steps);
    let err = 1e-5;

    // Compute angular displacement of the first upgrade
    let scroll_angle = up_box.scroll * PI / pos.0.w();
    // Compute the index displacement of the first upgrade
    // Use .5 - ERR so we don't round up at .5
    let base_idx = (scroll_angle / step + 0.5 - err).floor_i32();
    // Constrain scroll angle to [0, 2PI)
    let scroll_angle = scroll_angle % TAU;
    // Transform to CW with 0 at PI/2
    let theta = (5.0 * FRAC_PI_2 - scroll_angle) % TAU;
    // Find the step closest to PI/2
    let min_theta = (theta + 3.0 * FRAC_PI_2) % step;
    let min_theta = match min_theta + err < step - min_theta {
        true => min_theta + FRAC_PI_2,
        false => min_theta + FRAC_PI_2 - step,
    };

    let (cx, cy) = (pos.0.half_w(), pos.0.half_h() - w / 4.0);

    let get_rect = |angle: f32| {
        let angle_diff = ((angle + 3.0 * FRAC_PI_2) % TAU).min((5.0 * FRAC_PI_2 - angle) % TAU);
        Rect::from(
            cx + rx * angle.cos(),
            cy - ry * angle.sin(),
            w * angle_diff / PI,
            w * angle_diff / PI,
            Align::Center,
            Align::Center,
        )
    };

    let mut rect_angles = Vec::new();
    rect_angles.resize(n, None);
    up_box.front_upgrades.clear();
    up_box.back_upgrades.clear();

    // This pushes upgrades in order from front-most to back-most
    for i in (0..num_steps + 1).rev() {
        for sign in match i {
            i if i == 0 || i == num_steps => Left([1]),
            _ => Right([1, -1]),
        }
        .into_iter()
        {
            let angle = (min_theta + f32!(i) * f32!(sign) * step + TAU) % TAU;
            let idx = base_idx - sign * (num_steps - i);
            if 0 <= idx && (idx as usize) < n {
                let idx = idx as usize;
                rect_angles[idx] = Some(angle);
                match angle < PI {
                    true => &mut up_box.back_upgrades,
                    false => &mut up_box.front_upgrades,
                }
                .push((get_rect(angle), upgrades[idx].up.idx));
            }
        }
    }

    opts.set_visible(true);

    let new_tex = Texture::new(r, pos.0.w_i32() as u32, pos.0.h_i32() as u32, GRAY);
    for (rect, idx) in up_box.back_upgrades.iter().rev() {
        new_tex.draw(
            r,
            &mut Rectangle::new().fill(*rect).set_color(heatmap(*idx, n)),
        );
    }
    for (rect, idx) in up_box.front_upgrades.iter().rev() {
        new_tex.draw(
            r,
            &mut Rectangle::new().fill(*rect).set_color(heatmap(*idx, n)),
        );
    }
    tex.set(RenderTexture::new(Some(new_tex)));
}

fn heatmap(i: usize, n: usize) -> hyperfold_engine::sdl2::SDL_Color {
    let colors = [RED, GREEN, BLUE];

    if i >= n {
        return colors[colors.len() - 1];
    }

    let frac = i as f32 / n as f32 * (colors.len() - 1) as f32;

    let (i1, i2) = (frac.floor_i32() as usize, frac.ceil_i32() as usize);
    let (f1, f2) = (1.0 - frac % 1.0, frac % 1.0);

    hyperfold_engine::sdl2::SDL_Color {
        r: (colors[i1].r as f32 * f1 + colors[i2].r as f32 * f2) as u8,
        b: (colors[i1].b as f32 * f1 + colors[i2].b as f32 * f2) as u8,
        g: (colors[i1].g as f32 * f1 + colors[i2].g as f32 * f2) as u8,
        a: (colors[i1].a as f32 * f1 + colors[i2].a as f32 * f2) as u8,
    }
}

// Dragging
#[hyperfold_engine::system]
fn drag_start_upgrade_box(
    DragStart(id): &DragStart,
    UpgradeBoxPos { up_box, eid, .. }: UpgradeBoxPos,
) {
    if id == eid {
        up_box.v_scroll = 0.0;
    }
}

#[hyperfold_engine::system]
fn drag_upgrade_box(
    drag: &Drag,
    UpgradeBoxPos { up_box, eid, .. }: UpgradeBoxPos,
    events: &mut dyn Events,
) {
    if &drag.eid == eid {
        let prev = up_box.scroll;
        up_box.scroll = (up_box.scroll - f32!(drag.mouse_dx))
            .max(up_box.min_scroll)
            .min(up_box.max_scroll);
        up_box.v_scroll = -f32!(drag.mouse_dx) * 25.0;
        if (prev - up_box.scroll).abs() >= 1.0 {
            if let Some(id) = up_box.curr_id {
                events.new_event(OpenUpgrades { id });
            }
        }
    }
}

#[hyperfold_engine::system]
fn update_upgrade_box(
    Update(dt): &Update,
    UpgradeBoxPos { up_box, eid, .. }: UpgradeBoxPos,
    events: &mut dyn Events,
    drag_state: &DragState,
) {
    if drag_state.dragging(*eid) {
        return;
    }

    let s = f32!(*dt) / 1000.0;
    let prev = up_box.scroll;
    up_box.scroll = (up_box.scroll + up_box.v_scroll * s)
        .max(up_box.min_scroll)
        .min(up_box.max_scroll);
    if (prev - up_box.scroll).abs() >= 1.0 {
        if let Some(id) = up_box.curr_id {
            events.new_event(OpenUpgrades { id });
        }
        up_box.v_scroll *= 0.01_f32.powf(s);
    } else {
        up_box.v_scroll = 0.0;
    }
}
