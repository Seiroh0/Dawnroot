use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, room::{RoomState, RoomType, RoomEntity}};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyDefeated>()
            .add_systems(OnEnter(GameState::Playing), reset_enemy_state)
            .add_systems(
                Update,
                (
                    spawn_room_enemies,
                    ground_enemy_ai,
                    flying_enemy_ai,
                    turret_enemy_ai,
                    charger_enemy_ai,
                    enemy_projectile_movement,
                    count_alive_enemies,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

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

#[derive(Resource)]
struct EnemySpawnState {
    spawned_for_room: bool,
}

fn reset_enemy_state(mut commands: Commands) {
    commands.insert_resource(EnemySpawnState { spawned_for_room: false });
}

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

fn spawn_ground_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let speed = 80.0 + floor as f32 * 10.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.85, 0.3, 0.35),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 2 + floor / 2,
            max_health: 2 + floor / 2,
            contact_damage: 1,
            score_reward: 80,
            gold_drop: 5 + floor * 2,
        },
        GroundEnemy {
            speed,
            direction: 1.0,
            vy: 0.0,
            detect_range: 200.0,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

fn spawn_flying_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.7, 0.35, 0.6),
            custom_size: Some(Vec2::new(22.0, 18.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 100.0, Z_ENEMIES),
        Enemy {
            health: 2 + floor / 3,
            max_health: 2 + floor / 3,
            contact_damage: 1,
            score_reward: 110,
            gold_drop: 8 + floor * 2,
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
    ));
}

fn spawn_turret_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    let interval = (2.0 - floor as f32 * 0.1).clamp(0.8, 2.5);
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.45),
            custom_size: Some(Vec2::new(24.0, 24.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 3 + floor / 2,
            max_health: 3 + floor / 2,
            contact_damage: 1,
            score_reward: 130,
            gold_drop: 10 + floor * 3,
        },
        TurretEnemy {
            fire_interval: interval,
            fire_timer: interval * 0.5,
            projectile_speed: 300.0 + floor as f32 * 20.0,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

fn spawn_charger_enemy(commands: &mut Commands, x: f32, y: f32, floor: i32) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.5, 0.2),
            custom_size: Some(Vec2::new(26.0, 22.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_ENEMIES),
        Enemy {
            health: 3 + floor / 2,
            max_health: 3 + floor / 2,
            contact_damage: 2,
            score_reward: 150,
            gold_drop: 12 + floor * 3,
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
    ));
}

fn spawn_boss(commands: &mut Commands, floor: i32) {
    let hp = 15 + floor * 5;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.15, 0.15),
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
    ));
}

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
