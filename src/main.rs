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
mod hazards;
mod dialogue;
mod death_screen;
mod floor_complete;

use bevy::prelude::*;
use constants::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    WellIntro,
    Playing,
    Paused,
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
    pub enemies_killed: i32,
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
            enemies_killed: 0,
        }
    }
}

/// Persistent meta-progression (survives between runs)
#[derive(Resource, serde::Serialize, serde::Deserialize)]
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

/// Save slot data written to disk per slot
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SaveSlotData {
    pub floor: i32,
    pub gold: i32,
    pub score: i32,
    pub time_played: f32,
    pub max_health: i32,
    pub max_mana: f32,
    pub spells: [bool; 4],
    pub enemies_killed: i32,
}

/// Which save slot is currently active (0-2).
#[derive(Resource)]
pub struct ActiveSaveSlot(pub usize);

/// Global game font handle.
#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

/// When present, the next OnEnter(Playing) should restore from this data
/// instead of starting a fresh run.
#[derive(Resource)]
pub struct LoadedSave(pub SaveSlotData);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Dawnroot".into(),
                        resolution: (VIEWPORT_W, VIEWPORT_H).into(),
                        resizable: true,
                        mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.06, 0.04)))
        .insert_resource(RunData::default())
        .insert_resource(load_meta())
        .insert_resource(ActiveSaveSlot(0))
        .add_systems(Startup, load_game_font)
        .add_plugins((
            title::TitlePlugin,
            player::PlayerPlugin,
            room::RoomPlugin,
            enemy::EnemyPlugin,
            combat::CombatPlugin,
            spell::SpellPlugin,
            hud::HudPlugin,
            shop::ShopPlugin,
        ))
        .add_plugins((
            camera::CameraPlugin,
            effects::EffectsPlugin,
            animation::AnimationPlugin,
            loot::LootPlugin,
            hazards::HazardsPlugin,
            dialogue::DialoguePlugin,
            death_screen::DeathScreenPlugin,
            floor_complete::FloorCompletePlugin,
        ))
        .add_systems(OnEnter(GameState::Playing), (setup_run, apply_loaded_save).chain())
        .add_systems(OnExit(GameState::Playing), cleanup_run)
        .add_systems(OnEnter(GameState::GameOver), on_game_over)
        .add_systems(
            Update,
            (
                update_run_time,
                check_player_died,
                tick_deferred_save_cleanup,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, toggle_fullscreen)
        .run();
}

#[derive(Component)]
pub struct PlayingEntity;

fn setup_run(
    mut run: ResMut<RunData>,
    meta: Res<MetaProgression>,
    loaded: Option<Res<LoadedSave>>,
) {
    if let Some(save) = loaded {
        *run = RunData {
            gold: save.0.gold,
            score: save.0.score,
            time: save.0.time_played,
            current_floor: save.0.floor,
            current_room: 1,
            rooms_cleared: 0,
            enemies_alive: 0,
            enemies_killed: save.0.enemies_killed,
        };
    } else {
        *run = RunData {
            gold: meta.starting_gold,
            ..default()
        };
    }
}

/// If a LoadedSave resource exists, apply it to player/spells then remove it.
fn apply_loaded_save(
    mut commands: Commands,
    loaded: Option<Res<LoadedSave>>,
) {
    if loaded.is_some() {
        // Remove the resource so it doesn't apply again.
        // Player and spell restoration is handled by spawn_player/init_spell_slots
        // reading the LoadedSave resource, or by dedicated systems.
        // We defer removal by one frame so other OnEnter systems can read it.
        commands.insert_resource(DeferredSaveCleanup(2));
    }
}

/// Counts down frames before removing LoadedSave resource.
#[derive(Resource)]
struct DeferredSaveCleanup(u8);

fn cleanup_run(
    mut commands: Commands,
    q: Query<Entity, With<PlayingEntity>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<LoadedSave>();
    commands.remove_resource::<DeferredSaveCleanup>();
}

fn on_game_over(
    mut meta: ResMut<MetaProgression>,
    run: Res<RunData>,
    slot: Res<ActiveSaveSlot>,
) {
    meta.runs_completed += 1;
    if run.current_floor > meta.best_floor {
        meta.best_floor = run.current_floor;
    }
    save_meta(&meta);
    // Delete the save slot on death (permadeath)
    delete_slot(slot.0);
}

/// Centralized death handler: any PlayerDied event → GameOver.
fn check_player_died(
    mut ev: EventReader<player::PlayerDied>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in ev.read() {
        next_state.set(GameState::GameOver);
        break;
    }
}

fn load_game_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/PressStart2P-Regular.ttf");
    commands.insert_resource(GameFont(font));
}

fn update_run_time(mut run: ResMut<RunData>, time: Res<Time>) {
    run.time += time.delta_secs();
}

fn toggle_fullscreen(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    if keys.just_pressed(KeyCode::F11) {
        let mut window = windows.single_mut();
        window.mode = match window.mode {
            bevy::window::WindowMode::BorderlessFullscreen(_) => {
                bevy::window::WindowMode::Windowed
            }
            _ => bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        };
    }
}

fn tick_deferred_save_cleanup(
    mut commands: Commands,
    cleanup: Option<ResMut<DeferredSaveCleanup>>,
) {
    if let Some(mut c) = cleanup {
        if c.0 == 0 {
            commands.remove_resource::<LoadedSave>();
            commands.remove_resource::<DeferredSaveCleanup>();
        } else {
            c.0 -= 1;
        }
    }
}

// ─── Save / Load ──────────────────────────────────────────────────────────────

fn meta_path() -> std::path::PathBuf {
    std::path::PathBuf::from("dawnroot_meta.json")
}

pub fn slot_path(slot: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("dawnroot_slot_{}.json", slot))
}

fn save_meta(meta: &MetaProgression) {
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let _ = std::fs::write(meta_path(), json);
    }
}

fn load_meta() -> MetaProgression {
    // Try new path first, fall back to old path
    if let Ok(data) = std::fs::read_to_string(meta_path()) {
        serde_json::from_str(&data).unwrap_or_default()
    } else if let Ok(data) = std::fs::read_to_string("dawnroot_save.json") {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        MetaProgression::default()
    }
}

pub fn save_slot(slot: usize, data: &SaveSlotData) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(slot_path(slot), json);
    }
}

pub fn load_slot(slot: usize) -> Option<SaveSlotData> {
    let data = std::fs::read_to_string(slot_path(slot)).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn delete_slot(slot: usize) {
    let _ = std::fs::remove_file(slot_path(slot));
}
