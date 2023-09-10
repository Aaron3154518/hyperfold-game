use std::{
    any::TypeId,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use hyperfold_engine::{
    add_components, components,
    ecs::{
        entities::{Entity, NewEntity},
        events::core::{PreRender, Update},
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
        math::NormalizeAngle,
        rect::{Align, Rect},
        traits::Id,
        util::FloatMath,
    },
};
use itertools::Itertools;

use crate::{_engine::Components, utils::elevations::Elevations};

// Upgrade box
#[hyperfold_engine::component(Singleton)]
struct UpgradeBox {
    v_scroll: f32,
    scroll: f32,
    min_scroll: f32,
    max_scroll: f32,
    upgrade_rects: Vec<(Rect, usize)>,
    open_id: Option<TypeId>,
    update: bool,
}

impl Default for UpgradeBox {
    fn default() -> Self {
        Self {
            v_scroll: 0.0,
            scroll: 0.0,
            min_scroll: 0.0,
            max_scroll: 0.0,
            upgrade_rects: Vec::new(),
            open_id: None,
            update: false,
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

components!(Upgrades, up: &'a Upgrade);
components!(UpgradeBoxData, up_box: &'a mut UpgradeBox);
components!(UpgradeBoxDrawArgs, up_box: &'a mut UpgradeBox, pos: &'a Position, opts: &'a mut RenderOpts, tex: &'a mut RenderComponent);

// Draw upgrade box
#[hyperfold_engine::system]
fn draw_upgrades(
    _: &PreRender,
    upgrades: Vec<Upgrades>,
    UpgradeBoxDrawArgs {
        pos,
        up_box,
        opts,
        tex,
        ..
    }: UpgradeBoxDrawArgs,
    r: &Renderer,
) {
    if !up_box.update {
        return;
    }

    let id = match up_box.open_id {
        Some(id) => id,
        None => {
            opts.visible = false;
            return;
        }
    };

    let upgrades = filter_upgrades(upgrades, id);
    let n = upgrades.len();

    up_box.max_scroll = match pos.0.empty() {
        true => 0.0,
        false => pos.0.w() * f32!(n.max(1) - 1) / (pos.0.w() * 2.0 / pos.0.h()).floor(),
    };

    // Largest width of an upgrade icon
    // Also the scroll distance between each upgrade
    let w = pos.0.h() / 2.0;
    let (cx, cy) = (pos.0.half_w(), pos.0.half_h() - w / 4.0);
    // x/y radii of the upgrade ellipse
    let (rx, ry) = ((pos.0.w() - w) / 2.0, (pos.0.h() - w) / 2.0);

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

    // Higher means closer back upgrades
    let m = 10.0;
    // Number of front upgrades
    let num_steps = (pos.0.w() / w).floor_i32();
    let step = PI / f32!(num_steps);
    // Decays from step @ a=0 to step/m @ a = PI/2
    let lin_da = |a: f32| step * (1.0 - a / FRAC_PI_2 * (m - 1.0) / m);
    // x = a + da, da = lin_da(PI - x)
    // da(a) = (step*m*pi/2 + s*(m-1)*(a-pi)) / (m*pi/2 - s*(m-1))
    let pred_da = |a: f32| {
        let t = step * (m - 1.0);
        let u = m * FRAC_PI_2;
        (step * u + t * (a - PI)) / (u - t)
    };
    // Solve for da when a = PI/2
    let min_step = pred_da(FRAC_PI_2);

    // TODO: Fix max_scroll
    // TODO: compress near pi/2, stretch near 0,pi

    // Compute angular displacement of the first upgrade
    let mut angle = 3.0 * FRAC_PI_2 - up_box.scroll * PI / pos.0.w();
    let mut idx = 0;
    let mut rects = Vec::new();
    while angle < 5.0 * FRAC_PI_2 && idx < n {
        if angle >= FRAC_PI_2 {
            rects.push((get_rect(angle), idx, angle));
        }
        let a = angle.normalize_rad();
        angle += match a {
            // Linear scaling
            a if a <= FRAC_PI_2 - min_step => lin_da(a),
            // Constant min step to smooth accross PI/2
            a if a <= FRAC_PI_2 => min_step,
            // Predictive linear scaling
            a if a <= PI - step => pred_da(a),
            // Constant max step
            _ => step,
        };
        idx += 1;
    }

    up_box.upgrade_rects = rects
        .into_iter()
        // Sort top to bottom (render order)
        .sorted_by(|(.., a1), (.., a2)| {
            a1.sin()
                .partial_cmp(&a2.sin())
                .expect("NaN When sorting upgrade rects")
                .reverse()
        })
        .map(|(r, i, _)| (r, i))
        .collect();

    opts.set_visible(true);

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

    let new_tex = Texture::new(r, pos.0.w_i32() as u32, pos.0.h_i32() as u32, GRAY);
    for (rect, idx) in &up_box.upgrade_rects {
        new_tex.draw(
            r,
            &mut Rectangle::new().fill(*rect).set_color(heatmap(*idx, n)),
        );
    }
    tex.set(RenderTexture::new(Some(new_tex)));

    up_box.update = false;
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
    UpgradeBoxData { up_box, .. }: UpgradeBoxData,
) {
    up_box.open_id = Some(*id);
    up_box.update = true;
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

// TODO:
// Dragging
#[hyperfold_engine::system]
fn drag_start_upgrade_box(
    DragStart(id): &DragStart,
    UpgradeBoxData { up_box, eid }: UpgradeBoxData,
) {
    if id == eid {
        up_box.v_scroll = 0.0;
    }
}

#[hyperfold_engine::system]
fn drag_upgrade_box(drag: &Drag, UpgradeBoxData { up_box, eid }: UpgradeBoxData) {
    if &drag.eid == eid {
        let prev = up_box.scroll;
        up_box.scroll = (up_box.scroll - f32!(drag.mouse_dx))
            .max(up_box.min_scroll)
            .min(up_box.max_scroll);
        up_box.v_scroll = -f32!(drag.mouse_dx) * 25.0;
        if (prev - up_box.scroll).abs() >= 1.0 {
            up_box.update = true;
        }
    }
}

#[hyperfold_engine::system]
fn update_upgrade_box(
    Update(dt): &Update,
    UpgradeBoxData { up_box, eid }: UpgradeBoxData,
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
        up_box.v_scroll *= 0.01_f32.powf(s);
        up_box.update = true;
    } else {
        up_box.v_scroll = 0.0;
    }
}
