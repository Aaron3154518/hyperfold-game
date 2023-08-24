use std::ops::Range;

use hyperfold_engine::{
    add_components, components,
    ecs::{
        entities::{Entity, NewEntity},
        events::core::Update,
    },
    framework::render_system::{AssetManager, Camera, Renderer},
    sdl2::SDL_FPoint,
    utils::{
        rect::PointF,
        timer::{Timer, TimerTrait},
    },
};
use rand::Rng;

use crate::{
    _engine::Components,
    crystal::{crystal_radius, CrystalPos},
    utils::rand_sign::RandSign,
    wizard::WizardPos,
};

use super::enemy::spawn_enemy;

#[hyperfold_engine::component(Singleton)]
struct EnemySpawner;

#[hyperfold_engine::system(Init)]
fn init_enemy_spawner(entities: &mut dyn Components) {
    let e = Entity::new();
    add_components!(entities, e, EnemySpawner, Timer::new(1000));
}

components!(
    labels(EnemySpawner),
    EnemySpawnerTimer,
    timer: &'a mut Timer
);

#[hyperfold_engine::system]
fn enemy_spawner_timer(
    dt: &Update,
    EnemySpawnerTimer { timer, .. }: EnemySpawnerTimer,
    wizard: WizardPos,
    crystal: CrystalPos,
    entities: &mut dyn Components,
    r: &Renderer,
    am: &mut AssetManager,
    camera: &Camera,
) {
    let crys_rad = crystal_radius(camera);
    if wizard.pos.0.center().dist(crystal.pos.0.center()) <= crys_rad {
        return;
    }

    let n = timer.add_time(dt.0);
    if n > 0 {
        let mut rand = rand::thread_rng();
        timer.length = rand.gen_range(2500..5000);
        const SPAWN_RANGE: Range<f32> = 75.0..250.0;
        let SDL_FPoint { x: wiz_x, y: wiz_y } = wizard.pos.0.center();
        for _ in 0..n {
            let mut pos = PointF {
                x: wiz_x + rand.gen_sign() as f32 * rand.gen_range(SPAWN_RANGE),
                y: wiz_y + rand.gen_sign() as f32 * rand.gen_range(SPAWN_RANGE),
            };
            if pos.dist(crystal.pos.0.center()) <= crys_rad {
                pos.x = wiz_x - (pos.x - wiz_x);
                pos.y = wiz_y - (pos.y - wiz_y);
            }
            spawn_enemy(pos, entities, r, am);
        }
    }
}
