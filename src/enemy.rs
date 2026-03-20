use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, room::{RoomState, RoomType, RoomEntity, RoomTransition}};

// ---------------------------------------------------------------------------
// Tileset sprite assets
// ---------------------------------------------------------------------------

const TILESET_FRAMES: &str = "tilesets/0x72_DungeonTilesetII_v1.7/0x72_DungeonTilesetII_v1.7/frames";

/// Pre-loaded enemy sprite frame handles from the 0x72 tileset.
#[derive(Resource)]
pub struct EnemySpriteAssets {
    pub goblin_idle: [Handle<Image>; 4],
    pub goblin_run: [Handle<Image>; 4],
    pub imp_idle: [Handle<Image>; 4],
    pub imp_run: [Handle<Image>; 4],
    pub orc_shaman_idle: [Handle<Image>; 4],
    pub orc_warrior_idle: [Handle<Image>; 4],
    pub orc_warrior_run: [Handle<Image>; 4],
    pub necromancer: [Handle<Image>; 4],
    pub muddy: [Handle<Image>; 4],
    pub wogol_idle: [Handle<Image>; 4],
    pub big_demon_idle: [Handle<Image>; 4],
    pub big_demon_run: [Handle<Image>; 4],
    pub big_zombie_idle: [Handle<Image>; 4],
    pub big_zombie_run: [Handle<Image>; 4],
}

/// Marker for the sprite child entity that renders the enemy.
#[derive(Component)]
pub struct EnemySprite;

/// Tracks which frames to cycle through and at what speed.
#[derive(Component)]
pub struct EnemyAnimState {
    pub frames: [Handle<Image>; 4],
    pub run_frames: [Handle<Image>; 4],
    pub frame: usize,
    pub timer: f32,
    pub has_run_anim: bool,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let load4 = |name: &str| -> [Handle<Image>; 4] {
            [
                asset_server.load(format!("{TILESET_FRAMES}/{name}_f0.png")),
                asset_server.load(format!("{TILESET_FRAMES}/{name}_f1.png")),
                asset_server.load(format!("{TILESET_FRAMES}/{name}_f2.png")),
                asset_server.load(format!("{TILESET_FRAMES}/{name}_f3.png")),
            ]
        };
        let assets = EnemySpriteAssets {
            goblin_idle: load4("goblin_idle_anim"),
            goblin_run: load4("goblin_run_anim"),
            imp_idle: load4("imp_idle_anim"),
            imp_run: load4("imp_run_anim"),
            orc_shaman_idle: load4("orc_shaman_idle_anim"),
            orc_warrior_idle: load4("orc_warrior_idle_anim"),
            orc_warrior_run: load4("orc_warrior_run_anim"),
            necromancer: load4("necromancer_anim"),
            muddy: load4("muddy_anim"),
            wogol_idle: load4("wogol_idle_anim"),
            big_demon_idle: load4("big_demon_idle_anim"),
            big_demon_run: load4("big_demon_run_anim"),
            big_zombie_idle: load4("big_zombie_idle_anim"),
            big_zombie_run: load4("big_zombie_run_anim"),
        };
        app.insert_resource(assets);

        app.add_event::<EnemyDefeated>()
            .add_systems(OnEnter(GameState::Playing), reset_enemy_state)
            .add_systems(
                Update,
                (
                    on_room_transition,
                    spawn_room_enemies,
                    ground_enemy_ai,
                    flying_enemy_ai,
                    turret_enemy_ai,
                    charger_enemy_ai,
                    mage_enemy_ai,
                    slime_enemy_ai,
                    ghost_enemy_ai,
                    enemy_projectile_movement,
                    count_alive_enemies,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    animate_ground_enemies,
                    animate_flying_enemies,
                    animate_charger_enemies,
                    animate_turret_eye,
                    animate_mage_staff,
                    animate_ghost_wisps,
                    animate_slime_enemies,
                    animate_enemy_sprites,
                    update_enemy_health_bars,
                    apply_elite_to_new_enemies,
                    apply_elite_buffs,
                    animate_elite_aura,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Events / Core Components
// ---------------------------------------------------------------------------

#[derive(Event)]
#[allow(dead_code)]
pub struct EnemyDefeated {
    pub position: Vec3,
    pub score: i32,
    pub gold_drop: i32,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct Enemy {
    pub health: i32,
    pub max_health: i32,
    pub contact_damage: i32,
    pub score_reward: i32,
    pub gold_drop: i32,
}

#[derive(Component)]
pub struct GroundEnemy {
    pub speed: f32,
    pub direction: f32,
    pub vy: f32,
    pub detect_range: f32,
    /// Leap attack cooldown
    pub leap_cooldown: f32,
    pub is_leaping: bool,
}

#[derive(Component)]
pub struct FlyingEnemy {
    pub amplitude: f32,
    pub wave_speed: f32,
    pub base_y: f32,
    pub phase: f32,
    pub speed_x: f32,
    /// Dive bomb cooldown
    pub dive_cooldown: f32,
    pub is_diving: bool,
    pub dive_target_y: f32,
    /// Smooth ascent back to base_y after dive
    pub is_ascending: bool,
}

#[derive(Component)]
pub struct TurretEnemy {
    pub fire_interval: f32,
    pub fire_timer: f32,
    pub projectile_speed: f32,
    /// Burst fire: shoots 3 rapid shots
    pub burst_count: i32,
    pub burst_timer: f32,
}

#[derive(Component)]
pub struct ChargerEnemy {
    pub speed: f32,
    pub detect_range: f32,
    pub charging: bool,
    pub charge_dir: f32,
    pub cooldown: f32,
    /// Shockwave on charge end
    pub did_stomp: bool,
}

#[derive(Component)]
pub struct EnemyProjectile {
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub damage: i32,
}

// ---------------------------------------------------------------------------
// Child-part marker components (kept for backwards-compat, now dead code)
// ---------------------------------------------------------------------------

/// Left leg of a GroundEnemy (Goblin).
#[derive(Component)]
pub struct GoblinLegLeft;

/// Right leg of a GroundEnemy (Goblin).
#[derive(Component)]
pub struct GoblinLegRight;

/// Left wing of a FlyingEnemy (Bat).
#[derive(Component)]
pub struct BatWingLeft;

/// Right wing of a FlyingEnemy (Bat).
#[derive(Component)]
pub struct BatWingRight;

/// Horn part of a ChargerEnemy (Bull/Boar). `side`: -1.0 = left, 1.0 = right.
#[derive(Component)]
pub struct BoarHorn {
    pub side: f32,
}

/// Rotating eye/barrel of a TurretEnemy.
#[derive(Component)]
pub struct TurretEye;

// --- New enemy types (Sprint 3) ---

/// Skeleton Mage — teleports and shoots fireballs at the player.
#[derive(Component)]
pub struct MageEnemy {
    pub teleport_cooldown: f32,
    pub cast_cooldown: f32,
    pub is_invisible: bool,
    pub invis_timer: f32,
}

/// Slime — splits into 2 smaller slimes on death.
#[derive(Component)]
pub struct SlimeEnemy {
    pub size: SlimeSize,
    pub hop_timer: f32,
    pub vy: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlimeSize {
    Large,
    Small,
}

/// Ghost — phases in and out of visibility/invulnerability.
#[derive(Component)]
pub struct GhostEnemy {
    pub phase_timer: f32,
    pub phase_duration: f32,
    pub is_phased: bool,
    pub speed: f32,
    pub direction: f32,
}

/// Marker: this enemy is currently intangible (Ghost phase / Mage teleport).
#[derive(Component)]
pub struct Intangible;

/// Child glow sprite for mage staff.
#[derive(Component)]
pub struct MageStaffGlow;

/// Child sprite for ghost wisp trail.
#[derive(Component)]
pub struct GhostWisp;

/// Marker component for boss entities (enables phase behavior).
#[derive(Component)]
pub struct BossEnemy;

/// Elite enemy modifier — stronger, glowing variant of a normal enemy.
#[derive(Component)]
pub struct EliteEnemy {
    pub modifier: EliteModifier,
}

#[derive(Clone, Copy)]
pub enum EliteModifier {
    /// Takes 50% less damage
    Armored,
    /// Moves 60% faster
    Swift,
    /// 2x contact damage
    Brutal,
}

/// Marker: elite buffs have been applied to this enemy's stats
#[derive(Component)]
struct EliteBuffApplied;

/// Pulsing glow aura child sprite on elite enemies
#[derive(Component)]
pub struct EliteAura;

/// Health bar background (dark).
#[derive(Component)]
pub struct EnemyHealthBarBg;

/// Health bar foreground (colored fill).
#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub owner: Entity,
}

// ---------------------------------------------------------------------------
// Spawn state resource
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct EnemySpawnState {
    spawned_for_room: bool,
}

fn reset_enemy_state(mut commands: Commands, resuming: Option<Res<crate::ResumingFromPause>>) {
    if resuming.is_some() { return; }
    commands.insert_resource(EnemySpawnState { spawned_for_room: false });
}

fn on_room_transition(
    mut ev: EventReader<RoomTransition>,
    mut spawn_state: ResMut<EnemySpawnState>,
) {
    for _ in ev.read() {
        spawn_state.spawned_for_room = false;
    }
}

// ---------------------------------------------------------------------------
// Room-level enemy spawning
// ---------------------------------------------------------------------------

fn spawn_room_enemies(
    mut commands: Commands,
    mut spawn_state: ResMut<EnemySpawnState>,
    room_state: Res<RoomState>,
    mut run: ResMut<RunData>,
    assets: Res<EnemySpriteAssets>,
) {
    if spawn_state.spawned_for_room { return; }
    if room_state.current_type != RoomType::Combat && room_state.current_type != RoomType::Boss {
        spawn_state.spawned_for_room = true;
        run.enemies_alive = 0;
        return;
    }

    spawn_state.spawned_for_room = true;
    let floor = room_state.floor;
    let seed = room_state.seed.wrapping_add(room_state.room_index as u64);

    if room_state.current_type == RoomType::Boss {
        spawn_boss(&mut commands, floor, &assets);
        run.enemies_alive = 1;
        return;
    }

    let enemy_count = 3 + (floor as usize).min(4);
    run.enemies_alive = enemy_count as i32;

    // Floor 1-2: original 4 types. Floor 3+: all 7 types including new enemies.
    let type_count: u64 = if floor >= 3 { 7 } else { 4 };

    for i in 0..enemy_count {
        let x = 200.0 + (i as f32) * ((ROOM_W - 300.0) / enemy_count as f32);
        let y = 120.0 + ((seed.wrapping_add(i as u64) % 3) as f32) * 80.0;
        let enemy_type = ((seed.wrapping_add(i as u64)) % type_count) as i32;

        match enemy_type {
            0 => spawn_ground_enemy(&mut commands, x, y, floor, &assets),
            1 => spawn_flying_enemy(&mut commands, x, y, floor, &assets),
            2 => spawn_turret_enemy(&mut commands, x, y, floor, &assets),
            3 => spawn_charger_enemy(&mut commands, x, y, floor, &assets),
            4 => spawn_mage_enemy(&mut commands, x, y, floor, &assets),
            5 => spawn_slime_enemy(&mut commands, x, y, floor, SlimeSize::Large, &assets),
            _ => spawn_ghost_enemy(&mut commands, x, y, floor, &assets),
        }
    }

    // Apply elite status to enemies (15% chance per enemy on floor 2+)
    if floor >= 2 {
        // We'll mark elites in a separate pass using apply_elite_to_new_enemies
    }
}

// ---------------------------------------------------------------------------
// Spawn helpers – tileset sprite children
// ---------------------------------------------------------------------------

/// GroundEnemy – Goblin
fn spawn_ground_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    let speed = 80.0 + floor as f32 * 10.0;

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 2 + floor.min(5),
            max_health: 2 + floor.min(5),
            contact_damage: 1,
            score_reward: 80,
            gold_drop: 8 + floor * 3,
        },
        GroundEnemy {
            speed,
            direction: 1.0,
            vy: 0.0,
            detect_range: 200.0,
            leap_cooldown: 2.0,
            is_leaping: false,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.goblin_idle[0].clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.goblin_idle.clone(),
                run_frames: assets.goblin_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// FlyingEnemy – Bat / Imp
fn spawn_flying_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(22.0, 18.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 100.0, Z_ENEMIES),
        Enemy {
            health: 1 + floor.min(3),
            max_health: 1 + floor.min(3),
            contact_damage: 1,
            score_reward: 110,
            gold_drop: 10 + floor * 3,
        },
        FlyingEnemy {
            amplitude: 50.0,
            wave_speed: 2.0,
            base_y: y + 100.0,
            phase: 0.0,
            speed_x: 60.0 + floor as f32 * 5.0,
            dive_cooldown: 3.5,
            is_diving: false,
            dive_target_y: 0.0,
            is_ascending: false,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.imp_idle[0].clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.imp_idle.clone(),
                run_frames: assets.imp_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// TurretEnemy – Orc Shaman (stationary)
fn spawn_turret_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    let interval = (2.0 - floor as f32 * 0.1).clamp(0.8, 2.5);

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 3 + floor.min(4),
            max_health: 3 + floor.min(4),
            contact_damage: 1,
            score_reward: 130,
            gold_drop: 12 + floor * 3,
        },
        TurretEnemy {
            fire_interval: interval,
            fire_timer: interval * 0.5,
            projectile_speed: 300.0 + floor as f32 * 20.0,
            burst_count: 0,
            burst_timer: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.orc_shaman_idle[0].clone(),
                custom_size: Some(Vec2::new(32.0, 46.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.orc_shaman_idle.clone(),
                run_frames: assets.orc_shaman_idle.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: false,
            },
        ));
    });
}

/// ChargerEnemy – Orc Warrior / Boar
fn spawn_charger_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(26.0, 22.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 3 + floor.min(5),
            max_health: 3 + floor.min(5),
            contact_damage: 2,
            score_reward: 150,
            gold_drop: 15 + floor * 3,
        },
        ChargerEnemy {
            speed: 350.0 + floor as f32 * 15.0,
            detect_range: 250.0,
            charging: false,
            charge_dir: 0.0,
            cooldown: 0.0,
            did_stomp: false,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.orc_warrior_idle[0].clone(),
                custom_size: Some(Vec2::new(40.0, 46.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.orc_warrior_idle.clone(),
                run_frames: assets.orc_warrior_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// Skeleton Mage — Necromancer
pub fn spawn_mage_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(22.0, 22.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 3 + floor.min(4),
            max_health: 3 + floor.min(4),
            contact_damage: 1,
            score_reward: 150,
            gold_drop: 15 + floor * 4,
        },
        MageEnemy {
            teleport_cooldown: 4.0,
            cast_cooldown: 2.0,
            is_invisible: false,
            invis_timer: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.necromancer[0].clone(),
                custom_size: Some(Vec2::new(36.0, 46.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.necromancer.clone(),
                run_frames: assets.necromancer.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: false,
            },
        ));
    });
}

/// Slime — bouncy gelatinous blob (Muddy)
pub fn spawn_slime_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, size: SlimeSize, assets: &EnemySpriteAssets) {
    let (sz, hp, dmg, score, gold) = match size {
        SlimeSize::Large => (20.0, 4 + floor.min(3), 1, 100, 12 + floor * 3),
        SlimeSize::Small => (12.0, 1 + floor.min(2), 1, 40, 5 + floor),
    };

    let display_sz = sz * 1.6;

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(sz, sz)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: hp,
            max_health: hp,
            contact_damage: dmg,
            score_reward: score,
            gold_drop: gold,
        },
        SlimeEnemy {
            size,
            hop_timer: 0.0,
            vy: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.muddy[0].clone(),
                custom_size: Some(Vec2::new(display_sz, display_sz)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.muddy.clone(),
                run_frames: assets.muddy.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: false,
            },
        ));
    });
}

/// Ghost — ethereal floating specter (Wogol)
pub fn spawn_ghost_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, assets: &EnemySpriteAssets) {
    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(22.0, 22.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 80.0, Z_ENEMIES),
        Enemy {
            health: 2 + floor.min(3),
            max_health: 2 + floor.min(3),
            contact_damage: 1,
            score_reward: 140,
            gold_drop: 14 + floor * 3,
        },
        GhostEnemy {
            phase_timer: 0.0,
            phase_duration: 3.0,
            is_phased: false,
            speed: 50.0 + floor as f32 * 8.0,
            direction: -1.0,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.wogol_idle[0].clone(),
                custom_size: Some(Vec2::new(36.0, 46.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.wogol_idle.clone(),
                run_frames: assets.wogol_idle.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: false,
            },
        ));
    });
}

// ---------------------------------------------------------------------------
// Boss spawning
// ---------------------------------------------------------------------------

fn spawn_boss(commands: &mut Commands, floor: i32, assets: &EnemySpriteAssets) {
    match (floor - 1) % 4 {
        1 => spawn_boss_mushroom(commands, floor, assets),
        2 => spawn_boss_lava(commands, floor, assets),
        3 => spawn_boss_root(commands, floor, assets),
        _ => spawn_boss_warlord(commands, floor, assets),
    }
}

/// Floor 2: Mushroom Titan — uses big_zombie sprite.
fn spawn_boss_mushroom(commands: &mut Commands, floor: i32, assets: &EnemySpriteAssets) {
    let hp = 14 + floor * 4;

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(48.0, 48.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 140.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 2, score_reward: 600 + floor * 120, gold_drop: 60 + floor * 25 },
        GroundEnemy { speed: 70.0 + floor as f32 * 8.0, direction: -1.0, vy: 0.0, detect_range: 350.0, leap_cooldown: 2.0, is_leaping: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.big_zombie_idle[0].clone(),
                custom_size: Some(Vec2::new(64.0, 72.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.big_zombie_idle.clone(),
                run_frames: assets.big_zombie_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// Floor 3: Lava Wyrm — uses big_demon sprite.
fn spawn_boss_lava(commands: &mut Commands, floor: i32, assets: &EnemySpriteAssets) {
    let hp = 16 + floor * 4;

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(48.0, 40.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 140.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 3, score_reward: 700 + floor * 130, gold_drop: 70 + floor * 25 },
        ChargerEnemy { speed: 280.0 + floor as f32 * 15.0, detect_range: 500.0, charging: false, charge_dir: -1.0, cooldown: 0.0, did_stomp: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.big_demon_idle[0].clone(),
                custom_size: Some(Vec2::new(64.0, 72.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.big_demon_idle.clone(),
                run_frames: assets.big_demon_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// Floor 4: Root Ancient — uses big_zombie sprite.
fn spawn_boss_root(commands: &mut Commands, floor: i32, assets: &EnemySpriteAssets) {
    let hp = 20 + floor * 5;

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(52.0, 52.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 150.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 2, score_reward: 800 + floor * 150, gold_drop: 80 + floor * 30 },
        GroundEnemy { speed: 60.0 + floor as f32 * 6.0, direction: -1.0, vy: 0.0, detect_range: 400.0, leap_cooldown: 3.0, is_leaping: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.big_zombie_idle[0].clone(),
                custom_size: Some(Vec2::new(64.0, 72.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.big_zombie_idle.clone(),
                run_frames: assets.big_zombie_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

/// Floor 1 (default): Crimson Warlord — uses big_demon sprite.
fn spawn_boss_warlord(commands: &mut Commands, floor: i32, assets: &EnemySpriteAssets) {
    let hp = 10 + floor * 3;

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(48.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 140.0, Z_ENEMIES),
        Enemy {
            health: hp,
            max_health: hp,
            contact_damage: 2,
            score_reward: 500 + floor * 100,
            gold_drop: 50 + floor * 20,
        },
        GroundEnemy {
            speed: 100.0 + floor as f32 * 10.0,
            direction: -1.0,
            vy: 0.0,
            detect_range: 400.0,
            leap_cooldown: 1.5,
            is_leaping: false,
        },
        BossEnemy,
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.big_demon_idle[0].clone(),
                custom_size: Some(Vec2::new(64.0, 72.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            EnemySprite,
            EnemyAnimState {
                frames: assets.big_demon_idle.clone(),
                run_frames: assets.big_demon_run.clone(),
                frame: 0,
                timer: 0.0,
                has_run_anim: true,
            },
        ));
    });
}

// ---------------------------------------------------------------------------
// AI systems
// ---------------------------------------------------------------------------

fn ground_enemy_ai(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GroundEnemy, &Enemy, Option<&BossEnemy>, Option<&EliteEnemy>)>,
    player_q: Query<&Transform, (With<Player>, Without<GroundEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (_entity, mut tf, mut ge, enemy, is_boss, elite) in &mut query {
        let dt = time.delta_secs();
        ge.leap_cooldown = (ge.leap_cooldown - dt).max(0.0);

        // Boss phase modifiers
        let hp_ratio = enemy.health as f32 / enemy.max_health.max(1) as f32;
        let (phase_speed_mult, phase_leap_cd_mult): (f32, f32) = if is_boss.is_some() {
            if hp_ratio <= 0.25 {
                // Phase 3: enraged — very fast, rapid leaps
                (1.8, 0.4)
            } else if hp_ratio <= 0.5 {
                // Phase 2: aggressive — faster, shorter cooldowns
                (1.4, 0.65)
            } else {
                (1.0, 1.0)
            }
        } else {
            (1.0, 1.0)
        };

        ge.vy -= GRAVITY * 0.5 * dt;
        ge.vy = ge.vy.max(-600.0);

        if let Some(pp) = player_pos {
            let dx = (pp.x - tf.translation.x).abs();
            let dy = (pp.y - tf.translation.y).abs();
            if dx < ge.detect_range {
                ge.direction = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            }
            // Leap attack: when close and on ground, jump toward player
            let on_ground = tf.translation.y <= TILE_SIZE + 12.0;
            let leap_cd = GOBLIN_LEAP_COOLDOWN * phase_leap_cd_mult;
            if dx < GOBLIN_LEAP_RANGE && dy < 80.0 && on_ground && ge.leap_cooldown <= 0.0 && !ge.is_leaping {
                ge.is_leaping = true;
                ge.leap_cooldown = leap_cd;
                ge.vy = GOBLIN_LEAP_SPEED * phase_speed_mult.min(1.3_f32);
                ge.direction = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            }

            // Boss Phase 3: AoE slam when landing from leap
            if is_boss.is_some() && hp_ratio <= 0.25 && ge.is_leaping && ge.vy < 0.0 {
                let on_ground_now = tf.translation.y <= TILE_SIZE + 12.0;
                if on_ground_now {
                    // Spawn shockwaves in both directions
                    for dir in [-1.0_f32, 1.0] {
                        commands.spawn((
                            Sprite {
                                color: Color::srgba(0.9, 0.3, 0.1, 0.7),
                                custom_size: Some(Vec2::new(24.0, 12.0)),
                                ..default()
                            },
                            Transform::from_xyz(tf.translation.x, TILE_SIZE + 5.0, Z_PROJECTILES),
                            EnemyProjectile {
                                vx: dir * BOAR_SHOCKWAVE_SPEED * 1.5,
                                vy: 0.0,
                                lifetime: 1.0,
                                damage: 2,
                            },
                            RoomEntity,
                            PlayingEntity,
                        ));
                    }
                }
            }
        }

        let elite_speed = if elite.map_or(false, |e| matches!(e.modifier, EliteModifier::Swift)) { 1.6 } else { 1.0 };
        let speed_mult = if ge.is_leaping { GOBLIN_LEAP_SPEED_MULT } else { 1.0 };
        tf.translation.x += ge.direction * ge.speed * speed_mult * phase_speed_mult * elite_speed * dt;
        tf.translation.y += ge.vy * dt;

        let margin = TILE_SIZE + 12.0;
        if tf.translation.x < margin || tf.translation.x > ROOM_W - margin {
            ge.direction *= -1.0;
            tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
        }

        if tf.translation.y < TILE_SIZE + 10.0 {
            tf.translation.y = TILE_SIZE + 10.0;
            ge.vy = 0.0;
            ge.is_leaping = false;
        }
    }
}

fn flying_enemy_ai(
    mut query: Query<(&mut Transform, &mut FlyingEnemy, Option<&EliteEnemy>)>,
    player_q: Query<&Transform, (With<Player>, Without<FlyingEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (mut tf, mut fe, elite) in &mut query {
        let dt = time.delta_secs();
        fe.dive_cooldown = (fe.dive_cooldown - dt).max(0.0);

        if fe.is_diving {
            // Swoop down toward target
            let target_y = fe.dive_target_y;
            tf.translation.y += (target_y - tf.translation.y).signum() * BAT_DIVE_SPEED * dt;
            if (tf.translation.y - target_y).abs() < 10.0 || tf.translation.y < TILE_SIZE + 20.0 {
                fe.is_diving = false;
                fe.is_ascending = true;
                fe.dive_cooldown = BAT_DIVE_COOLDOWN;
            }
        } else if fe.is_ascending {
            // Smooth ascent back to base_y with lerp deceleration
            let ascend_speed = 120.0;
            let diff = fe.base_y - tf.translation.y;
            if diff.abs() < 3.0 {
                tf.translation.y = fe.base_y;
                fe.is_ascending = false;
                fe.phase = 0.0;
            } else {
                // Lerp: faster when far, slower when near target
                let speed = ascend_speed * (diff.abs() / 100.0).clamp(0.3, 1.0);
                tf.translation.y += diff.signum() * speed * dt;
            }
        } else {
            fe.phase += fe.wave_speed * dt;
            tf.translation.y = fe.base_y + fe.phase.sin() * fe.amplitude;

            // Initiate dive when player is below and in range
            if let Some(pp) = player_pos {
                let dx = (pp.x - tf.translation.x).abs();
                if dx < BAT_DIVE_RANGE && pp.y < tf.translation.y - 40.0 && fe.dive_cooldown <= 0.0 {
                    fe.is_diving = true;
                    fe.dive_target_y = pp.y;
                }
            }
        }

        let elite_speed = if elite.map_or(false, |e| matches!(e.modifier, EliteModifier::Swift)) { 1.6 } else { 1.0 };
        if let Some(pp) = player_pos {
            let dir = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            let speed = if fe.is_diving { fe.speed_x * 2.0 } else { fe.speed_x };
            tf.translation.x += dir * speed * elite_speed * dt;
        }

        let margin = TILE_SIZE + 14.0;
        tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
        tf.translation.y = tf.translation.y.max(TILE_SIZE + 10.0);
    }
}

/// Helper: compute aimed projectile velocity toward a target (or left if no target).
fn aim_at_target(origin: Vec3, target: Option<Vec3>, speed: f32) -> (f32, f32) {
    if let Some(pp) = target {
        let diff = pp - origin;
        let len = diff.length().max(1.0);
        (diff.x / len * speed, diff.y / len * speed)
    } else {
        (-speed, 0.0)
    }
}

/// Helper: spawn a turret projectile.
fn spawn_turret_projectile(
    commands: &mut Commands,
    pos: Vec3,
    vx: f32, vy: f32,
    color: Color,
    size: f32,
    lifetime: f32,
    damage: i32,
) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, Z_PROJECTILES),
        EnemyProjectile { vx, vy, lifetime, damage },
        RoomEntity,
        PlayingEntity,
    ));
}

fn turret_enemy_ai(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut TurretEnemy)>,
    player_q: Query<&Transform, (With<Player>, Without<TurretEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (tf, mut turret) in &mut query {
        let dt = time.delta_secs();

        // Handle burst fire (rapid follow-up shots)
        if turret.burst_count > 0 {
            turret.burst_timer -= dt;
            if turret.burst_timer <= 0.0 {
                turret.burst_count -= 1;
                turret.burst_timer = TURRET_BURST_INTERVAL;
                let (vx, vy) = aim_at_target(tf.translation, player_pos, turret.projectile_speed);
                spawn_turret_projectile(&mut commands, tf.translation, vx, vy,
                    Color::srgb(1.0, 0.3, 0.1), 6.0, 2.5, 1);
            }
            continue;
        }

        turret.fire_timer -= dt;
        if turret.fire_timer <= 0.0 {
            turret.fire_timer = turret.fire_interval;
            let (vx, vy) = aim_at_target(tf.translation, player_pos, turret.projectile_speed);
            spawn_turret_projectile(&mut commands, tf.translation, vx, vy,
                Color::srgb(1.0, 0.5, 0.2), 8.0, 3.0, 1);

            // Burst fire on higher-difficulty turrets (shorter fire interval = higher floors)
            if turret.fire_interval < 2.0 {
                turret.burst_count = TURRET_BURST_COUNT;
                turret.burst_timer = TURRET_BURST_INTERVAL;
            }
        }
    }
}

fn charger_enemy_ai(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut ChargerEnemy)>,
    player_q: Query<&Transform, (With<Player>, Without<ChargerEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (mut tf, mut charger) in &mut query {
        let dt = time.delta_secs();
        charger.cooldown = (charger.cooldown - dt).max(0.0);

        if charger.charging {
            tf.translation.x += charger.charge_dir * charger.speed * dt;
            // Flip sprite to face charge direction
            tf.scale.x = charger.charge_dir;
            let margin = TILE_SIZE + 14.0;
            if tf.translation.x < margin || tf.translation.x > ROOM_W - margin {
                charger.charging = false;
                charger.cooldown = 1.5;
                tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);

                // Stomp shockwave on charge end — spawns a ground projectile
                if !charger.did_stomp {
                    charger.did_stomp = true;
                    // Shockwave travels along the ground in both directions
                    for dir in [-1.0_f32, 1.0] {
                        commands.spawn((
                            Sprite {
                                color: Color::srgba(0.8, 0.5, 0.2, 0.7),
                                custom_size: Some(Vec2::new(20.0, 10.0)),
                                ..default()
                            },
                            Transform::from_xyz(tf.translation.x, TILE_SIZE + 5.0, Z_PROJECTILES),
                            EnemyProjectile {
                                vx: dir * BOAR_SHOCKWAVE_SPEED,
                                vy: 0.0,
                                lifetime: 1.2,
                                damage: 2,
                            },
                            RoomEntity,
                            PlayingEntity,
                        ));
                    }
                }
            }
        } else if charger.cooldown <= 0.0 {
            charger.did_stomp = false;
            if let Some(pp) = player_pos {
                let dx = (pp.x - tf.translation.x).abs();
                let dy = (pp.y - tf.translation.y).abs();
                if dx < charger.detect_range && dy < 60.0 {
                    charger.charging = true;
                    charger.charge_dir = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
                }
            }
        }
    }
}

fn enemy_projectile_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut EnemyProjectile)>,
    time: Res<Time>,
) {
    for (entity, mut tf, mut proj) in &mut query {
        tf.translation.x += proj.vx * time.delta_secs();
        tf.translation.y += proj.vy * time.delta_secs();
        proj.lifetime -= time.delta_secs();
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn count_alive_enemies(
    enemy_q: Query<&Enemy>,
    mut run: ResMut<RunData>,
) {
    run.enemies_alive = enemy_q.iter().count() as i32;
}

// ---------------------------------------------------------------------------
// Enemy health bars
// ---------------------------------------------------------------------------

fn update_enemy_health_bars(
    mut commands: Commands,
    enemy_q: Query<(Entity, &Enemy, &Transform, Option<&Children>)>,
    mut fill_q: Query<(&EnemyHealthBarFill, &mut Transform, &mut Sprite), Without<Enemy>>,
    bg_q: Query<&EnemyHealthBarBg>,
) {
    let bar_w = 24.0;
    let bar_h = 3.0;

    for (entity, _enemy, _e_tf, children) in &enemy_q {
        // Check if this enemy already has a health bar
        let has_bar = children.map_or(false, |ch| {
            ch.iter().any(|c| bg_q.get(*c).is_ok())
        });

        if !has_bar {
            // Spawn health bar as child of enemy
            commands.entity(entity).with_children(|parent| {
                // Background (dark)
                parent.spawn((
                    Sprite {
                        color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                        custom_size: Some(Vec2::new(bar_w + 2.0, bar_h + 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 18.0, 0.9),
                    EnemyHealthBarBg,
                ));
                // Fill (green/yellow/red based on HP ratio)
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.2, 0.8, 0.2),
                        custom_size: Some(Vec2::new(bar_w, bar_h)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 18.0, 0.95),
                    EnemyHealthBarFill { owner: entity },
                ));
            });
        }
    }

    // Update existing health bar fills
    for (fill, mut tf, mut sprite) in &mut fill_q {
        if let Ok((_, enemy, _, _)) = enemy_q.get(fill.owner) {
            let ratio = (enemy.health as f32 / enemy.max_health as f32).clamp(0.0, 1.0);
            let w = bar_w * ratio;
            sprite.custom_size = Some(Vec2::new(w, bar_h));
            // Offset so bar shrinks from right
            tf.translation.x = -(bar_w - w) / 2.0;
            // Color: green > yellow > red
            sprite.color = if ratio > 0.5 {
                Color::srgb(0.2, 0.8, 0.2)
            } else if ratio > 0.25 {
                Color::srgb(0.9, 0.8, 0.1)
            } else {
                Color::srgb(0.9, 0.2, 0.1)
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Elite enemy systems
// ---------------------------------------------------------------------------

/// Marks newly spawned enemies as elite (15% chance, floor 2+, not bosses).
fn apply_elite_to_new_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Enemy), (Without<EliteEnemy>, Without<BossEnemy>)>,
    room_state: Res<RoomState>,
    spawn_state: Res<EnemySpawnState>,
) {
    // Only run the frame enemies were spawned
    if !spawn_state.spawned_for_room || room_state.floor < 2 {
        return;
    }

    for (entity, _enemy) in &query {
        // Use entity index as pseudo-random seed
        let hash = entity.index().wrapping_mul(2654435761);
        let roll = (hash % 100) as i32;
        if roll < 15 {
            let modifier = match hash % 3 {
                0 => EliteModifier::Armored,
                1 => EliteModifier::Swift,
                _ => EliteModifier::Brutal,
            };
            let (aura_color, _label) = match modifier {
                EliteModifier::Armored => (Color::srgba(0.3, 0.5, 0.9, 0.4), "Armored"),
                EliteModifier::Swift   => (Color::srgba(0.2, 0.9, 0.3, 0.4), "Swift"),
                EliteModifier::Brutal  => (Color::srgba(0.9, 0.2, 0.1, 0.4), "Brutal"),
            };
            // Buff the enemy: Armored gets 2x HP, Brutal gets 2x contact damage
            // Swift modifier is applied in AI systems
            commands.entity(entity).insert(EliteEnemy { modifier });
            // Add pulsing aura child
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Sprite {
                        color: aura_color,
                        custom_size: Some(Vec2::new(36.0, 36.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, -0.1),
                    EliteAura,
                ));
            });
        }
    }
}

/// Apply stat buffs to newly marked elite enemies.
fn apply_elite_buffs(
    mut commands: Commands,
    mut query: Query<(Entity, &EliteEnemy, &mut Enemy), Without<EliteBuffApplied>>,
) {
    for (entity, elite, mut enemy) in &mut query {
        match elite.modifier {
            EliteModifier::Armored => {
                enemy.health *= 2;
                enemy.max_health *= 2;
            }
            EliteModifier::Brutal => {
                enemy.contact_damage *= 2;
                enemy.health = (enemy.health as f32 * 1.5) as i32;
                enemy.max_health = (enemy.max_health as f32 * 1.5) as i32;
            }
            EliteModifier::Swift => {
                enemy.health = (enemy.health as f32 * 1.3) as i32;
                enemy.max_health = (enemy.max_health as f32 * 1.3) as i32;
            }
        }
        // Elites drop more gold
        enemy.gold_drop = (enemy.gold_drop as f32 * 1.5) as i32;
        enemy.score_reward *= 2;
        commands.entity(entity).insert(EliteBuffApplied);
    }
}

/// Pulse elite aura size and apply elite modifiers to enemy stats.
fn animate_elite_aura(
    mut aura_q: Query<(&mut Transform, &mut Sprite), With<EliteAura>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (mut tf, mut sprite) in &mut aura_q {
        let pulse = 1.0 + (t * 3.0).sin() * 0.15;
        tf.scale = Vec3::splat(pulse);
        let c = sprite.color.to_srgba();
        let alpha = 0.25 + (t * 2.0).sin().abs() * 0.2;
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ---------------------------------------------------------------------------
// Tileset sprite animation
// ---------------------------------------------------------------------------

fn animate_enemy_sprites(
    enemy_q: Query<(&Children, Option<&GroundEnemy>, Option<&ChargerEnemy>, Option<&FlyingEnemy>)>,
    mut sprite_q: Query<(&mut EnemyAnimState, &mut Sprite), With<EnemySprite>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (children, ground, charger, flying) in &enemy_q {
        // Determine if enemy is "moving" for run animation
        let moving = ground.map_or(false, |g| g.speed > 0.1)
            || charger.map_or(false, |c| c.charging)
            || flying.is_some();

        for &child in children.iter() {
            let Ok((mut anim, mut sprite)) = sprite_q.get_mut(child) else { continue };
            anim.timer += dt;
            if anim.timer >= 0.125 {
                anim.timer -= 0.125;
                anim.frame = (anim.frame + 1) % 4;
            }
            let frames = if moving && anim.has_run_anim { &anim.run_frames } else { &anim.frames };
            sprite.image = frames[anim.frame].clone();
        }
    }
}

// ---------------------------------------------------------------------------
// Legacy animation systems (dead code — child parts no longer spawned)
// ---------------------------------------------------------------------------

/// Bob goblin legs up/down when the parent GroundEnemy is moving.
fn animate_ground_enemies(
    ground_q: Query<(&GroundEnemy, &Children)>,
    mut leg_q: Query<(&mut Transform, Option<&GoblinLegLeft>, Option<&GoblinLegRight>)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (ge, children) in &ground_q {
        let moving = ge.speed > 0.1;
        for &child in children.iter() {
            if let Ok((mut tf, is_left, is_right)) = leg_q.get_mut(child) {
                if is_left.is_some() || is_right.is_some() {
                    if moving {
                        let phase = if is_left.is_some() { 0.0f32 } else { std::f32::consts::PI };
                        let bob = (t * 8.0 + phase).sin() * 3.0;
                        tf.translation.y = -8.0 + bob;
                    } else {
                        tf.translation.y = -8.0;
                    }
                }
            }
        }
    }
}

/// Flap bat wings by rotating them back and forth.
fn animate_flying_enemies(
    flying_q: Query<(&FlyingEnemy, &Children)>,
    mut wing_q: Query<(&mut Transform, Option<&BatWingLeft>, Option<&BatWingRight>)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (_fe, children) in &flying_q {
        for &child in children.iter() {
            if let Ok((mut tf, is_left, is_right)) = wing_q.get_mut(child) {
                if is_left.is_some() {
                    let angle = (t * 6.0).sin() * 0.5;
                    tf.rotation = Quat::from_rotation_z(angle);
                } else if is_right.is_some() {
                    let angle = -(t * 6.0).sin() * 0.5;
                    tf.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}

/// Tilt boar horns forward when the charger is charging.
fn animate_charger_enemies(
    charger_q: Query<(&ChargerEnemy, &Children)>,
    mut horn_q: Query<(&mut Transform, &BoarHorn)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (charger, children) in &charger_q {
        for &child in children.iter() {
            if let Ok((mut tf, horn)) = horn_q.get_mut(child) {
                if charger.charging {
                    let vib = (t * 20.0).sin() * 0.04;
                    let tilt = -0.35 + vib;
                    tf.rotation = Quat::from_rotation_z(tilt * charger.charge_dir * horn.side);
                } else {
                    tf.rotation = Quat::from_rotation_z(0.0);
                }
            }
        }
    }
}

/// Rotate the turret eye to point toward the player.
fn animate_turret_eye(
    turret_q: Query<(&Transform, &Children), With<TurretEnemy>>,
    mut eye_q: Query<&mut Transform, (With<TurretEye>, Without<TurretEnemy>, Without<Player>)>,
    player_q: Query<&Transform, (With<Player>, Without<TurretEnemy>, Without<TurretEye>)>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (turret_tf, children) in &turret_q {
        for &child in children.iter() {
            if let Ok(mut eye_tf) = eye_q.get_mut(child) {
                if let Some(pp) = player_pos {
                    let diff = pp - turret_tf.translation;
                    let angle = diff.y.atan2(diff.x);
                    eye_tf.rotation = Quat::from_rotation_z(angle);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// New enemy AI systems (Sprint 3)
// ---------------------------------------------------------------------------

/// Skeleton Mage AI: teleport around, cast fireballs at player.
fn mage_enemy_ai(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut MageEnemy, &mut Sprite), Without<Player>>,
    player_q: Query<&Transform, (With<Player>, Without<MageEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (_entity, mut tf, mut mage, mut sprite) in &mut query {
        let dt = time.delta_secs();
        mage.teleport_cooldown = (mage.teleport_cooldown - dt).max(0.0);
        mage.cast_cooldown = (mage.cast_cooldown - dt).max(0.0);

        // Fade in/out during teleport
        if mage.is_invisible {
            mage.invis_timer -= dt;
            if mage.invis_timer <= 0.0 {
                mage.is_invisible = false;
                // Teleport to a new position near the player
                if let Some(pp) = player_pos {
                    let offset = if pp.x > ROOM_W / 2.0 { -120.0 } else { 120.0 };
                    tf.translation.x = (pp.x + offset).clamp(TILE_SIZE + 20.0, ROOM_W - TILE_SIZE - 20.0);
                    tf.translation.y = (pp.y + 40.0).clamp(TILE_SIZE + 20.0, ROOM_H - TILE_SIZE * 2.0);
                }
            }
        }

        // Set alpha based on visibility
        let target_alpha = if mage.is_invisible { 0.0 } else { 1.0 };
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, target_alpha);

        // Teleport when player gets close
        if let Some(pp) = player_pos {
            let dist = (pp.xy() - tf.translation.xy()).length();
            if dist < 80.0 && mage.teleport_cooldown <= 0.0 && !mage.is_invisible {
                mage.is_invisible = true;
                mage.invis_timer = 0.5;
                mage.teleport_cooldown = 4.0;
                // Temporarily make intangible
                commands.entity(_entity).insert(Intangible);
            }
        }

        // Remove intangible when visible again
        if !mage.is_invisible {
            commands.entity(_entity).remove::<Intangible>();
        }

        // Cast fireball at player
        if mage.cast_cooldown <= 0.0 && !mage.is_invisible {
            mage.cast_cooldown = 2.5;
            if let Some(pp) = player_pos {
                let (vx, vy) = aim_at_target(tf.translation, Some(pp), 250.0);
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.5, 0.15, 0.9),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_xyz(tf.translation.x, tf.translation.y, Z_PROJECTILES),
                    EnemyProjectile { vx, vy, lifetime: 2.5, damage: 2 },
                    RoomEntity,
                    PlayingEntity,
                ));
            }
        }
    }
}

/// Slime AI: hop toward player, splits on death handled by slime_split_on_death.
fn slime_enemy_ai(
    mut query: Query<(&mut Transform, &mut SlimeEnemy)>,
    player_q: Query<&Transform, (With<Player>, Without<SlimeEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (mut tf, mut slime) in &mut query {
        let dt = time.delta_secs();
        slime.hop_timer -= dt;

        // Apply gravity
        slime.vy -= GRAVITY * 0.5 * dt;
        slime.vy = slime.vy.max(-500.0);
        tf.translation.y += slime.vy * dt;

        // Ground check
        if tf.translation.y < TILE_SIZE + 10.0 {
            tf.translation.y = TILE_SIZE + 10.0;
            slime.vy = 0.0;

            // Hop toward player every 1.2 seconds
            if slime.hop_timer <= 0.0 {
                slime.hop_timer = match slime.size {
                    SlimeSize::Large => 1.2,
                    SlimeSize::Small => 0.8,
                };
                slime.vy = 300.0;

                if let Some(pp) = player_pos {
                    let dir = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
                    let hop_speed = match slime.size {
                        SlimeSize::Large => 80.0,
                        SlimeSize::Small => 120.0,
                    };
                    tf.translation.x += dir * hop_speed;
                }
            }
        }

        let margin = TILE_SIZE + 10.0;
        tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
    }
}

/// When a large slime is killed, spawn 2 small slimes.
pub fn slime_split_on_death(
    commands: &mut Commands,
    position: Vec3,
    slime_size: SlimeSize,
    floor: i32,
    run: &mut crate::RunData,
    assets: &EnemySpriteAssets,
) {
    if slime_size == SlimeSize::Large {
        for offset in [-20.0_f32, 20.0] {
            spawn_slime_enemy(
                commands,
                (position.x + offset).clamp(TILE_SIZE + 10.0, ROOM_W - TILE_SIZE - 10.0),
                position.y + 10.0,
                floor,
                SlimeSize::Small,
                assets,
            );
            run.enemies_alive += 1;
        }
    }
}

/// Ghost AI: floats toward player, phases in/out.
fn ghost_enemy_ai(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut GhostEnemy, &mut Sprite)>,
    player_q: Query<&Transform, (With<Player>, Without<GhostEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (entity, mut tf, mut ghost, mut sprite) in &mut query {
        let dt = time.delta_secs();
        ghost.phase_timer += dt;

        // Phase cycle: visible for phase_duration, then invisible for phase_duration/2
        let cycle = ghost.phase_duration + ghost.phase_duration * 0.5;
        let t = ghost.phase_timer % cycle;
        let was_phased = ghost.is_phased;
        ghost.is_phased = t > ghost.phase_duration;

        // Handle intangible component
        if ghost.is_phased && !was_phased {
            commands.entity(entity).insert(Intangible);
        } else if !ghost.is_phased && was_phased {
            commands.entity(entity).remove::<Intangible>();
        }

        // Set alpha based on phase
        let alpha = if ghost.is_phased {
            0.15 + (time.elapsed_secs() * 4.0).sin().abs() * 0.1
        } else {
            0.5 + (time.elapsed_secs() * 2.0).sin().abs() * 0.2
        };
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);

        // Float toward player
        if let Some(pp) = player_pos {
            ghost.direction = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
        }
        tf.translation.x += ghost.direction * ghost.speed * dt;

        // Flip sprite
        tf.scale.x = ghost.direction;

        // Vertical bobbing
        let bob = (time.elapsed_secs() * 1.5 + entity.index() as f32).sin() * 15.0;
        tf.translation.y = 120.0 + bob;

        let margin = TILE_SIZE + 10.0;
        tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
    }
}

/// Animate mage staff glow pulsing.
fn animate_mage_staff(
    time: Res<Time>,
    mut glow_q: Query<(&mut Transform, &mut Sprite), With<MageStaffGlow>>,
) {
    let t = time.elapsed_secs();
    for (mut tf, mut sprite) in &mut glow_q {
        let pulse = 1.0 + (t * 3.0).sin() * 0.2;
        tf.scale = Vec3::splat(pulse);
        let c = sprite.color.to_srgba();
        let alpha = 0.3 + (t * 2.5).sin().abs() * 0.3;
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

/// Animate ghost wisps undulating.
fn animate_ghost_wisps(
    time: Res<Time>,
    mut wisp_q: Query<&mut Transform, With<GhostWisp>>,
) {
    let t = time.elapsed_secs();
    for mut tf in &mut wisp_q {
        let wave = (t * 2.0 + tf.translation.x * 0.1).sin() * 3.0;
        tf.translation.x += wave * 0.02;
    }
}

/// Animate slime squash/stretch on hop.
fn animate_slime_enemies(
    mut query: Query<(&SlimeEnemy, &mut Transform)>,
) {
    for (slime, mut tf) in &mut query {
        if slime.vy > 10.0 {
            // Stretching upward
            tf.scale = Vec3::new(0.85, 1.2, 1.0);
        } else if slime.vy < -10.0 {
            // Squashing on descent
            tf.scale = Vec3::new(1.15, 0.85, 1.0);
        } else {
            // Resting
            tf.scale = Vec3::ONE;
        }
    }
}
