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
                    enemy_projectile_movement,
                    count_alive_enemies,
                    animate_ground_enemies,
                    animate_flying_enemies,
                    animate_charger_enemies,
                    animate_turret_eye,
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

    for i in 0..enemy_count {
        let x = 200.0 + (i as f32) * ((ROOM_W - 300.0) / enemy_count as f32);
        let y = 120.0 + ((seed.wrapping_add(i as u64) % 3) as f32) * 80.0;
        let enemy_type = ((seed.wrapping_add(i as u64)) % 4) as i32;

        match enemy_type {
            0 => spawn_ground_enemy(&mut commands, x, y, floor),
            1 => spawn_flying_enemy(&mut commands, x, y, floor),
            2 => spawn_turret_enemy(&mut commands, x, y, floor),
            _ => spawn_charger_enemy(&mut commands, x, y, floor),
        }
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
// AI systems (unchanged logic)
// ---------------------------------------------------------------------------

fn ground_enemy_ai(
    mut query: Query<(&mut Transform, &mut GroundEnemy)>,
    player_q: Query<&Transform, (With<Player>, Without<GroundEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (mut tf, mut ge) in &mut query {
        let dt = time.delta_secs();
        ge.leap_cooldown = (ge.leap_cooldown - dt).max(0.0);

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
            if dx < 120.0 && dy < 80.0 && on_ground && ge.leap_cooldown <= 0.0 && !ge.is_leaping {
                ge.is_leaping = true;
                ge.leap_cooldown = 3.0;
                ge.vy = 350.0; // jump up
                ge.direction = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            }
        }

        let speed_mult = if ge.is_leaping { 1.8 } else { 1.0 };
        tf.translation.x += ge.direction * ge.speed * speed_mult * dt;
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
    mut query: Query<(&mut Transform, &mut FlyingEnemy)>,
    player_q: Query<&Transform, (With<Player>, Without<FlyingEnemy>)>,
    time: Res<Time>,
) {
    let player_pos = player_q.get_single().map(|t| t.translation).ok();

    for (mut tf, mut fe) in &mut query {
        let dt = time.delta_secs();
        fe.dive_cooldown = (fe.dive_cooldown - dt).max(0.0);

        if fe.is_diving {
            // Swoop down toward target, then pull back up
            let target_y = fe.dive_target_y;
            tf.translation.y += (target_y - tf.translation.y).signum() * 300.0 * dt;
            if (tf.translation.y - target_y).abs() < 10.0 || tf.translation.y < TILE_SIZE + 20.0 {
                fe.is_diving = false;
                fe.dive_cooldown = 4.0;
            }
        } else {
            fe.phase += fe.wave_speed * dt;
            tf.translation.y = fe.base_y + fe.phase.sin() * fe.amplitude;

            // Initiate dive when player is below and in range
            if let Some(pp) = player_pos {
                let dx = (pp.x - tf.translation.x).abs();
                if dx < 100.0 && pp.y < tf.translation.y - 40.0 && fe.dive_cooldown <= 0.0 {
                    fe.is_diving = true;
                    fe.dive_target_y = pp.y;
                }
            }
        }

        if let Some(pp) = player_pos {
            let dir = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            let speed = if fe.is_diving { fe.speed_x * 2.0 } else { fe.speed_x };
            tf.translation.x += dir * speed * dt;
        }

        let margin = TILE_SIZE + 14.0;
        tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
        tf.translation.y = tf.translation.y.max(TILE_SIZE + 10.0);
    }
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
                turret.burst_timer = 0.12; // rapid fire interval

                let (vx, vy) = if let Some(pp) = player_pos {
                    let diff = pp - tf.translation;
                    let len = diff.length().max(1.0);
                    (diff.x / len * turret.projectile_speed, diff.y / len * turret.projectile_speed)
                } else {
                    (-turret.projectile_speed, 0.0)
                };

                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 0.3, 0.1),
                        custom_size: Some(Vec2::new(6.0, 6.0)),
                        ..default()
                    },
                    Transform::from_xyz(tf.translation.x, tf.translation.y, Z_PROJECTILES),
                    EnemyProjectile { vx, vy, lifetime: 2.5 },
                    RoomEntity,
                    PlayingEntity,
                ));
            }
            continue;
        }

        turret.fire_timer -= dt;
        if turret.fire_timer <= 0.0 {
            turret.fire_timer = turret.fire_interval;

            let (vx, vy) = if let Some(pp) = player_pos {
                let diff = pp - tf.translation;
                let len = diff.length().max(1.0);
                (diff.x / len * turret.projectile_speed, diff.y / len * turret.projectile_speed)
            } else {
                (-turret.projectile_speed, 0.0)
            };

            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.5, 0.2),
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(tf.translation.x, tf.translation.y, Z_PROJECTILES),
                EnemyProjectile { vx, vy, lifetime: 3.0 },
                RoomEntity,
                PlayingEntity,
            ));

            // Every 3rd shot triggers a burst (2 extra rapid shots)
            // Use fire_interval as a rough cycle: burst on shorter intervals
            if turret.fire_interval < 2.0 {
                turret.burst_count = 2;
                turret.burst_timer = 0.15;
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
                                vx: dir * 200.0,
                                vy: 0.0,
                                lifetime: 1.2,
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
