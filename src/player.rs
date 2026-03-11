use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, MetaProgression};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerLanded>()
            .add_event::<PlayerAttack>()
            .add_event::<PlayerDamaged>()
            .add_event::<PlayerDied>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    player_physics,
                    player_invuln,
                    melee_hitbox_lifetime,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// Events
#[derive(Event)]
pub struct PlayerLanded;

#[derive(Event)]
pub struct PlayerAttack;

#[derive(Event)]
#[allow(dead_code)]
pub struct PlayerDamaged {
    pub amount: i32,
    pub remaining: i32,
}

#[derive(Event)]
pub struct PlayerDied;

// Components
#[derive(Component)]
pub struct Player {
    pub vx: f32,
    pub vy: f32,
    pub facing: f32, // 1.0 = right, -1.0 = left
    pub max_health: i32,
    pub health: i32,
    pub mana: f32,
    pub max_mana: f32,
    pub invulnerable: f32,
    pub is_on_floor: bool,
    pub was_on_floor: bool,
    pub is_jumping: bool,
    pub jump_hold_time: f32,
    pub coyote_timer: f32,
    pub jump_buffer: f32,
    pub melee_cooldown: f32,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            facing: 1.0,
            max_health: 5,
            health: 5,
            mana: MANA_MAX,
            max_mana: MANA_MAX,
            invulnerable: 0.0,
            is_on_floor: false,
            was_on_floor: false,
            is_jumping: false,
            jump_hold_time: 0.0,
            coyote_timer: 0.0,
            jump_buffer: 0.0,
            melee_cooldown: 0.0,
            is_dashing: false,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
        }
    }
}

/// Short-lived melee hitbox entity
#[derive(Component)]
pub struct MeleeHitbox {
    pub damage: i32,
    pub lifetime: f32,
}

fn spawn_player(mut commands: Commands, meta: Res<MetaProgression>) {
    let max_hp = 5 + meta.max_health_bonus;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.7, 0.3),
            custom_size: Some(Vec2::new(20.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(80.0, 100.0, Z_PLAYER),
        Player {
            max_health: max_hp,
            health: max_hp,
            ..default()
        },
        PlayingEntity,
    ));
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Player, &Transform)>,
    mut commands: Commands,
    mut ev_attack: EventWriter<PlayerAttack>,
    time: Res<Time>,
) {
    let Ok((mut player, tf)) = query.get_single_mut() else { return };
    let dt = time.delta_secs();

    // Decrement cooldowns
    player.melee_cooldown = (player.melee_cooldown - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    player.jump_buffer = (player.jump_buffer - dt).max(0.0);

    // Dash logic
    if player.is_dashing {
        player.dash_timer -= dt;
        if player.dash_timer <= 0.0 {
            player.is_dashing = false;
        }
        return; // Skip normal input during dash
    }

    // Horizontal input
    let mut input_dir = 0.0_f32;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        input_dir -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        input_dir += 1.0;
    }

    // Update facing
    if input_dir != 0.0 {
        player.facing = input_dir;
    }

    let target_speed = input_dir * if player.is_on_floor { MOVE_SPEED } else { AIR_SPEED };
    let accel = if player.is_on_floor { ACCEL_GROUND } else { ACCEL_AIR };

    if input_dir == 0.0 && player.is_on_floor {
        player.vx = move_toward(player.vx, 0.0, FRICTION * dt);
    } else {
        player.vx = move_toward(player.vx, target_speed, accel * dt);
    }

    // Jump
    let jump_pressed = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    let jump_held = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    let jump_released = keys.just_released(KeyCode::Space) || keys.just_released(KeyCode::KeyW) || keys.just_released(KeyCode::ArrowUp);

    if jump_pressed {
        player.jump_buffer = JUMP_BUFFER_TIME;
    }

    // Can jump if on floor or within coyote time
    let can_jump = player.is_on_floor || player.coyote_timer > 0.0;
    if player.jump_buffer > 0.0 && can_jump {
        player.vy = JUMP_SPEED;
        player.is_jumping = true;
        player.jump_hold_time = MAX_JUMP_HOLD;
        player.jump_buffer = 0.0;
        player.coyote_timer = 0.0;
    }

    // Variable jump height
    if player.is_jumping && jump_held && player.jump_hold_time > 0.0 {
        player.vy += JUMP_HOLD_BOOST * dt;
        player.jump_hold_time = (player.jump_hold_time - dt).max(0.0);
    } else if jump_released {
        player.is_jumping = false;
    }

    // Dash
    if keys.just_pressed(KeyCode::ShiftLeft) && player.dash_cooldown <= 0.0 {
        player.is_dashing = true;
        player.dash_timer = DASH_DURATION;
        player.dash_cooldown = DASH_COOLDOWN;
        player.vx = player.facing * DASH_SPEED;
        player.vy = 0.0;
        player.invulnerable = player.invulnerable.max(DASH_DURATION);
    }

    // Melee attack (J key or left click)
    let attack = keys.just_pressed(KeyCode::KeyJ) || mouse.just_pressed(MouseButton::Left);
    if attack && player.melee_cooldown <= 0.0 {
        player.melee_cooldown = MELEE_COOLDOWN;
        ev_attack.send(PlayerAttack);

        // Spawn hitbox in front of player
        let hitbox_x = tf.translation.x + player.facing * MELEE_RANGE;
        let hitbox_y = tf.translation.y;
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 0.8, 0.6),
                custom_size: Some(Vec2::new(MELEE_RANGE, MELEE_WIDTH)),
                ..default()
            },
            Transform::from_xyz(hitbox_x, hitbox_y, Z_EFFECTS),
            MeleeHitbox {
                damage: MELEE_DAMAGE,
                lifetime: MELEE_ACTIVE_TIME,
            },
            PlayingEntity,
        ));
    }

    // Mana regen
    player.mana = (player.mana + MANA_REGEN_RATE * dt).min(player.max_mana);
}

fn player_physics(
    mut query: Query<(&mut Player, &mut Transform)>,
    tile_q: Query<(&GlobalTransform, &Sprite), (With<crate::room::Tile>, Without<Player>)>,
    mut ev_landed: EventWriter<PlayerLanded>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut ev_died: EventWriter<PlayerDied>,
) {
    let Ok((mut player, mut tf)) = query.get_single_mut() else { return };
    let dt = time.delta_secs();

    // Skip physics during dash (just apply dash velocity)
    if player.is_dashing {
        tf.translation.x += player.vx * dt;
        return;
    }

    // Gravity
    if player.vy > -TERMINAL_VELOCITY {
        player.vy = (player.vy - GRAVITY * dt).max(-TERMINAL_VELOCITY);
    }

    // Apply velocity
    let new_x = tf.translation.x + player.vx * dt;
    let new_y = tf.translation.y + player.vy * dt;

    // Room bounds clamping
    let player_half_w = 10.0;
    let player_half_h = 16.0;
    let room_left = 0.0;
    let room_right = ROOM_W;
    let clamped_x = new_x.clamp(room_left + player_half_w, room_right - player_half_w);
    if (clamped_x - new_x).abs() > 0.1 {
        player.vx = 0.0;
    }
    tf.translation.x = clamped_x;

    // Tile collision (AABB)
    player.was_on_floor = player.is_on_floor;
    player.is_on_floor = false;
    let mut resolved_y = new_y;

    for (tile_gtf, tile_sprite) in &tile_q {
        let tile_pos = tile_gtf.translation();
        let tile_size = tile_sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let tile_half_w = tile_size.x / 2.0;
        let tile_half_h = tile_size.y / 2.0;

        let dx = (tf.translation.x - tile_pos.x).abs();
        let dy = resolved_y - tile_pos.y;

        if dx < player_half_w + tile_half_w - 2.0 {
            // Landing on top
            if dy > 0.0 && dy < player_half_h + tile_half_h && player.vy <= 0.0 {
                let tile_top = tile_pos.y + tile_half_h;
                if resolved_y - player_half_h < tile_top && tf.translation.y - player_half_h >= tile_top - 4.0 {
                    resolved_y = tile_top + player_half_h;
                    player.vy = 0.0;
                    player.is_on_floor = true;
                }
            }
            // Hitting head
            else if dy < 0.0 && -dy < player_half_h + tile_half_h && player.vy > 0.0 {
                let tile_bottom = tile_pos.y - tile_half_h;
                if resolved_y + player_half_h > tile_bottom && tf.translation.y + player_half_h <= tile_bottom + 4.0 {
                    resolved_y = tile_bottom - player_half_h;
                    player.vy = 0.0;
                }
            }
        }

        // Horizontal wall collision
        let hy = tf.translation.y - tile_pos.y;
        if hy.abs() < player_half_h + tile_half_h - 2.0 {
            let hx = tf.translation.x - tile_pos.x;
            if hx > 0.0 && hx < player_half_w + tile_half_w && player.vx < 0.0 {
                let tile_right = tile_pos.x + tile_half_w;
                if tf.translation.x - player_half_w < tile_right {
                    tf.translation.x = tile_right + player_half_w;
                    player.vx = 0.0;
                }
            } else if hx < 0.0 && -hx < player_half_w + tile_half_w && player.vx > 0.0 {
                let tile_left = tile_pos.x - tile_half_w;
                if tf.translation.x + player_half_w > tile_left {
                    tf.translation.x = tile_left - player_half_w;
                    player.vx = 0.0;
                }
            }
        }
    }

    tf.translation.y = resolved_y;

    // Coyote time
    if player.was_on_floor && !player.is_on_floor && player.vy <= 0.0 {
        player.coyote_timer = COYOTE_TIME;
    }
    if !player.is_on_floor {
        player.coyote_timer = (player.coyote_timer - dt).max(0.0);
    }

    // Land event
    if player.is_on_floor && !player.was_on_floor {
        player.is_jumping = false;
        ev_landed.send(PlayerLanded);
    }

    // Death: fell below room
    if tf.translation.y < -200.0 {
        ev_died.send(PlayerDied);
        next_state.set(GameState::GameOver);
    }
}

fn player_invuln(mut query: Query<(&mut Player, &mut Sprite)>, time: Res<Time>) {
    let Ok((mut player, mut sprite)) = query.get_single_mut() else { return };
    if player.invulnerable > 0.0 {
        player.invulnerable = (player.invulnerable - time.delta_secs()).max(0.0);
        let flash = (time.elapsed_secs() * 15.0).sin() > 0.0;
        sprite.color = if flash {
            Color::srgba(0.2, 0.7, 0.3, 0.4)
        } else {
            Color::srgb(0.2, 0.7, 0.3)
        };
    }
}

fn melee_hitbox_lifetime(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MeleeHitbox)>,
    time: Res<Time>,
) {
    for (entity, mut hitbox) in &mut query {
        hitbox.lifetime -= time.delta_secs();
        if hitbox.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn move_toward(current: f32, target: f32, step: f32) -> f32 {
    if current < target {
        (current + step).min(target)
    } else {
        (current - step).max(target)
    }
}
