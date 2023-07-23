use hyperfold_engine::{
    _engine::Entity,
    add_components, components,
    ecs::{
        entities::{EntityTrash, NewEntity},
        events::core::Update,
    },
    framework::{
        physics::{PhysicsData, Position},
        render_system::{
            render_data::{Animation, RenderAsset, RenderDataBuilderTrait},
            Asset, AssetManager, Camera, Elevation, RenderComponent, Renderer,
        },
    },
    utils::rect::{Align, PointF, Rect},
};
use itertools::Itertools;

use crate::{
    _engine::AddComponent,
    crystal::{crystal_radius, CrystalPos},
    utils::elevations::Elevations,
    wizard::WizardPos,
};

#[hyperfold_engine::component]
struct Enemy;

pub fn spawn_enemy(
    pos: PointF,
    entities: &mut dyn AddComponent,
    r: &Renderer,
    am: &mut AssetManager,
) {
    let e = Entity::new();
    let anim = Animation::new(8, 100);
    add_components!(
        entities,
        e,
        Enemy,
        Elevation(Elevations::Enemies as u8),
        RenderComponent::new(
            RenderAsset::new(
                Asset::File("res/wizards/power_wizard_ss.png".to_string()),
                r,
                am
            )
            .with_animation(anim)
        ),
        anim,
        Position(Rect::from(
            pos.x,
            pos.y,
            50.0,
            50.0,
            Align::Center,
            Align::Center
        )),
        PhysicsData {
            v: PointF::new(),
            a: PointF::new(),
            // TODO: boundary is crystal range
            boundary: None
        }
    );
}

#[hyperfold_engine::component]
struct TargetEnemy;

components!(labels(TargetEnemy), EnemyProjectiles, pos: &'a Position);
components!(labels(Enemy), Enemies, pos: &'a Position);
components!(
    labels(Enemy),
    UpdateEnemies,
    pos: &'a Position,
    pd: &'a mut PhysicsData
);

#[hyperfold_engine::system]
fn update_enemies(
    _: &Update,
    enemies: Vec<UpdateEnemies>,
    wizard: WizardPos,
    crystal: CrystalPos,
    camera: &Camera,
    trash: &mut EntityTrash,
) {
    const SPEED: f32 = 125.0;
    match wizard.pos.0.center().dist(crystal.pos.0.center()) <= crystal_radius(camera) {
        true => enemies.into_iter().for_each(|enemy| {
            enemy.pd.v = PointF::new();
            enemy.pd.a = PointF::new();
        }),
        false => enemies.into_iter().for_each(|enemy| {
            let target = wizard.pos.0.center();
            let delta = target - enemy.pos.0.center();
            let mag = delta.mag();
            match mag <= 25.0 {
                true => {
                    trash.0.push(*enemy.eid);
                }
                false => {
                    enemy.pd.v = PointF {
                        x: delta.x * SPEED / mag,
                        y: delta.y * SPEED / mag,
                    }
                }
            }
        }),
    }
}

#[hyperfold_engine::system]
fn fb_hit_enemy(
    _: &Update,
    mut projs: Vec<EnemyProjectiles>,
    enemies: Vec<Enemies>,
    trash: &mut EntityTrash,
) {
    for enemy in enemies {
        if let Some((i, proj)) = projs
            .iter()
            .find_position(|proj| enemy.pos.0.intersects(&proj.pos.0))
        {
            trash.0.extend([enemy.eid, proj.eid]);
            projs.swap_remove(i);
        }
    }
}
