use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, MetaProgression, LoadedSave, equipment::PlayerStats, shop::ShopUiState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerLanded>()
            .add_event::<PlayerAttack>()
            .add_event::<PlayerDamaged>()
            .add_event::<PlayerDied>()
            .add_event::<PlayerDashed>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input,
                    player_physics,
                    player_invuln,
                    melee_hitbox_lifetime,
                    player_projectile_movement,
                    update_player_visuals,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Event)]
pub struct PlayerLanded;
#[derive(Event)]
pub struct PlayerAttack;
#[derive(Event)]
#[allow(dead_code)]
pub struct PlayerDamaged { pub amount: i32, pub remaining: i32 }
#[derive(Event)]
pub struct PlayerDied;
/// Fired once when the player starts a dash, carrying position + facing.
#[derive(Event)]
pub struct PlayerDashed {
    pub position: Vec3,
    pub facing: f32,
}

#[derive(Component)]
pub struct Player {
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
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
    pub ranged_cooldown: f32,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub anim_time: f32,
    pub land_squash: f32,
    /// Permanent attack bonus from shop upgrades (stacks per purchase).
    pub bonus_attack: i32,
    /// Permanent defense bonus from shop upgrades.
    pub bonus_defense: i32,
    /// Permanent speed bonus from shop upgrades (0.1 = +10%).
    pub bonus_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            vx: 0.0, vy: 0.0, facing: 1.0,
            max_health: 10, health: 10,
            mana: MANA_MAX, max_mana: MANA_MAX,
            invulnerable: 0.0,
            is_on_floor: false, was_on_floor: false,
            is_jumping: false, jump_hold_time: 0.0,
            coyote_timer: 0.0, jump_buffer: 0.0,
            melee_cooldown: 0.0,
            ranged_cooldown: 0.0,
            is_dashing: false, dash_timer: 0.0, dash_cooldown: 0.0,
            anim_time: 0.0, land_squash: 0.0,
            bonus_attack: 0, bonus_defense: 0, bonus_speed: 0.0,
        }
    }
}

#[derive(Component)]
pub struct MeleeHitbox { pub damage: i32, pub lifetime: f32 }

#[derive(Component)]
pub struct PlayerProjectile {
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub lifetime: f32,
}

// Body part markers
#[derive(Component)] struct PlayerBody;
#[derive(Component)] struct PlayerHead;
#[derive(Component)] struct PlayerLegL;
#[derive(Component)] struct PlayerLegR;
#[derive(Component)] struct PlayerWeapon;

fn spawn_player(mut commands: Commands, meta: Res<MetaProgression>, loaded: Option<Res<LoadedSave>>) {
    let max_hp = if let Some(ref save) = loaded {
        save.0.max_health
    } else {
        10 + meta.max_health_bonus
    };
    let max_mp = if let Some(ref save) = loaded {
        save.0.max_mana
    } else {
        MANA_MAX
    };

    commands.spawn((
        // Invisible collision root
        Sprite { color: Color::srgba(0.0, 0.0, 0.0, 0.0), custom_size: Some(Vec2::new(20.0, 32.0)), ..default() },
        Transform::from_xyz(80.0, 100.0, Z_PLAYER),
        Player { max_health: max_hp, health: max_hp, max_mana: max_mp, mana: max_mp, ..default() },
        PlayingEntity,
    )).with_children(|p| {
        // Body (green tunic)
        p.spawn((
            Sprite { color: Color::srgb(0.55, 0.30, 0.12), custom_size: Some(Vec2::new(14.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1), PlayerBody,
        ));
        // Belt
        p.spawn((
            Sprite { color: Color::srgb(0.40, 0.25, 0.10), custom_size: Some(Vec2::new(14.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, -5.0, 0.15),
        ));
        // Head
        p.spawn((
            Sprite { color: Color::srgb(0.78, 0.62, 0.48), custom_size: Some(Vec2::new(12.0, 11.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.2), PlayerHead,
        )).with_children(|head| {
            // Eyes
            head.spawn((
                Sprite { color: Color::srgb(0.18, 0.12, 0.08), custom_size: Some(Vec2::new(2.5, 3.0)), ..default() },
                Transform::from_xyz(-2.5, 0.5, 0.1),
            ));
            head.spawn((
                Sprite { color: Color::srgb(0.18, 0.12, 0.08), custom_size: Some(Vec2::new(2.5, 3.0)), ..default() },
                Transform::from_xyz(2.5, 0.5, 0.1),
            ));
            // Hood/hair
            head.spawn((
                Sprite { color: Color::srgb(0.45, 0.25, 0.10), custom_size: Some(Vec2::new(14.0, 5.0)), ..default() },
                Transform::from_xyz(0.0, 4.5, 0.15),
            ));
        });
        // Legs
        p.spawn((
            Sprite { color: Color::srgb(0.32, 0.26, 0.2), custom_size: Some(Vec2::new(5.0, 10.0)), ..default() },
            Transform::from_xyz(-3.5, -12.0, 0.0), PlayerLegL,
        ));
        p.spawn((
            Sprite { color: Color::srgb(0.3, 0.24, 0.18), custom_size: Some(Vec2::new(5.0, 10.0)), ..default() },
            Transform::from_xyz(3.5, -12.0, 0.0), PlayerLegR,
        ));
        // Weapon (sword held in front)
        p.spawn((
            Sprite { color: Color::srgb(0.75, 0.65, 0.50), custom_size: Some(Vec2::new(3.0, 16.0)), ..default() },
            Transform::from_xyz(10.0, 3.0, 0.35), PlayerWeapon,
        ));
    });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut Player, &Transform)>,
    mut commands: Commands,
    mut ev_attack: EventWriter<PlayerAttack>,
    mut ev_dashed: EventWriter<PlayerDashed>,
    time: Res<Time>,
    stats: Res<PlayerStats>,
    shop_state: Option<Res<ShopUiState>>,
) {
    let Ok((mut player, tf)) = query.get_single_mut() else { return };
    // Block player movement/actions while shop overlay is open
    if shop_state.map_or(false, |s| s.active) { return; }
    let dt = time.delta_secs();
    let gp = gamepads.iter().next();

    player.anim_time += dt;
    player.melee_cooldown = (player.melee_cooldown - dt).max(0.0);
    player.ranged_cooldown = (player.ranged_cooldown - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    player.jump_buffer = (player.jump_buffer - dt).max(0.0);
    player.land_squash = (player.land_squash - dt * 5.0).max(0.0);

    if player.is_dashing {
        player.dash_timer -= dt;
        if player.dash_timer <= 0.0 { player.is_dashing = false; }
        return;
    }

    // Movement: keyboard + gamepad left stick / dpad
    let mut input_dir = 0.0_f32;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) { input_dir -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { input_dir += 1.0; }
    if let Some(gp) = gp {
        let stick_x = gp.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        if stick_x.abs() > 0.2 { input_dir += stick_x; }
        if gp.pressed(GamepadButton::DPadLeft) { input_dir -= 1.0; }
        if gp.pressed(GamepadButton::DPadRight) { input_dir += 1.0; }
    }
    input_dir = input_dir.clamp(-1.0, 1.0);
    if input_dir != 0.0 { player.facing = input_dir.signum(); }

    let target = input_dir * if player.is_on_floor { MOVE_SPEED } else { AIR_SPEED } * stats.speed_mult;
    let acc = if player.is_on_floor { ACCEL_GROUND } else { ACCEL_AIR };
    if input_dir == 0.0 && player.is_on_floor {
        player.vx = move_toward(player.vx, 0.0, FRICTION * dt);
    } else {
        player.vx = move_toward(player.vx, target, acc * dt);
    }

    // Jump: Space/W/Up + gamepad South(A)/DPadUp
    let gp_jp = gp.map_or(false, |g| g.just_pressed(GamepadButton::South) || g.just_pressed(GamepadButton::DPadUp));
    let gp_jh = gp.map_or(false, |g| g.pressed(GamepadButton::South) || g.pressed(GamepadButton::DPadUp));
    let gp_jr = gp.map_or(false, |g| g.just_released(GamepadButton::South) || g.just_released(GamepadButton::DPadUp));
    let jp = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp) || gp_jp;
    let jh = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) || gp_jh;
    let jr = keys.just_released(KeyCode::Space) || keys.just_released(KeyCode::KeyW) || keys.just_released(KeyCode::ArrowUp) || gp_jr;

    if jp { player.jump_buffer = JUMP_BUFFER_TIME; }
    let can_jump = player.is_on_floor || player.coyote_timer > 0.0;
    if player.jump_buffer > 0.0 && can_jump {
        player.vy = JUMP_SPEED;
        player.is_jumping = true;
        player.jump_hold_time = MAX_JUMP_HOLD;
        player.jump_buffer = 0.0;
        player.coyote_timer = 0.0;
    }
    if player.is_jumping && jh && player.jump_hold_time > 0.0 {
        player.vy += JUMP_HOLD_BOOST * dt;
        player.jump_hold_time = (player.jump_hold_time - dt).max(0.0);
    } else if jr { player.is_jumping = false; }

    // Dash: Shift + gamepad East(B)
    let dash_pressed = keys.just_pressed(KeyCode::ShiftLeft) || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if dash_pressed && player.dash_cooldown <= 0.0 {
        player.is_dashing = true;
        player.dash_timer = DASH_DURATION;
        player.dash_cooldown = DASH_COOLDOWN;
        player.vx = player.facing * DASH_SPEED;
        player.vy = 0.0;
        player.invulnerable = player.invulnerable.max(DASH_DURATION);
        ev_dashed.send(PlayerDashed {
            position: tf.translation,
            facing: player.facing,
        });
    }

    // Melee: E/J/LMB + gamepad West(X)
    let attack = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::KeyJ) || mouse.just_pressed(MouseButton::Left) || gp.map_or(false, |g| g.just_pressed(GamepadButton::West));
    if attack && player.melee_cooldown <= 0.0 {
        player.melee_cooldown = MELEE_COOLDOWN;
        ev_attack.send(PlayerAttack);
        let hx = tf.translation.x + player.facing * MELEE_RANGE;
        commands.spawn((
            Sprite { color: Color::srgba(1.0, 0.75, 0.3, 0.5), custom_size: Some(Vec2::new(MELEE_RANGE, MELEE_WIDTH)), ..default() },
            Transform::from_xyz(hx, tf.translation.y, Z_EFFECTS),
            MeleeHitbox { damage: MELEE_DAMAGE, lifetime: MELEE_ACTIVE_TIME },
            PlayingEntity,
        ));
    }

    // Ranged attack (F key or right-click or gamepad North/Y)
    let ranged = keys.just_pressed(KeyCode::KeyF) || mouse.just_pressed(MouseButton::Right) || gp.map_or(false, |g| g.just_pressed(GamepadButton::North));
    if ranged && player.ranged_cooldown <= 0.0 {
        player.ranged_cooldown = RANGED_COOLDOWN;
        let origin = Vec3::new(
            tf.translation.x + player.facing * 16.0,
            tf.translation.y,
            Z_PROJECTILES,
        );
        commands.spawn((
            Sprite {
                color: Color::srgb(0.95, 0.65, 0.2),
                custom_size: Some(Vec2::new(10.0, 5.0)),
                ..default()
            },
            Transform::from_translation(origin),
            PlayerProjectile {
                vx: player.facing * RANGED_SPEED,
                vy: 0.0,
                damage: RANGED_DAMAGE,
                lifetime: RANGED_LIFETIME,
            },
            PlayingEntity,
        )).with_children(|bolt| {
            // Bright core
            bolt.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.9, 0.5, 0.9),
                    custom_size: Some(Vec2::new(6.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            // Trailing glow
            bolt.spawn((
                Sprite {
                    color: Color::srgba(0.9, 0.5, 0.1, 0.5),
                    custom_size: Some(Vec2::new(14.0, 7.0)),
                    ..default()
                },
                Transform::from_xyz(-3.0, 0.0, -0.1),
            ));
        });
    }

    player.mana = (player.mana + MANA_REGEN_RATE * dt).min(player.max_mana);
}

fn player_physics(
    mut query: Query<(&mut Player, &mut Transform)>,
    tile_q: Query<(&GlobalTransform, &Sprite), (With<crate::room::Tile>, Without<Player>)>,
    mut ev_landed: EventWriter<PlayerLanded>,
    time: Res<Time>,
    mut ev_died: EventWriter<PlayerDied>,
) {
    let Ok((mut player, mut tf)) = query.get_single_mut() else { return };
    let dt = time.delta_secs();

    if player.is_dashing {
        let new_x = tf.translation.x + player.vx * dt;
        let phw = 10.0;
        tf.translation.x = new_x.clamp(phw, ROOM_W - phw);
        if (tf.translation.x - new_x).abs() > 0.1 {
            player.vx = 0.0;
            player.is_dashing = false;
        }
        return;
    }

    if player.vy > -TERMINAL_VELOCITY {
        player.vy = (player.vy - GRAVITY * dt).max(-TERMINAL_VELOCITY);
    }

    let new_x = tf.translation.x + player.vx * dt;
    let new_y = tf.translation.y + player.vy * dt;

    let phw = 10.0;
    let phh = 16.0;
    let cx = new_x.clamp(phw, ROOM_W - phw);
    if (cx - new_x).abs() > 0.1 { player.vx = 0.0; }
    tf.translation.x = cx;

    player.was_on_floor = player.is_on_floor;
    player.is_on_floor = false;
    let mut ry = new_y;

    for (tg, ts) in &tile_q {
        let tp = tg.translation();
        let tsz = ts.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let thw = tsz.x / 2.0;
        let thh = tsz.y / 2.0;

        let dx = (tf.translation.x - tp.x).abs();
        let dy = ry - tp.y;

        if dx < phw + thw - 2.0 {
            if dy > 0.0 && dy < phh + thh && player.vy <= 0.0 {
                let top = tp.y + thh;
                if ry - phh < top && tf.translation.y - phh >= top - 4.0 {
                    ry = top + phh;
                    player.vy = 0.0;
                    player.is_on_floor = true;
                }
            } else if dy < 0.0 && -dy < phh + thh && player.vy > 0.0 {
                let bot = tp.y - thh;
                if ry + phh > bot && tf.translation.y + phh <= bot + 4.0 {
                    ry = bot - phh;
                    player.vy = 0.0;
                }
            }
        }

        let hy = tf.translation.y - tp.y;
        if hy.abs() < phh + thh - 2.0 {
            let hx = tf.translation.x - tp.x;
            if hx > 0.0 && hx < phw + thw && player.vx < 0.0 {
                let tr = tp.x + thw;
                if tf.translation.x - phw < tr {
                    tf.translation.x = tr + phw;
                    player.vx = 0.0;
                }
            } else if hx < 0.0 && -hx < phw + thw && player.vx > 0.0 {
                let tl = tp.x - thw;
                if tf.translation.x + phw > tl {
                    tf.translation.x = tl - phw;
                    player.vx = 0.0;
                }
            }
        }
    }

    tf.translation.y = ry;

    if player.was_on_floor && !player.is_on_floor && player.vy <= 0.0 {
        player.coyote_timer = COYOTE_TIME;
    }
    if !player.is_on_floor {
        player.coyote_timer = (player.coyote_timer - dt).max(0.0);
    }
    if player.is_on_floor && !player.was_on_floor {
        player.is_jumping = false;
        player.land_squash = 1.0;
        ev_landed.send(PlayerLanded);
    }
    if tf.translation.y < -200.0 {
        ev_died.send(PlayerDied);
    }
}

fn player_invuln(mut query: Query<&mut Player>, time: Res<Time>) {
    let Ok(mut player) = query.get_single_mut() else { return };
    if player.invulnerable > 0.0 {
        player.invulnerable = (player.invulnerable - time.delta_secs()).max(0.0);
    }
}

fn melee_hitbox_lifetime(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MeleeHitbox, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut hb, mut sprite) in &mut query {
        hb.lifetime -= time.delta_secs();
        let alpha = (hb.lifetime / MELEE_ACTIVE_TIME).max(0.0) * 0.5;
        sprite.color = Color::srgba(1.0, 0.75, 0.3, alpha);
        if hb.lifetime <= 0.0 { commands.entity(entity).despawn(); }
    }
}

fn player_projectile_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut PlayerProjectile)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut proj) in &mut query {
        tf.translation.x += proj.vx * dt;
        tf.translation.y += proj.vy * dt;
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).try_despawn_recursive();
        }
    }
}

/// Animate player body parts
fn update_player_visuals(
    mut player_q: Query<(&Player, &mut Transform), Without<PlayerBody>>,
    mut body_q: Query<&mut Transform, (With<PlayerBody>, Without<Player>, Without<PlayerHead>, Without<PlayerLegL>, Without<PlayerLegR>, Without<PlayerWeapon>)>,
    mut head_q: Query<&mut Transform, (With<PlayerHead>, Without<Player>, Without<PlayerBody>, Without<PlayerLegL>, Without<PlayerLegR>, Without<PlayerWeapon>)>,
    mut legl_q: Query<&mut Transform, (With<PlayerLegL>, Without<Player>, Without<PlayerBody>, Without<PlayerHead>, Without<PlayerLegR>, Without<PlayerWeapon>)>,
    mut legr_q: Query<&mut Transform, (With<PlayerLegR>, Without<Player>, Without<PlayerBody>, Without<PlayerHead>, Without<PlayerLegL>, Without<PlayerWeapon>)>,
    mut weap_q: Query<(&mut Transform, &mut Sprite), (With<PlayerWeapon>, Without<Player>, Without<PlayerBody>, Without<PlayerHead>, Without<PlayerLegL>, Without<PlayerLegR>)>,
) {
    let Ok((player, mut root_tf)) = player_q.get_single_mut() else { return };
    let t = player.anim_time;
    let running = player.is_on_floor && player.vx.abs() > 30.0;

    // Facing + squash/stretch
    let face = if player.facing < 0.0 { -1.0 } else { 1.0 };
    let (sx, sy) = if player.land_squash > 0.0 {
        (1.0 + player.land_squash * 0.2, 1.0 - player.land_squash * 0.15)
    } else if !player.is_on_floor && player.vy > 50.0 {
        (0.9, 1.1)
    } else if !player.is_on_floor && player.vy < -100.0 {
        (0.92, 1.08)
    } else if player.is_dashing {
        (1.15, 0.85)
    } else {
        (1.0, 1.0)
    };
    root_tf.scale = Vec3::new(face * sx, sy, 1.0);

    // Body bob
    if let Ok(mut b) = body_q.get_single_mut() {
        b.translation.y = if running { (t * 14.0).sin().abs() * 2.0 } else { 0.0 };
    }
    // Head bob
    if let Ok(mut h) = head_q.get_single_mut() {
        h.translation.y = 12.0 + if running { (t * 14.0).sin().abs() * 1.5 } else { (t * 2.0).sin() * 0.5 };
    }
    // Legs
    if let Ok(mut ll) = legl_q.get_single_mut() {
        if running {
            let sw = (t * 14.0).sin() * 6.0;
            ll.translation = Vec3::new(-3.5 + sw * 0.3, -12.0 + sw.abs() * 0.5, 0.0);
        } else if !player.is_on_floor {
            ll.translation = Vec3::new(-4.0, -11.0, 0.0);
        } else {
            ll.translation = Vec3::new(-3.5, -12.0, 0.0);
        }
    }
    if let Ok(mut rl) = legr_q.get_single_mut() {
        if running {
            let sw = (t * 14.0 + std::f32::consts::PI).sin() * 6.0;
            rl.translation = Vec3::new(3.5 + sw * 0.3, -12.0 + sw.abs() * 0.5, 0.0);
        } else if !player.is_on_floor {
            rl.translation = Vec3::new(4.0, -11.0, 0.0);
        } else {
            rl.translation = Vec3::new(3.5, -12.0, 0.0);
        }
    }
    // Weapon swing
    if let Ok((mut wt, mut ws)) = weap_q.get_single_mut() {
        let attacking = player.melee_cooldown > MELEE_COOLDOWN - 0.15;
        if attacking {
            let st = (MELEE_COOLDOWN - player.melee_cooldown) / 0.15;
            wt.rotation = Quat::from_rotation_z(-1.5 + st * 3.0);
            wt.translation = Vec3::new(14.0, 6.0, 0.35);
            ws.color = Color::srgb(0.95, 0.75, 0.35);
        } else {
            wt.rotation = Quat::from_rotation_z(-0.3);
            wt.translation = Vec3::new(10.0, 3.0, 0.35);
            ws.color = Color::srgb(0.75, 0.65, 0.50);
        }
    }
}

fn move_toward(current: f32, target: f32, step: f32) -> f32 {
    if current < target { (current + step).min(target) } else { (current - step).max(target) }
}
