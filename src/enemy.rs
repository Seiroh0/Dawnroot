use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, room::{RoomState, RoomType, RoomEntity, RoomTransition}};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
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
// Child-part marker components for animation
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

fn reset_enemy_state(mut commands: Commands) {
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
        spawn_boss(&mut commands, floor);
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
            0 => spawn_ground_enemy(&mut commands, x, y, floor),
            1 => spawn_flying_enemy(&mut commands, x, y, floor),
            2 => spawn_turret_enemy(&mut commands, x, y, floor),
            3 => spawn_charger_enemy(&mut commands, x, y, floor),
            4 => spawn_mage_enemy(&mut commands, x, y, floor),
            5 => spawn_slime_enemy(&mut commands, x, y, floor, SlimeSize::Large),
            _ => spawn_ghost_enemy(&mut commands, x, y, floor),
        }
    }

    // Apply elite status to enemies (15% chance per enemy on floor 2+)
    if floor >= 2 {
        // We'll mark elites in a separate pass using apply_elite_to_new_enemies
    }
}

// ---------------------------------------------------------------------------
// Spawn helpers – multi-part procedural pixel sprites
// ---------------------------------------------------------------------------

/// GroundEnemy – Goblin
/// Layout (all offsets relative to root at logical center):
///   Shadow           22x4   offset (0, -13)   dark translucent (ground shadow)
///   Body             20x14  offset (0,  2)     red-brown
///   Body highlight    8x3   offset (-3, 7)     lighter rim (top-left catch-light)
///   Belly patch      10x6   offset (0, -1)     cream/tan (underbelly)
///   Shoulder L        7x5   offset (-12, 4)    dark armor gray
///   Shoulder R        7x5   offset ( 12, 4)    dark armor gray
///   Belt              20x3  offset (0, -4)     dark brown leather
///   Belt buckle        4x3  offset (0, -4)     dull gold
///   Head             16x14  offset (0, 14)     slightly lighter red-brown
///   Head shadow       14x4  offset (0, 10)     darker underside of head
///   Brow ridge L       5x2  offset (-4, 20)    dark brow crease
///   Brow ridge R       5x2  offset ( 4, 20)    dark brow crease
///   Eye left           3x3  offset (-4, 17)    glowing yellow
///   Eye pupil L        1x2  offset (-4, 16)    black slit pupil
///   Eye right          3x3  offset ( 4, 17)    glowing yellow
///   Eye pupil R        1x2  offset ( 4, 16)    black slit pupil
///   Nose               4x2  offset (0, 14)     darker nostril patch
///   Mouth/fang row     8x2  offset (0, 12)     dark gum line
///   Fang left          2x3  offset (-2, 11)    dirty white fang
///   Fang right         2x3  offset ( 2, 11)    dirty white fang
///   Scar               6x1  offset (-2, 16)    lighter scar slash
///   Club handle       4x14  offset (14, 5)     dark brown handle
///   Club knob         8x8   offset (14, 14)    darker knob head
///   Club spike        3x3   offset (14, 18)    metal spike tip
///   Club grip wrap    4x3   offset (14, 0)     lighter wrap band
///   Leg left          6x8   offset (-4, -8)    red-brown  (animated)
///   Boot left         6x4   offset (-4, -14)   very dark brown boot
///   Leg right         6x8   offset ( 4, -8)    red-brown  (animated)
///   Boot right        6x4   offset ( 4, -14)   very dark brown boot
fn spawn_ground_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let speed = 80.0 + floor as f32 * 10.0;

    let body_color      = Color::srgb(0.65, 0.28, 0.20);
    let body_hi_color   = Color::srgb(0.78, 0.40, 0.30);
    let head_color      = Color::srgb(0.72, 0.34, 0.25);
    let head_shadow     = Color::srgb(0.50, 0.20, 0.14);
    let belly_color     = Color::srgb(0.82, 0.72, 0.52);
    let eye_color       = Color::srgb(1.0,  0.95, 0.10);
    let pupil_color     = Color::srgb(0.05, 0.02, 0.02);
    let brow_color      = Color::srgb(0.30, 0.10, 0.06);
    let nose_color      = Color::srgb(0.42, 0.16, 0.10);
    let gum_color       = Color::srgb(0.38, 0.08, 0.08);
    let fang_color      = Color::srgb(0.90, 0.88, 0.75);
    let scar_color      = Color::srgb(0.85, 0.55, 0.45);
    let shoulder_color  = Color::srgb(0.28, 0.28, 0.30);
    let belt_color      = Color::srgb(0.30, 0.18, 0.08);
    let buckle_color    = Color::srgb(0.70, 0.60, 0.20);
    let club_color      = Color::srgb(0.35, 0.20, 0.10);
    let club_hi_color   = Color::srgb(0.50, 0.32, 0.16);
    let spike_color     = Color::srgb(0.60, 0.62, 0.65);
    let leg_color       = Color::srgb(0.60, 0.25, 0.18);
    let boot_color      = Color::srgb(0.22, 0.12, 0.06);
    let shadow_color    = Color::srgba(0.0, 0.0, 0.0, 0.35);

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
        // Ground shadow blob (behind everything)
        parent.spawn((
            Sprite { color: shadow_color, custom_size: Some(Vec2::new(22.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -13.0, 0.0),
        ));
        // Body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(20.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.1),
        ));
        // Body top-left highlight (rim light)
        parent.spawn((
            Sprite { color: body_hi_color, custom_size: Some(Vec2::new(8.0, 3.0)), ..default() },
            Transform::from_xyz(-3.0, 8.0, 0.15),
        ));
        // Belly / underbelly patch
        parent.spawn((
            Sprite { color: belly_color, custom_size: Some(Vec2::new(10.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, -1.0, 0.15),
        ));
        // Left shoulder armor plate
        parent.spawn((
            Sprite { color: shoulder_color, custom_size: Some(Vec2::new(7.0, 5.0)), ..default() },
            Transform::from_xyz(-12.0, 4.0, 0.18),
        ));
        // Right shoulder armor plate
        parent.spawn((
            Sprite { color: shoulder_color, custom_size: Some(Vec2::new(7.0, 5.0)), ..default() },
            Transform::from_xyz(12.0, 4.0, 0.18),
        ));
        // Belt strap
        parent.spawn((
            Sprite { color: belt_color, custom_size: Some(Vec2::new(20.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, -4.0, 0.19),
        ));
        // Belt buckle
        parent.spawn((
            Sprite { color: buckle_color, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, -4.0, 0.20),
        ));
        // Head
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(16.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 14.0, 0.1),
        ));
        // Head underside shadow
        parent.spawn((
            Sprite { color: head_shadow, custom_size: Some(Vec2::new(14.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, 10.0, 0.15),
        ));
        // Brow ridge left (angry crease)
        parent.spawn((
            Sprite { color: brow_color, custom_size: Some(Vec2::new(5.0, 2.0)), ..default() },
            Transform::from_xyz(-4.0, 20.0, 0.22),
        ));
        // Brow ridge right
        parent.spawn((
            Sprite { color: brow_color, custom_size: Some(Vec2::new(5.0, 2.0)), ..default() },
            Transform::from_xyz(4.0, 20.0, 0.22),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(-4.0, 17.0, 0.2),
        ));
        // Slit pupil left
        parent.spawn((
            Sprite { color: pupil_color, custom_size: Some(Vec2::new(1.0, 2.0)), ..default() },
            Transform::from_xyz(-4.0, 17.0, 0.25),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(4.0, 17.0, 0.2),
        ));
        // Slit pupil right
        parent.spawn((
            Sprite { color: pupil_color, custom_size: Some(Vec2::new(1.0, 2.0)), ..default() },
            Transform::from_xyz(4.0, 17.0, 0.25),
        ));
        // Nostril patch
        parent.spawn((
            Sprite { color: nose_color, custom_size: Some(Vec2::new(4.0, 2.0)), ..default() },
            Transform::from_xyz(0.0, 14.0, 0.22),
        ));
        // Gum / mouth line
        parent.spawn((
            Sprite { color: gum_color, custom_size: Some(Vec2::new(8.0, 2.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.22),
        ));
        // Left fang
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(2.0, 3.0)), ..default() },
            Transform::from_xyz(-2.0, 11.0, 0.23),
        ));
        // Right fang
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(2.0, 3.0)), ..default() },
            Transform::from_xyz(2.0, 11.0, 0.23),
        ));
        // Scar slash across face
        parent.spawn((
            Sprite { color: scar_color, custom_size: Some(Vec2::new(6.0, 1.0)), ..default() },
            Transform::from_xyz(-2.0, 16.0, 0.24),
        ));
        // Club handle (shaft)
        parent.spawn((
            Sprite { color: club_color, custom_size: Some(Vec2::new(4.0, 14.0)), ..default() },
            Transform::from_xyz(14.0, 5.0, 0.1),
        ));
        // Club knob head
        parent.spawn((
            Sprite { color: club_color, custom_size: Some(Vec2::new(8.0, 8.0)), ..default() },
            Transform::from_xyz(14.0, 14.0, 0.12),
        ));
        // Club knob highlight
        parent.spawn((
            Sprite { color: club_hi_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(12.0, 16.0, 0.14),
        ));
        // Club spike on top
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(14.0, 19.0, 0.15),
        ));
        // Club grip wrap band
        parent.spawn((
            Sprite { color: club_hi_color, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(14.0, 0.0, 0.14),
        ));
        // Leg left (animated)
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(-4.0, -8.0, 0.1),
            GoblinLegLeft,
        ));
        // Boot left
        parent.spawn((
            Sprite { color: boot_color, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
            Transform::from_xyz(-4.0, -14.0, 0.12),
        ));
        // Leg right (animated)
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(4.0, -8.0, 0.1),
            GoblinLegRight,
        ));
        // Boot right
        parent.spawn((
            Sprite { color: boot_color, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
            Transform::from_xyz(4.0, -14.0, 0.12),
        ));
    });
}

/// FlyingEnemy – Bat
/// Layout:
///   Wing left          16x8   offset (-16, 2)   dark purple         (animated rotate)
///   Wing vein left      2x6   offset (-18, 1)   even darker vein    (membrane detail)
///   Wing left tip       6x4   offset (-26, 2)   near-black claw tip
///   Wing right         16x8   offset ( 16, 2)   dark purple         (animated rotate)
///   Wing vein right     2x6   offset ( 18, 1)   even darker vein
///   Wing right tip      6x4   offset ( 26, 2)   near-black claw tip
///   Body               18x12  center             purple
///   Body belly patch   10x6   offset (0, -2)     lighter underbelly
///   Body highlight      6x3   offset (-3, 4)     rim light
///   Fur head tuft       8x5   offset (0, 8)      slightly lighter purple
///   Ear left            4x6   offset (-6, 12)    dark purple ear
///   Ear inner left      2x4   offset (-6, 13)    pink inner ear
///   Ear right           4x6   offset ( 6, 12)    dark purple ear
///   Ear inner right     2x4   offset ( 6, 13)    pink inner ear
///   Nose leaf           5x3   offset (0, 3)      dark nose ornament
///   Eye left            3x3   offset (-4, 3)     red
///   Eye glow left       5x5   offset (-4, 3)     dim red glow behind eye
///   Eye right           3x3   offset ( 4, 3)     red
///   Eye glow right      5x5   offset ( 4, 3)     dim red glow
///   Mouth               6x2   offset (0, 0)      dark gum
///   Fang left           2x3   offset (-2, -1)    white fang
///   Fang right          2x3   offset ( 2, -1)    white fang
fn spawn_flying_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let body_color    = Color::srgb(0.55, 0.20, 0.70);
    let body_hi_color = Color::srgb(0.68, 0.32, 0.82);
    let belly_color   = Color::srgb(0.72, 0.52, 0.80);
    let eye_color     = Color::srgb(0.95, 0.15, 0.15);
    let eye_glow      = Color::srgba(0.80, 0.05, 0.05, 0.45);
    let wing_color    = Color::srgb(0.38, 0.10, 0.55);
    let vein_color    = Color::srgb(0.22, 0.04, 0.35);
    let tip_color     = Color::srgb(0.12, 0.02, 0.18);
    let ear_color     = Color::srgb(0.35, 0.08, 0.50);
    let ear_inner     = Color::srgb(0.80, 0.40, 0.55);
    let fur_color     = Color::srgb(0.62, 0.26, 0.76);
    let nose_color    = Color::srgb(0.28, 0.06, 0.40);
    let gum_color     = Color::srgb(0.35, 0.05, 0.08);
    let fang_color    = Color::srgb(0.92, 0.90, 0.80);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Wing left (animated - spawned first so body renders on top)
        parent.spawn((
            Sprite { color: wing_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() },
            Transform::from_xyz(-16.0, 2.0, 0.0),
            BatWingLeft,
        ));
        // Wing left membrane vein
        parent.spawn((
            Sprite { color: vein_color, custom_size: Some(Vec2::new(2.0, 6.0)), ..default() },
            Transform::from_xyz(-18.0, 1.0, 0.01),
        ));
        // Wing left claw tip
        parent.spawn((
            Sprite { color: tip_color, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
            Transform::from_xyz(-26.0, 2.0, 0.01),
        ));
        // Wing right (animated)
        parent.spawn((
            Sprite { color: wing_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() },
            Transform::from_xyz(16.0, 2.0, 0.0),
            BatWingRight,
        ));
        // Wing right membrane vein
        parent.spawn((
            Sprite { color: vein_color, custom_size: Some(Vec2::new(2.0, 6.0)), ..default() },
            Transform::from_xyz(18.0, 1.0, 0.01),
        ));
        // Wing right claw tip
        parent.spawn((
            Sprite { color: tip_color, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
            Transform::from_xyz(26.0, 2.0, 0.01),
        ));
        // Body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(18.0, 12.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Belly underbelly patch
        parent.spawn((
            Sprite { color: belly_color, custom_size: Some(Vec2::new(10.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, -2.0, 0.15),
        ));
        // Body rim highlight
        parent.spawn((
            Sprite { color: body_hi_color, custom_size: Some(Vec2::new(6.0, 3.0)), ..default() },
            Transform::from_xyz(-3.0, 4.0, 0.15),
        ));
        // Head fur tuft
        parent.spawn((
            Sprite { color: fur_color, custom_size: Some(Vec2::new(8.0, 5.0)), ..default() },
            Transform::from_xyz(0.0, 8.0, 0.12),
        ));
        // Ear left
        parent.spawn((
            Sprite { color: ear_color, custom_size: Some(Vec2::new(4.0, 6.0)), ..default() },
            Transform::from_xyz(-6.0, 12.0, 0.12),
        ));
        // Ear left inner
        parent.spawn((
            Sprite { color: ear_inner, custom_size: Some(Vec2::new(2.0, 4.0)), ..default() },
            Transform::from_xyz(-6.0, 13.0, 0.14),
        ));
        // Ear right
        parent.spawn((
            Sprite { color: ear_color, custom_size: Some(Vec2::new(4.0, 6.0)), ..default() },
            Transform::from_xyz(6.0, 12.0, 0.12),
        ));
        // Ear right inner
        parent.spawn((
            Sprite { color: ear_inner, custom_size: Some(Vec2::new(2.0, 4.0)), ..default() },
            Transform::from_xyz(6.0, 13.0, 0.14),
        ));
        // Nose leaf ornament
        parent.spawn((
            Sprite { color: nose_color, custom_size: Some(Vec2::new(5.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, 3.0, 0.18),
        ));
        // Eye glow left (diffuse behind eye)
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(-4.0, 3.0, 0.19),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(-4.0, 3.0, 0.2),
        ));
        // Eye glow right
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(4.0, 3.0, 0.19),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(4.0, 3.0, 0.2),
        ));
        // Gum / mouth line
        parent.spawn((
            Sprite { color: gum_color, custom_size: Some(Vec2::new(6.0, 2.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.22),
        ));
        // Fang left
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(2.0, 3.0)), ..default() },
            Transform::from_xyz(-2.0, -1.0, 0.23),
        ));
        // Fang right
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(2.0, 3.0)), ..default() },
            Transform::from_xyz(2.0, -1.0, 0.23),
        ));
    });
}

/// TurretEnemy – Stone Tower
/// Layout:
///   Base shadow        26x4   offset (0, -13)    cast shadow
///   Stone base         24x20  offset (0, -2)     mid gray stone
///   Base highlight     22x3   offset (0, 6)      light catch on top face
///   Left mortar line    2x18  offset (-8, -2)    dark mortar seam
///   Right mortar line   2x18  offset ( 8, -2)    dark mortar seam
///   Horiz mortar line  22x2   offset (0, -8)     horizontal mortar band
///   Arrow slit          4x8   offset (0, -5)     dark window opening
///   Arrow slit rim      6x10  offset (0, -5)     darker stone arch surround
///   Base front face    24x5   offset (0, -14)    slightly lighter base ledge
///   Left crenel         6x6   offset (-9, 12)    darker gray merlon
///   Left crenel face    4x3   offset (-9, 10)    front face of merlon
///   Mid crenel          6x6   offset ( 0, 12)    darker gray
///   Mid crenel face     4x3   offset ( 0, 10)    front face
///   Right crenel        6x6   offset ( 9, 12)    darker gray
///   Right crenel face   4x3   offset ( 9, 10)    front face
///   Eye socket ring    12x8   offset (0, 2)      dark socket surround
///   Eye inner glow     10x6   offset (0, 2)      deep orange glow fill
///   Eye barrel         10x6   offset (0, 2)      bright barrel tip  (TurretEye - rotates)
///   Barrel tip ring     4x4   offset (6, 2)      lighter barrel end ring
fn spawn_turret_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let interval = (2.0 - floor as f32 * 0.1).clamp(0.8, 2.5);

    let stone_color    = Color::srgb(0.50, 0.52, 0.55);
    let stone_hi_color = Color::srgb(0.65, 0.67, 0.70);
    let stone_ledge    = Color::srgb(0.58, 0.60, 0.63);
    let mortar_color   = Color::srgb(0.28, 0.29, 0.32);
    let crenel_color   = Color::srgb(0.35, 0.37, 0.40);
    let crenel_face    = Color::srgb(0.44, 0.46, 0.50);
    let slit_color     = Color::srgb(0.10, 0.10, 0.12);
    let slit_rim       = Color::srgb(0.28, 0.29, 0.32);
    let socket_color   = Color::srgb(0.18, 0.12, 0.05);
    let eye_glow_color = Color::srgb(0.80, 0.35, 0.02);
    let eye_color      = Color::srgb(1.0,  0.55, 0.05);
    let barrel_rim     = Color::srgb(1.0,  0.75, 0.40);
    let shadow_color   = Color::srgba(0.0, 0.0, 0.0, 0.35);

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
        // Ground cast shadow
        parent.spawn((
            Sprite { color: shadow_color, custom_size: Some(Vec2::new(26.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -13.0, 0.0),
        ));
        // Stone base body
        parent.spawn((
            Sprite { color: stone_color, custom_size: Some(Vec2::new(24.0, 20.0)), ..default() },
            Transform::from_xyz(0.0, -2.0, 0.1),
        ));
        // Top face catch-light
        parent.spawn((
            Sprite { color: stone_hi_color, custom_size: Some(Vec2::new(22.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, 6.0, 0.15),
        ));
        // Vertical mortar seam left
        parent.spawn((
            Sprite { color: mortar_color, custom_size: Some(Vec2::new(2.0, 18.0)), ..default() },
            Transform::from_xyz(-8.0, -2.0, 0.13),
        ));
        // Vertical mortar seam right
        parent.spawn((
            Sprite { color: mortar_color, custom_size: Some(Vec2::new(2.0, 18.0)), ..default() },
            Transform::from_xyz(8.0, -2.0, 0.13),
        ));
        // Horizontal mortar band
        parent.spawn((
            Sprite { color: mortar_color, custom_size: Some(Vec2::new(22.0, 2.0)), ..default() },
            Transform::from_xyz(0.0, -8.0, 0.13),
        ));
        // Arrow slit surround / rim
        parent.spawn((
            Sprite { color: slit_rim, custom_size: Some(Vec2::new(6.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, -5.0, 0.14),
        ));
        // Arrow slit opening
        parent.spawn((
            Sprite { color: slit_color, custom_size: Some(Vec2::new(4.0, 8.0)), ..default() },
            Transform::from_xyz(0.0, -5.0, 0.16),
        ));
        // Base front ledge
        parent.spawn((
            Sprite { color: stone_ledge, custom_size: Some(Vec2::new(24.0, 5.0)), ..default() },
            Transform::from_xyz(0.0, -14.0, 0.16),
        ));
        // Crenellations (3 merlons on top)
        for cx in [-9i32, 0, 9] {
            // Merlon body
            parent.spawn((
                Sprite { color: crenel_color, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
                Transform::from_xyz(cx as f32, 12.0, 0.1),
            ));
            // Merlon front face highlight
            parent.spawn((
                Sprite { color: crenel_face, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
                Transform::from_xyz(cx as f32, 10.0, 0.14),
            ));
        }
        // Eye socket / dark surround ring
        parent.spawn((
            Sprite { color: socket_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.17),
        ));
        // Inner glow fill
        parent.spawn((
            Sprite { color: eye_glow_color, custom_size: Some(Vec2::new(10.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.18),
        ));
        // Glowing barrel (rotates toward player)
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(10.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.2),
            TurretEye,
        ));
        // Barrel end ring / muzzle highlight
        parent.spawn((
            Sprite { color: barrel_rim, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
            Transform::from_xyz(6.0, 2.0, 0.21),
        ));
    });
}

/// ChargerEnemy – Bull / Boar
/// Layout:
///   Shadow           28x4   offset (0, -13)    ground shadow
///   Body             26x16  offset (0, 0)       orange-brown
///   Body underside   20x5   offset (0, -6)      darker belly underside
///   Belly patch      14x7   offset (0, -3)      cream underbelly
///   Body highlight    8x3   offset (-6, 6)      rim light
///   Spine ridge L     4x12  offset (-10, 4)     dark ridge stripe
///   Spine ridge R     4x12  offset (-6,  4)     slightly lighter
///   Shoulder muscle  10x8   offset (-8, 4)      bulge highlight
///   Head             18x14  offset (14, 2)      slightly lighter
///   Head shadow       16x4  offset (14, -2)     chin shadow
///   Nostril           4x3   offset (21, 1)      dark nostril patch
///   Snort marking     6x2   offset (19, 5)      angular angry marking
///   Eye socket ring   5x5   offset (19, 6)      dark eye ring
///   Eye               3x3   offset (19, 6)      angry red
///   Hoof front L      6x5   offset (-10, -10)   dark brown
///   Hoof front R      6x5   offset (-2,  -10)   dark brown
///   Hoof rear L       6x5   offset (-18, -10)   dark brown (rear legs)
///   Hoof rear R       6x5   offset (-26, -10)   dark brown
///   Leg upper front   6x8   offset (-6, -4)     body-color leg
///   Leg upper rear    6x8   offset (-22, -4)    body-color leg
///   Tail nub          5x4   offset (-16, 6)     dark tail stub
///   Horn upper       12x5   offset (18, 9)      cream/tan  (BoarHorn animated)
///   Horn tip upper    4x3   offset (28, 9)      darker horn tip
///   Horn lower       12x5   offset (18, 4)      cream/tan  (BoarHorn animated)
///   Horn tip lower    4x3   offset (28, 4)      darker horn tip
fn spawn_charger_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let body_color      = Color::srgb(0.72, 0.40, 0.14);
    let body_hi_color   = Color::srgb(0.85, 0.52, 0.22);
    let body_dark       = Color::srgb(0.52, 0.28, 0.08);
    let belly_color     = Color::srgb(0.88, 0.78, 0.58);
    let head_color      = Color::srgb(0.80, 0.48, 0.18);
    let head_shadow     = Color::srgb(0.58, 0.32, 0.10);
    let hoof_color      = Color::srgb(0.28, 0.18, 0.08);
    let leg_color       = Color::srgb(0.65, 0.36, 0.12);
    let horn_color      = Color::srgb(0.90, 0.85, 0.65);
    let horn_tip        = Color::srgb(0.65, 0.55, 0.35);
    let eye_color       = Color::srgb(0.95, 0.15, 0.15);
    let socket_color    = Color::srgb(0.25, 0.05, 0.05);
    let nostril_color   = Color::srgb(0.38, 0.20, 0.06);
    let marking_color   = Color::srgb(0.48, 0.18, 0.04);
    let ridge_dark      = Color::srgb(0.48, 0.26, 0.08);
    let ridge_mid       = Color::srgb(0.60, 0.34, 0.10);
    let shoulder_color  = Color::srgb(0.80, 0.48, 0.20);
    let tail_color      = Color::srgb(0.38, 0.20, 0.06);
    let shadow_color    = Color::srgba(0.0, 0.0, 0.0, 0.35);

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
        // Ground shadow blob
        parent.spawn((
            Sprite { color: shadow_color, custom_size: Some(Vec2::new(28.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -13.0, 0.0),
        ));
        // Body main
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(26.0, 16.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Body underside dark strip
        parent.spawn((
            Sprite { color: body_dark, custom_size: Some(Vec2::new(20.0, 5.0)), ..default() },
            Transform::from_xyz(0.0, -6.0, 0.12),
        ));
        // Belly cream underbelly
        parent.spawn((
            Sprite { color: belly_color, custom_size: Some(Vec2::new(14.0, 7.0)), ..default() },
            Transform::from_xyz(0.0, -3.0, 0.14),
        ));
        // Body top rim highlight
        parent.spawn((
            Sprite { color: body_hi_color, custom_size: Some(Vec2::new(8.0, 3.0)), ..default() },
            Transform::from_xyz(-6.0, 6.0, 0.14),
        ));
        // Spine ridge left dark stripe
        parent.spawn((
            Sprite { color: ridge_dark, custom_size: Some(Vec2::new(4.0, 12.0)), ..default() },
            Transform::from_xyz(-10.0, 4.0, 0.13),
        ));
        // Spine ridge right lighter stripe
        parent.spawn((
            Sprite { color: ridge_mid, custom_size: Some(Vec2::new(4.0, 12.0)), ..default() },
            Transform::from_xyz(-6.0, 4.0, 0.13),
        ));
        // Shoulder muscle bulge
        parent.spawn((
            Sprite { color: shoulder_color, custom_size: Some(Vec2::new(10.0, 8.0)), ..default() },
            Transform::from_xyz(-8.0, 4.0, 0.14),
        ));
        // Rear upper leg
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(-22.0, -4.0, 0.09),
        ));
        // Rear hoof left
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-18.0, -10.0, 0.09),
        ));
        // Rear hoof right
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-26.0, -10.0, 0.09),
        ));
        // Tail stub
        parent.spawn((
            Sprite { color: tail_color, custom_size: Some(Vec2::new(5.0, 4.0)), ..default() },
            Transform::from_xyz(-16.0, 6.0, 0.09),
        ));
        // Head (forward-facing)
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(18.0, 14.0)), ..default() },
            Transform::from_xyz(14.0, 2.0, 0.1),
        ));
        // Head chin shadow
        parent.spawn((
            Sprite { color: head_shadow, custom_size: Some(Vec2::new(16.0, 4.0)), ..default() },
            Transform::from_xyz(14.0, -2.0, 0.14),
        ));
        // Angry face marking / slash
        parent.spawn((
            Sprite { color: marking_color, custom_size: Some(Vec2::new(6.0, 2.0)), ..default() },
            Transform::from_xyz(19.0, 5.0, 0.16),
        ));
        // Nostril patch
        parent.spawn((
            Sprite { color: nostril_color, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(21.0, 1.0, 0.16),
        ));
        // Front upper leg
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(-6.0, -4.0, 0.11),
        ));
        // Front hoof left
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-10.0, -10.0, 0.11),
        ));
        // Front hoof right
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-2.0, -10.0, 0.11),
        ));
        // Eye socket dark ring
        parent.spawn((
            Sprite { color: socket_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(19.0, 6.0, 0.28),
        ));
        // Angry eye
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(19.0, 6.0, 0.30),
        ));
        // Horn upper (animated)
        parent.spawn((
            Sprite { color: horn_color, custom_size: Some(Vec2::new(12.0, 5.0)), ..default() },
            Transform::from_xyz(18.0, 9.0, 0.2),
            BoarHorn { side: 1.0 },
        ));
        // Horn upper tip
        parent.spawn((
            Sprite { color: horn_tip, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(28.0, 9.0, 0.2),
        ));
        // Horn lower (animated)
        parent.spawn((
            Sprite { color: horn_color, custom_size: Some(Vec2::new(12.0, 5.0)), ..default() },
            Transform::from_xyz(18.0, 4.0, 0.2),
            BoarHorn { side: -1.0 },
        ));
        // Horn lower tip
        parent.spawn((
            Sprite { color: horn_tip, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(28.0, 4.0, 0.2),
        ));
    });
}

/// Boss – Lich-like dark creature (48x48 root hitbox)
/// Layout:
///   Dark aura          52x52  offset (0, 0)       near-black translucent halo
///   Robe/skirt base    32x14  offset (0, -18)      very dark red
///   Robe hem trim      36x4   offset (0, -24)      slightly lighter hem edge
///   Robe inner lining  18x10  offset (0, -17)      dark crimson inner
///   Robe lining edge    2x10  offset (±9, -17)     even darker fold lines
///   Main body          36x28  offset (0, -4)       dark crimson
///   Body shadow strip  30x6   offset (0, -14)      very dark bottom body
///   Body highlight     14x5   offset (-8, 6)       lighter rim catch-light
///   Rib line 1          2x16  offset (-6, -4)      bone-colored rib
///   Rib line 2          2x16  offset (-2, -4)      bone-colored rib
///   Rib line 3          2x16  offset ( 2, -4)      bone-colored rib
///   Rib line 4          2x16  offset ( 6, -4)      bone-colored rib
///   Pauldron left      12x8   offset (-22, 6)      dark maroon shoulder pad
///   Pauldron spike L    4x6   offset (-24, 12)     near-black spike on pad
///   Pauldron right     12x8   offset ( 22, 6)      dark maroon shoulder pad
///   Pauldron spike R    4x6   offset ( 24, 12)     near-black spike on pad
///   Claw left arm      12x8   offset (-22, -2)     dark maroon
///   Claw finger 1 L     3x5   offset (-28, -4)     bone claw
///   Claw finger 2 L     3x5   offset (-31, -1)     bone claw (spread)
///   Claw right arm     12x8   offset ( 22, -2)     dark maroon
///   Claw finger 1 R     3x5   offset ( 28, -4)     bone claw
///   Claw finger 2 R     3x5   offset ( 31, -1)     bone claw
///   Head               24x22  offset (0,  18)      deep red
///   Head shadow strip  22x6   offset (0,  12)      very dark chin underside
///   Head highlight     10x4   offset (-4, 26)      rim catch-light on skull
///   Cheekbone L         6x3   offset (-8, 20)      prominent cheek ridge
///   Cheekbone R         6x3   offset ( 8, 20)      prominent cheek ridge
///   Nose cavity         5x4   offset (0,  16)      very dark nose socket
///   Jaw line           18x3   offset (0,  12)      dark jaw edge
///   Teeth row          16x3   offset (0,  10)      off-white tooth row
///   Fang center L       3x5   offset (-3, 9)       long fang
///   Fang center R       3x5   offset ( 3, 9)       long fang
///   Eye socket L       10x10  offset (-6, 22)      very dark socket depression
///   Eye glow L          7x7   offset (-6, 22)      magenta diffuse glow
///   Eye left            5x5   offset (-6, 22)      bright magenta eye
///   Eye socket R       10x10  offset ( 6, 22)      very dark socket
///   Eye glow R          7x7   offset ( 6, 22)      magenta diffuse glow
///   Eye right           5x5   offset ( 6, 22)      bright magenta eye
///   Crown band         26x4   offset (0, 29)       dark crown base band
///   Spike outer L      10x5   offset (-14, 30)     near-black spike
///   Spike inner L       8x4   offset (-8,  32)     inner spike
///   Spike mid           8x10  offset (  0, 33)     center tall spike
///   Spike mid gem       4x4   offset (  0, 38)     glowing gem atop center spike
///   Spike inner R       8x4   offset ( 8,  32)     inner spike
///   Spike outer R      10x5   offset ( 14, 30)     near-black spike
fn spawn_boss(commands: &mut Commands, floor: i32) {
    match (floor - 1) % 4 {
        1 => spawn_boss_mushroom(commands, floor),
        2 => spawn_boss_lava(commands, floor),
        3 => spawn_boss_root(commands, floor),
        _ => spawn_boss_warlord(commands, floor),
    }
}

/// Floor 2: Mushroom Titan — massive fungal beast, slower but tanky, uses ChargerEnemy AI.
fn spawn_boss_mushroom(commands: &mut Commands, floor: i32) {
    let hp = 14 + floor * 4;
    let cap_color = Color::srgb(0.55, 0.22, 0.18);
    let cap_spots = Color::srgb(0.85, 0.80, 0.65);
    let stem_color = Color::srgb(0.75, 0.70, 0.58);
    let stem_dark = Color::srgb(0.55, 0.50, 0.40);
    let eye_color = Color::srgb(0.9, 0.95, 0.1);
    let mouth_color = Color::srgb(0.20, 0.08, 0.05);
    let aura_color = Color::srgba(0.2, 0.5, 0.1, 0.25);

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(48.0, 48.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 140.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 2, score_reward: 600 + floor * 120, gold_drop: 60 + floor * 25 },
        GroundEnemy { speed: 70.0 + floor as f32 * 8.0, direction: -1.0, vy: 0.0, detect_range: 350.0, leap_cooldown: 2.0, is_leaping: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((Sprite { color: aura_color, custom_size: Some(Vec2::new(56.0, 56.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.0)));
        // Thick stem body
        parent.spawn((Sprite { color: stem_color, custom_size: Some(Vec2::new(30.0, 28.0)), ..default() }, Transform::from_xyz(0.0, -6.0, 0.1)));
        parent.spawn((Sprite { color: stem_dark, custom_size: Some(Vec2::new(26.0, 8.0)), ..default() }, Transform::from_xyz(0.0, -16.0, 0.12)));
        // Mushroom cap
        parent.spawn((Sprite { color: cap_color, custom_size: Some(Vec2::new(44.0, 22.0)), ..default() }, Transform::from_xyz(0.0, 14.0, 0.14)));
        parent.spawn((Sprite { color: cap_color, custom_size: Some(Vec2::new(36.0, 16.0)), ..default() }, Transform::from_xyz(0.0, 24.0, 0.14)));
        // Cap spots
        for (dx, dy) in [(-12.0, 18.0), (8.0, 20.0), (-4.0, 26.0), (14.0, 14.0)] {
            parent.spawn((Sprite { color: cap_spots, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() }, Transform::from_xyz(dx, dy, 0.16)));
        }
        // Eyes
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() }, Transform::from_xyz(-8.0, 6.0, 0.18)));
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() }, Transform::from_xyz(8.0, 6.0, 0.18)));
        // Mouth
        parent.spawn((Sprite { color: mouth_color, custom_size: Some(Vec2::new(10.0, 4.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.18)));
    });
}

/// Floor 3: Lava Wyrm — serpentine fire beast, uses ChargerEnemy AI for charges.
fn spawn_boss_lava(commands: &mut Commands, floor: i32) {
    let hp = 16 + floor * 4;
    let body_color = Color::srgb(0.70, 0.20, 0.05);
    let body_hi = Color::srgb(0.90, 0.40, 0.10);
    let belly_color = Color::srgb(0.95, 0.60, 0.15);
    let eye_color = Color::srgb(1.0, 0.9, 0.2);
    let horn_color = Color::srgb(0.35, 0.10, 0.05);
    let flame_color = Color::srgba(1.0, 0.5, 0.1, 0.6);

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(48.0, 40.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 140.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 3, score_reward: 700 + floor * 130, gold_drop: 70 + floor * 25 },
        ChargerEnemy { speed: 280.0 + floor as f32 * 15.0, detect_range: 500.0, charging: false, charge_dir: -1.0, cooldown: 0.0, did_stomp: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        // Flame aura
        parent.spawn((Sprite { color: flame_color, custom_size: Some(Vec2::new(52.0, 44.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.0)));
        // Body
        parent.spawn((Sprite { color: body_color, custom_size: Some(Vec2::new(40.0, 24.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.1)));
        parent.spawn((Sprite { color: body_hi, custom_size: Some(Vec2::new(16.0, 6.0)), ..default() }, Transform::from_xyz(-6.0, 8.0, 0.14)));
        // Belly
        parent.spawn((Sprite { color: belly_color, custom_size: Some(Vec2::new(30.0, 8.0)), ..default() }, Transform::from_xyz(0.0, -6.0, 0.14)));
        // Head
        parent.spawn((Sprite { color: body_color, custom_size: Some(Vec2::new(22.0, 18.0)), ..default() }, Transform::from_xyz(0.0, 14.0, 0.12)));
        // Horns
        parent.spawn((Sprite { color: horn_color, custom_size: Some(Vec2::new(4.0, 10.0)), ..default() }, Transform::from_xyz(-8.0, 24.0, 0.16)));
        parent.spawn((Sprite { color: horn_color, custom_size: Some(Vec2::new(4.0, 10.0)), ..default() }, Transform::from_xyz(8.0, 24.0, 0.16)));
        // Eyes
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 4.0)), ..default() }, Transform::from_xyz(-5.0, 16.0, 0.18)));
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 4.0)), ..default() }, Transform::from_xyz(5.0, 16.0, 0.18)));
        // Tail segments
        for i in 0..3 {
            let dx = 18.0 + i as f32 * 10.0;
            let sz = 14.0 - i as f32 * 3.0;
            parent.spawn((Sprite { color: body_color, custom_size: Some(Vec2::new(sz, sz * 0.7)), ..default() }, Transform::from_xyz(dx, -2.0 - i as f32 * 2.0, 0.08 - i as f32 * 0.01)));
        }
    });
}

/// Floor 4: Root Ancient — massive tree creature, uses GroundEnemy AI.
fn spawn_boss_root(commands: &mut Commands, floor: i32) {
    let hp = 20 + floor * 5;
    let bark_color = Color::srgb(0.30, 0.22, 0.12);
    let bark_hi = Color::srgb(0.42, 0.34, 0.20);
    let bark_dark = Color::srgb(0.18, 0.12, 0.06);
    let leaf_color = Color::srgb(0.20, 0.50, 0.15);
    let eye_color = Color::srgb(0.6, 1.0, 0.3);
    let eye_glow = Color::srgba(0.3, 0.8, 0.1, 0.5);
    let root_color = Color::srgb(0.25, 0.18, 0.10);
    let moss_color = Color::srgba(0.3, 0.5, 0.2, 0.4);

    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(52.0, 52.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0 + 100.0, 150.0, Z_ENEMIES),
        Enemy { health: hp, max_health: hp, contact_damage: 2, score_reward: 800 + floor * 150, gold_drop: 80 + floor * 30 },
        GroundEnemy { speed: 60.0 + floor as f32 * 6.0, direction: -1.0, vy: 0.0, detect_range: 400.0, leap_cooldown: 3.0, is_leaping: false },
        BossEnemy,
        RoomEntity, PlayingEntity,
    )).with_children(|parent| {
        // Moss aura
        parent.spawn((Sprite { color: moss_color, custom_size: Some(Vec2::new(58.0, 58.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.0)));
        // Trunk body
        parent.spawn((Sprite { color: bark_color, custom_size: Some(Vec2::new(32.0, 36.0)), ..default() }, Transform::from_xyz(0.0, -2.0, 0.1)));
        parent.spawn((Sprite { color: bark_hi, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() }, Transform::from_xyz(-6.0, 8.0, 0.14)));
        parent.spawn((Sprite { color: bark_dark, custom_size: Some(Vec2::new(28.0, 8.0)), ..default() }, Transform::from_xyz(0.0, -18.0, 0.12)));
        // Bark texture lines
        for dx in [-8.0_f32, 0.0, 8.0] {
            parent.spawn((Sprite { color: bark_dark, custom_size: Some(Vec2::new(2.0, 20.0)), ..default() }, Transform::from_xyz(dx, 0.0, 0.13)));
        }
        // Head / crown of leaves
        parent.spawn((Sprite { color: bark_color, custom_size: Some(Vec2::new(26.0, 20.0)), ..default() }, Transform::from_xyz(0.0, 18.0, 0.1)));
        parent.spawn((Sprite { color: leaf_color, custom_size: Some(Vec2::new(36.0, 16.0)), ..default() }, Transform::from_xyz(0.0, 28.0, 0.15)));
        parent.spawn((Sprite { color: leaf_color, custom_size: Some(Vec2::new(28.0, 10.0)), ..default() }, Transform::from_xyz(0.0, 36.0, 0.15)));
        // Eyes (glowing green)
        parent.spawn((Sprite { color: eye_glow, custom_size: Some(Vec2::new(8.0, 8.0)), ..default() }, Transform::from_xyz(-6.0, 18.0, 0.17)));
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() }, Transform::from_xyz(-6.0, 18.0, 0.19)));
        parent.spawn((Sprite { color: eye_glow, custom_size: Some(Vec2::new(8.0, 8.0)), ..default() }, Transform::from_xyz(6.0, 18.0, 0.17)));
        parent.spawn((Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() }, Transform::from_xyz(6.0, 18.0, 0.19)));
        // Root arms
        parent.spawn((Sprite { color: root_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() }, Transform::from_xyz(-22.0, 2.0, 0.08)));
        parent.spawn((Sprite { color: root_color, custom_size: Some(Vec2::new(8.0, 12.0)), ..default() }, Transform::from_xyz(-30.0, -4.0, 0.07)));
        parent.spawn((Sprite { color: root_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() }, Transform::from_xyz(22.0, 2.0, 0.08)));
        parent.spawn((Sprite { color: root_color, custom_size: Some(Vec2::new(8.0, 12.0)), ..default() }, Transform::from_xyz(30.0, -4.0, 0.07)));
        // Root base / feet
        parent.spawn((Sprite { color: root_color, custom_size: Some(Vec2::new(40.0, 6.0)), ..default() }, Transform::from_xyz(0.0, -22.0, 0.06)));
    });
}

/// Floor 1 (default): Crimson Warlord — the original boss.
fn spawn_boss_warlord(commands: &mut Commands, floor: i32) {
    let hp = 10 + floor * 3;

    let body_color      = Color::srgb(0.45, 0.05, 0.08);
    let body_hi_color   = Color::srgb(0.60, 0.12, 0.15);
    let body_dark       = Color::srgb(0.25, 0.02, 0.04);
    let robe_color      = Color::srgb(0.30, 0.03, 0.05);
    let robe_hem        = Color::srgb(0.40, 0.06, 0.08);
    let robe_lining     = Color::srgb(0.38, 0.05, 0.07);
    let robe_fold       = Color::srgb(0.18, 0.02, 0.03);
    let head_color      = Color::srgb(0.52, 0.08, 0.10);
    let head_shadow     = Color::srgb(0.28, 0.03, 0.05);
    let head_hi         = Color::srgb(0.65, 0.15, 0.18);
    let cheek_color     = Color::srgb(0.42, 0.06, 0.08);
    let nose_color      = Color::srgb(0.10, 0.01, 0.02);
    let jaw_color       = Color::srgb(0.30, 0.03, 0.05);
    let teeth_color     = Color::srgb(0.80, 0.75, 0.65);
    let fang_color      = Color::srgb(0.90, 0.88, 0.78);
    let eye_socket      = Color::srgb(0.06, 0.01, 0.01);
    let eye_glow        = Color::srgba(0.90, 0.10, 0.80, 0.50);
    let eye_color       = Color::srgb(1.0,  0.20, 0.90);
    let spike_color     = Color::srgb(0.22, 0.02, 0.04);
    let crown_color     = Color::srgb(0.18, 0.02, 0.03);
    let gem_color       = Color::srgb(0.90, 0.30, 1.00);
    let claw_color      = Color::srgb(0.38, 0.04, 0.06);
    let bone_color      = Color::srgb(0.75, 0.70, 0.60);
    let pauldron_color  = Color::srgb(0.28, 0.03, 0.05);
    let pauldron_spike  = Color::srgb(0.12, 0.01, 0.02);
    let rib_color       = Color::srgb(0.55, 0.42, 0.38);
    let aura_color      = Color::srgba(0.15, 0.0, 0.20, 0.30);

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
        // Dark aura / shadow halo (behind everything)
        parent.spawn((
            Sprite { color: aura_color, custom_size: Some(Vec2::new(52.0, 52.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        // Robe/skirt base (behind body)
        parent.spawn((
            Sprite { color: robe_color, custom_size: Some(Vec2::new(32.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, -18.0, 0.05),
        ));
        // Robe inner lining center
        parent.spawn((
            Sprite { color: robe_lining, custom_size: Some(Vec2::new(18.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, -17.0, 0.06),
        ));
        // Robe fold line left
        parent.spawn((
            Sprite { color: robe_fold, custom_size: Some(Vec2::new(2.0, 10.0)), ..default() },
            Transform::from_xyz(-9.0, -17.0, 0.07),
        ));
        // Robe fold line right
        parent.spawn((
            Sprite { color: robe_fold, custom_size: Some(Vec2::new(2.0, 10.0)), ..default() },
            Transform::from_xyz(9.0, -17.0, 0.07),
        ));
        // Robe hem trim at bottom
        parent.spawn((
            Sprite { color: robe_hem, custom_size: Some(Vec2::new(36.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -24.0, 0.06),
        ));
        // Main body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(36.0, 28.0)), ..default() },
            Transform::from_xyz(0.0, -4.0, 0.1),
        ));
        // Body bottom shadow strip
        parent.spawn((
            Sprite { color: body_dark, custom_size: Some(Vec2::new(30.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, -14.0, 0.14),
        ));
        // Body top rim highlight
        parent.spawn((
            Sprite { color: body_hi_color, custom_size: Some(Vec2::new(14.0, 5.0)), ..default() },
            Transform::from_xyz(-8.0, 6.0, 0.14),
        ));
        // Ribcage lines
        for (i, rx) in [-6i32, -2, 2, 6].iter().enumerate() {
            let z_off = 0.01 * i as f32;
            parent.spawn((
                Sprite { color: rib_color, custom_size: Some(Vec2::new(2.0, 16.0)), ..default() },
                Transform::from_xyz(*rx as f32, -4.0, 0.15 + z_off),
            ));
        }
        // Pauldron left
        parent.spawn((
            Sprite { color: pauldron_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(-22.0, 6.0, 0.16),
        ));
        // Pauldron spike left
        parent.spawn((
            Sprite { color: pauldron_spike, custom_size: Some(Vec2::new(4.0, 6.0)), ..default() },
            Transform::from_xyz(-24.0, 12.0, 0.17),
        ));
        // Pauldron right
        parent.spawn((
            Sprite { color: pauldron_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(22.0, 6.0, 0.16),
        ));
        // Pauldron spike right
        parent.spawn((
            Sprite { color: pauldron_spike, custom_size: Some(Vec2::new(4.0, 6.0)), ..default() },
            Transform::from_xyz(24.0, 12.0, 0.17),
        ));
        // Claw left arm
        parent.spawn((
            Sprite { color: claw_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(-22.0, -2.0, 0.1),
        ));
        // Claw finger 1 left (pointing down)
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(-28.0, -4.0, 0.12),
        ));
        // Claw finger 2 left (spread out)
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(-31.0, -1.0, 0.12),
        ));
        // Claw right arm
        parent.spawn((
            Sprite { color: claw_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(22.0, -2.0, 0.1),
        ));
        // Claw finger 1 right
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(28.0, -4.0, 0.12),
        ));
        // Claw finger 2 right
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(31.0, -1.0, 0.12),
        ));
        // Head
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(24.0, 22.0)), ..default() },
            Transform::from_xyz(0.0, 18.0, 0.1),
        ));
        // Head chin shadow underside
        parent.spawn((
            Sprite { color: head_shadow, custom_size: Some(Vec2::new(22.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.14),
        ));
        // Head skull catch-light
        parent.spawn((
            Sprite { color: head_hi, custom_size: Some(Vec2::new(10.0, 4.0)), ..default() },
            Transform::from_xyz(-4.0, 26.0, 0.14),
        ));
        // Cheekbone left
        parent.spawn((
            Sprite { color: cheek_color, custom_size: Some(Vec2::new(6.0, 3.0)), ..default() },
            Transform::from_xyz(-8.0, 20.0, 0.14),
        ));
        // Cheekbone right
        parent.spawn((
            Sprite { color: cheek_color, custom_size: Some(Vec2::new(6.0, 3.0)), ..default() },
            Transform::from_xyz(8.0, 20.0, 0.14),
        ));
        // Nose cavity (dark hollow)
        parent.spawn((
            Sprite { color: nose_color, custom_size: Some(Vec2::new(5.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, 16.0, 0.16),
        ));
        // Jaw edge line
        parent.spawn((
            Sprite { color: jaw_color, custom_size: Some(Vec2::new(18.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.16),
        ));
        // Teeth row
        parent.spawn((
            Sprite { color: teeth_color, custom_size: Some(Vec2::new(16.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, 10.0, 0.17),
        ));
        // Center fang left
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(-3.0, 9.0, 0.18),
        ));
        // Center fang right
        parent.spawn((
            Sprite { color: fang_color, custom_size: Some(Vec2::new(3.0, 5.0)), ..default() },
            Transform::from_xyz(3.0, 9.0, 0.18),
        ));
        // Eye socket left (deep dark hollow)
        parent.spawn((
            Sprite { color: eye_socket, custom_size: Some(Vec2::new(10.0, 10.0)), ..default() },
            Transform::from_xyz(-6.0, 22.0, 0.18),
        ));
        // Eye diffuse glow left
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(7.0, 7.0)), ..default() },
            Transform::from_xyz(-6.0, 22.0, 0.19),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(-6.0, 22.0, 0.2),
        ));
        // Eye socket right
        parent.spawn((
            Sprite { color: eye_socket, custom_size: Some(Vec2::new(10.0, 10.0)), ..default() },
            Transform::from_xyz(6.0, 22.0, 0.18),
        ));
        // Eye diffuse glow right
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(7.0, 7.0)), ..default() },
            Transform::from_xyz(6.0, 22.0, 0.19),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(6.0, 22.0, 0.2),
        ));
        // Crown band base
        parent.spawn((
            Sprite { color: crown_color, custom_size: Some(Vec2::new(26.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, 29.0, 0.2),
        ));
        // Crown spike outer left
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(10.0, 5.0)), ..default() },
            Transform::from_xyz(-14.0, 30.0, 0.2),
        ));
        // Crown spike inner left
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(8.0, 4.0)), ..default() },
            Transform::from_xyz(-8.0, 32.0, 0.2),
        ));
        // Crown spike center (tallest)
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(8.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, 33.0, 0.2),
        ));
        // Crown center gem
        parent.spawn((
            Sprite { color: gem_color, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, 38.0, 0.22),
        ));
        // Crown spike inner right
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(8.0, 4.0)), ..default() },
            Transform::from_xyz(8.0, 32.0, 0.2),
        ));
        // Crown spike outer right
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(10.0, 5.0)), ..default() },
            Transform::from_xyz(14.0, 30.0, 0.2),
        ));
    });
}

// ---------------------------------------------------------------------------
// New enemy spawn helpers (Sprint 3)
// ---------------------------------------------------------------------------

/// Skeleton Mage — hooded skull with glowing staff
pub fn spawn_mage_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let robe_color = Color::srgb(0.20, 0.10, 0.30);
    let robe_hi = Color::srgb(0.30, 0.18, 0.42);
    let hood_color = Color::srgb(0.15, 0.08, 0.25);
    let skull_color = Color::srgb(0.85, 0.82, 0.75);
    let eye_color = Color::srgb(0.4, 1.0, 0.5);
    let eye_glow = Color::srgba(0.2, 0.8, 0.3, 0.5);
    let staff_color = Color::srgb(0.35, 0.25, 0.15);
    let orb_color = Color::srgb(0.5, 0.2, 0.9);
    let orb_glow = Color::srgba(0.4, 0.1, 0.8, 0.4);
    let bone_color = Color::srgb(0.80, 0.75, 0.65);

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
        // Robe body
        parent.spawn((
            Sprite { color: robe_color, custom_size: Some(Vec2::new(18.0, 16.0)), ..default() },
            Transform::from_xyz(0.0, -2.0, 0.1),
        ));
        // Robe highlight
        parent.spawn((
            Sprite { color: robe_hi, custom_size: Some(Vec2::new(8.0, 4.0)), ..default() },
            Transform::from_xyz(-3.0, 4.0, 0.15),
        ));
        // Hood
        parent.spawn((
            Sprite { color: hood_color, custom_size: Some(Vec2::new(16.0, 12.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.1),
        ));
        // Skull face
        parent.spawn((
            Sprite { color: skull_color, custom_size: Some(Vec2::new(12.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.12),
        ));
        // Eye glow left
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(-3.0, 13.0, 0.14),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(-3.0, 13.0, 0.16),
        ));
        // Eye glow right
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(3.0, 13.0, 0.14),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(3.0, 13.0, 0.16),
        ));
        // Jaw / teeth
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(8.0, 2.0)), ..default() },
            Transform::from_xyz(0.0, 8.0, 0.14),
        ));
        // Staff
        parent.spawn((
            Sprite { color: staff_color, custom_size: Some(Vec2::new(3.0, 24.0)), ..default() },
            Transform::from_xyz(12.0, 4.0, 0.08),
        ));
        // Staff orb glow
        parent.spawn((
            Sprite { color: orb_glow, custom_size: Some(Vec2::new(12.0, 12.0)), ..default() },
            Transform::from_xyz(12.0, 18.0, 0.09),
            MageStaffGlow,
        ));
        // Staff orb
        parent.spawn((
            Sprite { color: orb_color, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
            Transform::from_xyz(12.0, 18.0, 0.1),
        ));
        // Robe bottom / skirt
        parent.spawn((
            Sprite { color: robe_color, custom_size: Some(Vec2::new(22.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, -12.0, 0.08),
        ));
        // Bony hand left
        parent.spawn((
            Sprite { color: bone_color, custom_size: Some(Vec2::new(4.0, 3.0)), ..default() },
            Transform::from_xyz(-8.0, 0.0, 0.16),
        ));
    });
}

/// Slime — bouncy gelatinous blob
pub fn spawn_slime_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32, size: SlimeSize) {
    let (sz, hp, dmg, score, gold) = match size {
        SlimeSize::Large => (20.0, 4 + floor.min(3), 1, 100, 12 + floor * 3),
        SlimeSize::Small => (12.0, 1 + floor.min(2), 1, 40, 5 + floor),
    };

    let body_color = Color::srgb(0.30, 0.75, 0.25);
    let body_hi = Color::srgba(0.5, 0.95, 0.4, 0.7);
    let body_dark = Color::srgb(0.18, 0.55, 0.15);
    let eye_white = Color::srgb(0.95, 0.95, 0.90);
    let pupil_color = Color::srgb(0.05, 0.08, 0.05);
    let mouth_color = Color::srgb(0.12, 0.40, 0.10);
    let bubble_color = Color::srgba(0.7, 1.0, 0.7, 0.3);

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
        let s = if size == SlimeSize::Large { 1.0 } else { 0.6 };
        // Body blob
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(20.0 * s, 16.0 * s)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Body top dome (rounder shape)
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(16.0 * s, 10.0 * s)), ..default() },
            Transform::from_xyz(0.0, 6.0 * s, 0.12),
        ));
        // Highlight shimmer
        parent.spawn((
            Sprite { color: body_hi, custom_size: Some(Vec2::new(6.0 * s, 4.0 * s)), ..default() },
            Transform::from_xyz(-3.0 * s, 6.0 * s, 0.16),
        ));
        // Bottom shadow
        parent.spawn((
            Sprite { color: body_dark, custom_size: Some(Vec2::new(18.0 * s, 4.0 * s)), ..default() },
            Transform::from_xyz(0.0, -6.0 * s, 0.14),
        ));
        // Eye left white
        parent.spawn((
            Sprite { color: eye_white, custom_size: Some(Vec2::new(5.0 * s, 5.0 * s)), ..default() },
            Transform::from_xyz(-4.0 * s, 4.0 * s, 0.18),
        ));
        // Eye left pupil
        parent.spawn((
            Sprite { color: pupil_color, custom_size: Some(Vec2::new(2.0 * s, 3.0 * s)), ..default() },
            Transform::from_xyz(-3.0 * s, 3.5 * s, 0.2),
        ));
        // Eye right white
        parent.spawn((
            Sprite { color: eye_white, custom_size: Some(Vec2::new(5.0 * s, 5.0 * s)), ..default() },
            Transform::from_xyz(4.0 * s, 4.0 * s, 0.18),
        ));
        // Eye right pupil
        parent.spawn((
            Sprite { color: pupil_color, custom_size: Some(Vec2::new(2.0 * s, 3.0 * s)), ..default() },
            Transform::from_xyz(5.0 * s, 3.5 * s, 0.2),
        ));
        // Mouth (happy/derpy line)
        parent.spawn((
            Sprite { color: mouth_color, custom_size: Some(Vec2::new(6.0 * s, 2.0 * s)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.2),
        ));
        // Internal bubble detail
        parent.spawn((
            Sprite { color: bubble_color, custom_size: Some(Vec2::new(3.0 * s, 3.0 * s)), ..default() },
            Transform::from_xyz(5.0 * s, -2.0 * s, 0.15),
        ));
        parent.spawn((
            Sprite { color: bubble_color, custom_size: Some(Vec2::new(2.0 * s, 2.0 * s)), ..default() },
            Transform::from_xyz(-6.0 * s, 1.0 * s, 0.15),
        ));
    });
}

/// Ghost — ethereal floating specter
pub fn spawn_ghost_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let body_color = Color::srgba(0.75, 0.80, 0.90, 0.6);
    let body_inner = Color::srgba(0.85, 0.88, 0.95, 0.4);
    let eye_color = Color::srgb(0.2, 0.4, 1.0);
    let eye_glow = Color::srgba(0.1, 0.3, 0.9, 0.5);
    let wisp_color = Color::srgba(0.6, 0.7, 0.9, 0.3);
    let mouth_color = Color::srgba(0.1, 0.1, 0.2, 0.7);
    let chain_color = Color::srgba(0.5, 0.5, 0.55, 0.4);

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
        // Main spectral body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(20.0, 22.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Inner glow
        parent.spawn((
            Sprite { color: body_inner, custom_size: Some(Vec2::new(14.0, 16.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.12),
        ));
        // Tattered bottom (wispy tendrils)
        for dx in [-6.0_f32, -2.0, 2.0, 6.0] {
            parent.spawn((
                Sprite { color: body_color, custom_size: Some(Vec2::new(4.0, 8.0)), ..default() },
                Transform::from_xyz(dx, -13.0, 0.08),
                GhostWisp,
            ));
        }
        // Eye glow left
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
            Transform::from_xyz(-4.0, 4.0, 0.14),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
            Transform::from_xyz(-4.0, 4.0, 0.16),
        ));
        // Eye glow right
        parent.spawn((
            Sprite { color: eye_glow, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
            Transform::from_xyz(4.0, 4.0, 0.14),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
            Transform::from_xyz(4.0, 4.0, 0.16),
        ));
        // Mouth (dark hollow)
        parent.spawn((
            Sprite { color: mouth_color, custom_size: Some(Vec2::new(6.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -1.0, 0.16),
        ));
        // Wisp trail behind
        parent.spawn((
            Sprite { color: wisp_color, custom_size: Some(Vec2::new(10.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, -8.0, 0.05),
            GhostWisp,
        ));
        // Broken chains (spectral chains hanging)
        parent.spawn((
            Sprite { color: chain_color, custom_size: Some(Vec2::new(2.0, 10.0)), ..default() },
            Transform::from_xyz(-8.0, -4.0, 0.06),
        ));
        parent.spawn((
            Sprite { color: chain_color, custom_size: Some(Vec2::new(2.0, 8.0)), ..default() },
            Transform::from_xyz(9.0, -2.0, 0.06),
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
            // Swoop down toward target, then pull back up
            let target_y = fe.dive_target_y;
            tf.translation.y += (target_y - tf.translation.y).signum() * BAT_DIVE_SPEED * dt;
            if (tf.translation.y - target_y).abs() < 10.0 || tf.translation.y < TILE_SIZE + 20.0 {
                fe.is_diving = false;
                fe.dive_cooldown = BAT_DIVE_COOLDOWN;
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
// Animation systems
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
                        // Bob ±3 px at ~8 Hz walk cycle
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
                    // Left wing flaps upward (positive angle = CCW = up for left side)
                    let angle = (t * 6.0).sin() * 0.5;
                    tf.rotation = Quat::from_rotation_z(angle);
                } else if is_right.is_some() {
                    // Right wing mirrors the left
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
                    // Tilt down toward the charge direction with a small vibration
                    let vib = (t * 20.0).sin() * 0.04;
                    let tilt = -0.35 + vib; // ~20 degrees downward
                    tf.rotation = Quat::from_rotation_z(tilt * charger.charge_dir * horn.side);
                } else {
                    // Return to resting angle
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
) {
    if slime_size == SlimeSize::Large {
        for offset in [-20.0_f32, 20.0] {
            spawn_slime_enemy(
                commands,
                (position.x + offset).clamp(TILE_SIZE + 10.0, ROOM_W - TILE_SIZE - 10.0),
                position.y + 10.0,
                floor,
                SlimeSize::Small,
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
