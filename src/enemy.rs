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
}

#[derive(Component)]
pub struct FlyingEnemy {
    pub amplitude: f32,
    pub wave_speed: f32,
    pub base_y: f32,
    pub phase: f32,
    pub speed_x: f32,
}

#[derive(Component)]
pub struct TurretEnemy {
    pub fire_interval: f32,
    pub fire_timer: f32,
    pub projectile_speed: f32,
}

#[derive(Component)]
pub struct ChargerEnemy {
    pub speed: f32,
    pub detect_range: f32,
    pub charging: bool,
    pub charge_dir: f32,
    pub cooldown: f32,
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
///   Body      20x14  offset (0,  2)  red-brown
///   Head      16x14  offset (0, 14)  slightly lighter red-brown
///   Eyes   2x 3x3    offset (±4, 17) glowing yellow
///   Club       6x14  offset (14, 8)  dark brown (to the right)
///   LegLeft    6x8   offset (-4, -8) red-brown  (animated)
///   LegRight   6x8   offset ( 4, -8) red-brown  (animated)
fn spawn_ground_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let speed = 80.0 + floor as f32 * 10.0;

    let body_color   = Color::srgb(0.65, 0.28, 0.20);
    let head_color   = Color::srgb(0.72, 0.34, 0.25);
    let eye_color    = Color::srgb(1.0,  0.95, 0.1);
    let club_color   = Color::srgb(0.35, 0.20, 0.10);
    let leg_color    = Color::srgb(0.60, 0.25, 0.18);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(20.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.1),
        ));
        // Head
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(16.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 14.0, 0.1),
        ));
        // Eye left
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(-4.0, 17.0, 0.2),
        ));
        // Eye right
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(4.0, 17.0, 0.2),
        ));
        // Club (held to side)
        parent.spawn((
            Sprite { color: club_color, custom_size: Some(Vec2::new(6.0, 14.0)), ..default() },
            Transform::from_xyz(14.0, 8.0, 0.1),
        ));
        // Leg left (animated)
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(-4.0, -8.0, 0.1),
            GoblinLegLeft,
        ));
        // Leg right (animated)
        parent.spawn((
            Sprite { color: leg_color, custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
            Transform::from_xyz(4.0, -8.0, 0.1),
            GoblinLegRight,
        ));
    });
}

/// FlyingEnemy – Bat
/// Layout:
///   Body       18x12  center           purple
///   EyeLeft     3x3   offset (-4, 3)   red
///   EyeRight    3x3   offset ( 4, 3)   red
///   WingLeft   16x8   offset (-16, 2)  dark purple  (animated rotate)
///   WingRight  16x8   offset ( 16, 2)  dark purple  (animated rotate)
fn spawn_flying_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let body_color = Color::srgb(0.55, 0.20, 0.70);
    let eye_color  = Color::srgb(0.95, 0.15, 0.15);
    let wing_color = Color::srgb(0.38, 0.10, 0.55);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(18.0, 12.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Eyes
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(-4.0, 3.0, 0.2),
        ));
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(4.0, 3.0, 0.2),
        ));
        // Wing left (animated)
        parent.spawn((
            Sprite { color: wing_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() },
            Transform::from_xyz(-16.0, 2.0, 0.0),
            BatWingLeft,
        ));
        // Wing right (animated)
        parent.spawn((
            Sprite { color: wing_color, custom_size: Some(Vec2::new(16.0, 8.0)), ..default() },
            Transform::from_xyz(16.0, 2.0, 0.0),
            BatWingRight,
        ));
    });
}

/// TurretEnemy – Stone Tower
/// Layout:
///   Base        24x20  offset (0, -2)   gray stone
///   Left crenel  6x6   offset (-9, 12)  darker gray
///   Mid crenel   6x6   offset ( 0, 12)  darker gray
///   Right crenel 6x6   offset ( 9, 12)  darker gray
///   Eye barrel  10x6   offset (0, 2)    glowing orange  (rotates)
fn spawn_turret_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let interval = (2.0 - floor as f32 * 0.1).clamp(0.8, 2.5);

    let stone_color    = Color::srgb(0.50, 0.52, 0.55);
    let crenel_color   = Color::srgb(0.35, 0.37, 0.40);
    let eye_color      = Color::srgb(1.0,  0.55, 0.05);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Stone base
        parent.spawn((
            Sprite { color: stone_color, custom_size: Some(Vec2::new(24.0, 20.0)), ..default() },
            Transform::from_xyz(0.0, -2.0, 0.1),
        ));
        // Crenellations (3 merlons on top)
        for cx in [-9i32, 0, 9] {
            parent.spawn((
                Sprite { color: crenel_color, custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
                Transform::from_xyz(cx as f32, 12.0, 0.1),
            ));
        }
        // Glowing eye / barrel (rotates toward player)
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(10.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, 2.0, 0.2),
            TurretEye,
        ));
    });
}

/// ChargerEnemy – Bull / Boar
/// Layout:
///   Body        26x16  offset (0, 0)     orange-brown
///   Head        18x14  offset (14, 2)    slightly lighter
///   Hoof left    6x5   offset (-10, -10) dark brown
///   Hoof right   6x5   offset (-2,  -10) dark brown
///   Horn left   12x5   offset (18, 9)    cream/tan  (tilts when charging)
///   Horn right  12x5   offset (18, 4)    cream/tan  (tilts when charging)
///   Eye          3x3   offset (19, 6)    angry red
fn spawn_charger_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let body_color = Color::srgb(0.72, 0.40, 0.14);
    let head_color = Color::srgb(0.80, 0.48, 0.18);
    let hoof_color = Color::srgb(0.28, 0.18, 0.08);
    let horn_color = Color::srgb(0.90, 0.85, 0.65);
    let eye_color  = Color::srgb(0.95, 0.15, 0.15);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(26.0, 16.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        // Head (forward-facing)
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(18.0, 14.0)), ..default() },
            Transform::from_xyz(14.0, 2.0, 0.1),
        ));
        // Hooves
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-10.0, -10.0, 0.1),
        ));
        parent.spawn((
            Sprite { color: hoof_color, custom_size: Some(Vec2::new(6.0, 5.0)), ..default() },
            Transform::from_xyz(-2.0, -10.0, 0.1),
        ));
        // Horn upper (animated)
        parent.spawn((
            Sprite { color: horn_color, custom_size: Some(Vec2::new(12.0, 5.0)), ..default() },
            Transform::from_xyz(18.0, 9.0, 0.2),
            BoarHorn { side: 1.0 },
        ));
        // Horn lower (animated)
        parent.spawn((
            Sprite { color: horn_color, custom_size: Some(Vec2::new(12.0, 5.0)), ..default() },
            Transform::from_xyz(18.0, 4.0, 0.2),
            BoarHorn { side: -1.0 },
        ));
        // Angry eye
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(19.0, 6.0, 0.3),
        ));
    });
}

/// Boss – Lich-like dark creature (48x48 root hitbox)
/// Layout:
///   Main body        36x28  offset (0, -4)    dark crimson
///   Robe/skirt       32x14  offset (0, -18)   very dark red
///   Head             24x22  offset (0,  18)   deep red
///   Eye left          5x5   offset (-6, 22)   glowing magenta
///   Eye right         5x5   offset ( 6, 22)   glowing magenta
///   Spike left       10x5   offset (-14, 30)  near-black red  (crown)
///   Spike mid         8x6   offset (  0, 33)  near-black red  (crown center, taller)
///   Spike right      10x5   offset ( 14, 30)  near-black red  (crown)
///   Claw left        12x8   offset (-22, -2)  dark maroon
///   Claw right       12x8   offset ( 22, -2)  dark maroon
fn spawn_boss(commands: &mut Commands, floor: i32) {
    let hp = 10 + floor * 3;

    let body_color  = Color::srgb(0.45, 0.05, 0.08);
    let robe_color  = Color::srgb(0.30, 0.03, 0.05);
    let head_color  = Color::srgb(0.52, 0.08, 0.10);
    let eye_color   = Color::srgb(1.0,  0.20, 0.90);
    let spike_color = Color::srgb(0.22, 0.02, 0.04);
    let claw_color  = Color::srgb(0.38, 0.04, 0.06);

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
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Robe/skirt (behind body)
        parent.spawn((
            Sprite { color: robe_color, custom_size: Some(Vec2::new(32.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, -18.0, 0.0),
        ));
        // Main body
        parent.spawn((
            Sprite { color: body_color, custom_size: Some(Vec2::new(36.0, 28.0)), ..default() },
            Transform::from_xyz(0.0, -4.0, 0.1),
        ));
        // Claws
        parent.spawn((
            Sprite { color: claw_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(-22.0, -2.0, 0.1),
        ));
        parent.spawn((
            Sprite { color: claw_color, custom_size: Some(Vec2::new(12.0, 8.0)), ..default() },
            Transform::from_xyz(22.0, -2.0, 0.1),
        ));
        // Head
        parent.spawn((
            Sprite { color: head_color, custom_size: Some(Vec2::new(24.0, 22.0)), ..default() },
            Transform::from_xyz(0.0, 18.0, 0.1),
        ));
        // Eyes
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(-6.0, 22.0, 0.2),
        ));
        parent.spawn((
            Sprite { color: eye_color, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(6.0, 22.0, 0.2),
        ));
        // Crown spikes
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(10.0, 5.0)), ..default() },
            Transform::from_xyz(-14.0, 30.0, 0.2),
        ));
        parent.spawn((
            Sprite { color: spike_color, custom_size: Some(Vec2::new(8.0, 8.0)), ..default() },
            Transform::from_xyz(0.0, 33.0, 0.2),
        ));
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

        ge.vy -= GRAVITY * 0.5 * dt;
        ge.vy = ge.vy.max(-600.0);

        if let Some(pp) = player_pos {
            let dist = (pp.x - tf.translation.x).abs();
            if dist < ge.detect_range {
                ge.direction = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            }
        }

        tf.translation.x += ge.direction * ge.speed * dt;
        tf.translation.y += ge.vy * dt;

        let margin = TILE_SIZE + 12.0;
        if tf.translation.x < margin || tf.translation.x > ROOM_W - margin {
            ge.direction *= -1.0;
            tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
        }

        if tf.translation.y < TILE_SIZE + 10.0 {
            tf.translation.y = TILE_SIZE + 10.0;
            ge.vy = 0.0;
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
        fe.phase += fe.wave_speed * dt;
        tf.translation.y = fe.base_y + fe.phase.sin() * fe.amplitude;

        if let Some(pp) = player_pos {
            let dir = if pp.x > tf.translation.x { 1.0 } else { -1.0 };
            tf.translation.x += dir * fe.speed_x * dt;
        }

        let margin = TILE_SIZE + 14.0;
        tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
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
        turret.fire_timer -= time.delta_secs();
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
        }
    }
}

fn charger_enemy_ai(
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
                charger.cooldown = 1.0;
                tf.translation.x = tf.translation.x.clamp(margin, ROOM_W - margin);
            }
        } else if charger.cooldown <= 0.0 {
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
