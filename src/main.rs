mod constants;
mod title;
mod player;
mod room;
mod enemy;
mod combat;
mod spell;
mod hud;
mod shop;
mod camera;
mod effects;
mod animation;
mod loot;

use bevy::prelude::*;
use constants::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    WellIntro,
    Playing,
    Paused,
    Shop,
    GameOver,
}

/// Per-run data (resets each run)
#[derive(Resource)]
pub struct RunData {
    pub score: i32,
    pub gold: i32,
    pub time: f32,
    pub current_floor: i32,
    pub current_room: i32,
    pub rooms_cleared: i32,
    pub enemies_alive: i32,
}

impl Default for RunData {
    fn default() -> Self {
        Self {
            score: 0,
            gold: 0,
            time: 0.0,
            current_floor: 1,
            current_room: 1,
            rooms_cleared: 0,
            enemies_alive: 0,
        }
    }
}

/// Persistent meta-progression (survives between runs)
#[derive(Resource)]
pub struct MetaProgression {
    pub max_health_bonus: i32,
    pub starting_gold: i32,
    pub runs_completed: i32,
    pub best_floor: i32,
}

impl Default for MetaProgression {
    fn default() -> Self {
        Self {
            max_health_bonus: 0,
            starting_gold: 0,
            runs_completed: 0,
            best_floor: 0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Dawnroot".into(),
                        resolution: (VIEWPORT_W, VIEWPORT_H).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .insert_resource(RunData::default())
        .insert_resource(MetaProgression::default())
        .add_plugins((
            title::TitlePlugin,
            player::PlayerPlugin,
            room::RoomPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            spell::SpellPlugin,
            hud::HudPlugin,
            shop::ShopPlugin,
            camera::CameraPlugin,
            effects::EffectsPlugin,
            animation::AnimationPlugin,
            loot::LootPlugin,
        ))
        .add_systems(OnEnter(GameState::Playing), setup_run)
        .add_systems(OnExit(GameState::Playing), cleanup_run)
        .add_systems(OnEnter(GameState::GameOver), on_game_over)
        .add_systems(
            Update,
            update_run_time.run_if(in_state(GameState::Playing)),
        )
        .run();
}

#[derive(Component)]
pub struct PlayingEntity;

fn setup_run(mut run: ResMut<RunData>, meta: Res<MetaProgression>) {
    *run = RunData {
        gold: meta.starting_gold,
        ..default()
    };
}

fn cleanup_run(mut commands: Commands, q: Query<Entity, With<PlayingEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn on_game_over(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Title);
}

fn update_run_time(mut run: ResMut<RunData>, time: Res<Time>) {
    run.time += time.delta_secs();
}
