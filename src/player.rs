use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, MetaProgression, LoadedSave, equipment::{PlayerStats, ItemId, Equipment}, shop::ShopUiState, audio::{PlaySfxEvent, SfxType}, game_feel::{ShakeEvent, AttackPulse}};

// ─── Sprite Assets ───────────────────────────────────────────────────────────

/// Holds the loaded satiro spritesheet texture + atlas layout.
#[derive(Resource)]
pub struct PlayerSpriteAssets {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Holds pre-loaded weapon sprite handles, one per weapon ItemId.
#[derive(Resource)]
pub struct WeaponSpriteAssets {
    pub rusty_sword:     Handle<Image>,
    pub steel_blade:     Handle<Image>,
    pub flame_edge:      Handle<Image>,
    pub frost_fang:      Handle<Image>,
    pub thunder_cleaver: Handle<Image>,
}

impl WeaponSpriteAssets {
    pub fn handle_for(&self, id: ItemId) -> Option<&Handle<Image>> {
        match id {
            ItemId::RustySword      => Some(&self.rusty_sword),
            ItemId::SteelBlade      => Some(&self.steel_blade),
            ItemId::FlameEdge       => Some(&self.flame_edge),
            ItemId::FrostFang       => Some(&self.frost_fang),
            ItemId::ThunderCleaver  => Some(&self.thunder_cleaver),
            _                       => None,
        }
    }
}

/// Tracks the player's current animation state and frame timer.
#[derive(Component)]
pub struct PlayerAnimState {
    pub state: AnimKind,
    pub frame: usize,
    pub timer: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimKind { Idle, Run, Jump, Fall }

/// Satiro sheet: 320×256 → 10 columns × 8 rows of 32×32 frames.
const SATIRO_COLS: u32 = 10;
const SATIRO_ROWS: u32 = 8;
const SATIRO_FRAME: u32 = 32;
/// How many frames per animation (first N of each row).
const IDLE_FRAMES: usize = 4;
const RUN_FRAMES: usize = 4;
/// Row indices in the sheet.
const IDLE_ROW: usize = 0;
const RUN_ROW: usize = 1;
/// Seconds per animation frame.
const ANIM_FPS: f32 = 0.12;
/// Display scale for the 32px sprite → ~64px on screen.
const PLAYER_SPRITE_SCALE: f32 = 2.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Load the satiro spritesheet and build the atlas layout.
        let asset_server = app.world().resource::<AssetServer>();
        let texture: Handle<Image> = asset_server.load("sprites/satiro-Sheet v1.1.png");
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(SATIRO_FRAME, SATIRO_FRAME),
            SATIRO_COLS,
            SATIRO_ROWS,
            None,
            None,
        );

        // Load weapon sprites at startup (path has spaces — exact string required).
        let weapon_base = "Weapons/Weapons Asset 16x16/Weapons Asset 16x16/Weapons Asset 16x16/";
        let rusty_sword:     Handle<Image> = asset_server.load(format!("{weapon_base}001.png"));
        let steel_blade:     Handle<Image> = asset_server.load(format!("{weapon_base}010.png"));
        let flame_edge:      Handle<Image> = asset_server.load(format!("{weapon_base}020.png"));
        let frost_fang:      Handle<Image> = asset_server.load(format!("{weapon_base}050.png"));
        let thunder_cleaver: Handle<Image> = asset_server.load(format!("{weapon_base}070.png"));

        let mut layouts = app.world_mut().resource_mut::<Assets<TextureAtlasLayout>>();
        let layout_handle = layouts.add(layout);
        app.insert_resource(PlayerSpriteAssets { texture, layout: layout_handle });
        app.insert_resource(WeaponSpriteAssets {
            rusty_sword,
            steel_blade,
            flame_edge,
            frost_fang,
            thunder_cleaver,
        });

        app.add_event::<PlayerLanded>()
            .add_event::<PlayerAttack>()
            .add_event::<PlayerDamaged>()
            .add_event::<PlayerDied>()
            .add_event::<PlayerDashed>()
            .add_event::<PlayerBlocked>()
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
                    animate_player_sprite,
                    animate_weapon_swing,
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
/// Fired when the player successfully blocks an attack.
#[derive(Event)]
pub struct PlayerBlocked {
    pub position: Vec3,
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
    pub is_blocking: bool,
    pub block_timer: f32,
    pub block_cooldown: f32,
    pub land_squash: f32,
    /// Permanent attack bonus from shop upgrades (stacks per purchase).
    pub bonus_attack: i32,
    /// Permanent defense bonus from shop upgrades.
    pub bonus_defense: i32,
    /// Permanent speed bonus from shop upgrades (0.1 = +10%).
    pub bonus_speed: f32,
    /// Tracks how much max_health comes from equipment (for recalc delta).
    pub equipment_health_bonus: i32,
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
            is_blocking: false, block_timer: 0.0, block_cooldown: 0.0,
            land_squash: 0.0,
            bonus_attack: 0, bonus_defense: 0, bonus_speed: 0.0,
            equipment_health_bonus: 0,
        }
    }
}

#[derive(Component)]
pub struct MeleeHitbox {
    pub damage: i32,
    pub lifetime: f32,
    /// Enemies already hit by this swing (prevents multi-hit per swing).
    pub hit_entities: Vec<Entity>,
}

#[derive(Component)]
pub struct PlayerProjectile {
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub lifetime: f32,
}

/// Marker for the child entity that renders the player spritesheet.
#[derive(Component)]
pub struct PlayerSprite;

/// Marker for the child entity that renders the held weapon sprite.
#[derive(Component)]
pub struct WeaponSprite;

/// Animates a weapon swing: rotates ±90° over `duration`, then returns.
#[derive(Component)]
pub struct WeaponSwingAnim {
    /// Total duration of one full swing-and-return (seconds).
    pub duration: f32,
    /// Current elapsed time.
    pub elapsed: f32,
    /// Starting rotation (radians) — set when the animation begins.
    pub base_angle: f32,
    /// Direction of initial swing (+1 = clockwise, -1 = counter-clockwise).
    pub direction: f32,
}

fn spawn_player(
    mut commands: Commands,
    meta: Res<MetaProgression>,
    loaded: Option<Res<LoadedSave>>,
    existing: Query<&Player>,
    sprite_assets: Res<PlayerSpriteAssets>,
    weapon_assets: Res<WeaponSpriteAssets>,
) {
    if existing.iter().next().is_some() { return; }
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
        // Invisible collision root (keeps original hitbox size)
        Sprite { color: Color::srgba(0.0, 0.0, 0.0, 0.0), custom_size: Some(Vec2::new(20.0, 32.0)), ..default() },
        Transform::from_xyz(80.0, 100.0, Z_PLAYER),
        Player { max_health: max_hp, health: max_hp, max_mana: max_mp, mana: max_mp, ..default() },
        PlayingEntity,
    )).with_children(|p| {
        // Spritesheet child — renders the satiro character
        let sprite_size = SATIRO_FRAME as f32 * PLAYER_SPRITE_SCALE;
        p.spawn((
            Sprite {
                image: sprite_assets.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_assets.layout.clone(),
                    index: IDLE_ROW * SATIRO_COLS as usize,
                }),
                custom_size: Some(Vec2::new(sprite_size, sprite_size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            PlayerSprite,
            PlayerAnimState {
                state: AnimKind::Idle,
                frame: 0,
                timer: 0.0,
            },
        ));

        // Weapon sprite child — initially hidden (no weapon equipped)
        p.spawn((
            Sprite {
                image: weapon_assets.rusty_sword.clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(12.0, -2.0, 0.2),
            Visibility::Hidden,
            WeaponSprite,
        ));
    });
}

fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(Entity, &mut Player, &Transform)>,
    mut commands: Commands,
    mut ev_attack: EventWriter<PlayerAttack>,
    mut ev_dashed: EventWriter<PlayerDashed>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
    time: Res<Time>,
    stats: Res<PlayerStats>,
    shop_state: Option<Res<ShopUiState>>,
    weapon_sprite_q: Query<Entity, With<WeaponSprite>>,
) {
    let Ok((player_entity, mut player, tf)) = query.get_single_mut() else { return };
    // Block player movement/actions while shop overlay is open
    if shop_state.map_or(false, |s| s.active) {
        player.vx = 0.0;
        return;
    }
    let dt = time.delta_secs();
    let gp = gamepads.iter().next();

    player.melee_cooldown = (player.melee_cooldown - dt).max(0.0);
    player.ranged_cooldown = (player.ranged_cooldown - dt).max(0.0);
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    player.block_cooldown = (player.block_cooldown - dt).max(0.0);
    player.jump_buffer = (player.jump_buffer - dt).max(0.0);
    player.land_squash = (player.land_squash - dt * 5.0).max(0.0);

    // Block timer
    if player.is_blocking {
        player.block_timer -= dt;
        if player.block_timer <= 0.0 {
            player.is_blocking = false;
        }
    }

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
        ev_sfx.send(PlaySfxEvent(SfxType::Jump));
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

    // Block: K / RMB + gamepad North(Y)
    let block_pressed = keys.just_pressed(KeyCode::KeyK) || mouse.just_pressed(MouseButton::Right) || gp.map_or(false, |g| g.just_pressed(GamepadButton::North));
    if block_pressed && player.block_cooldown <= 0.0 && !player.is_blocking {
        player.is_blocking = true;
        player.block_timer = BLOCK_DURATION;
        player.block_cooldown = BLOCK_COOLDOWN;
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
            MeleeHitbox { damage: MELEE_DAMAGE, lifetime: MELEE_ACTIVE_TIME, hit_entities: Vec::new() },
            PlayingEntity,
        ));
        // Attack pulse on root entity
        commands.entity(player_entity).insert(AttackPulse {
            timer: 0.0,
            duration: 0.15,
        });
        // Trigger weapon swing animation
        if let Ok(weapon_entity) = weapon_sprite_q.get_single() {
            commands.entity(weapon_entity).insert(WeaponSwingAnim {
                duration: MELEE_COOLDOWN * 0.8,
                elapsed: 0.0,
                base_angle: 0.0,
                direction: player.facing,
            });
        }
    }

    // Ranged attack (F key only — RMB/North reserved for Block)
    let ranged = keys.just_pressed(KeyCode::KeyF);
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

    player.mana = (player.mana + MANA_REGEN_RATE * stats.mana_regen_mult * dt).min(player.max_mana);
}

fn player_physics(
    mut query: Query<(&mut Player, &mut Transform)>,
    tile_q: Query<(&GlobalTransform, &Sprite), (With<crate::room::Tile>, Without<Player>)>,
    mut ev_landed: EventWriter<PlayerLanded>,
    time: Res<Time>,
    mut ev_died: EventWriter<PlayerDied>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
    mut ev_shake: EventWriter<ShakeEvent>,
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
        ev_sfx.send(PlaySfxEvent(SfxType::PlayerDeath));
        ev_shake.send(ShakeEvent { trauma: 0.5 });
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

/// Apply facing, squash/stretch, and invuln flash to the player root + sprite.
/// Also syncs the weapon sprite position/visibility to the equipped weapon.
fn update_player_visuals(
    player_q: Query<(Entity, &Player), Without<PlayerSprite>>,
    mut root_tf_q: Query<&mut Transform, (With<Player>, Without<PlayerSprite>, Without<WeaponSprite>)>,
    mut sprite_q: Query<(&mut Sprite, &mut Transform), (With<PlayerSprite>, Without<Player>, Without<WeaponSprite>)>,
    mut weapon_q: Query<(&mut Sprite, &mut Transform, &mut Visibility), (With<WeaponSprite>, Without<PlayerSprite>, Without<Player>)>,
    equip_q: Query<&Equipment>,
    weapon_assets: Res<WeaponSpriteAssets>,
) {
    let Ok((player_entity, player)) = player_q.get_single() else { return };
    let Ok(mut root_tf) = root_tf_q.get_single_mut() else { return };

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

    // Invulnerability flash (blink the sprite) + jump/fall tilt
    if let Ok((mut sprite, mut sprite_tf)) = sprite_q.get_single_mut() {
        if player.invulnerable > 0.0 {
            let blink = ((player.invulnerable * 15.0) as i32 % 2) == 0;
            sprite.color = if blink { Color::srgba(1.0, 1.0, 1.0, 0.3) } else { Color::WHITE };
        } else {
            sprite.color = Color::WHITE;
        }

        // Slight tilt when airborne
        let tilt = if !player.is_on_floor && player.vy > 50.0 {
            0.26 * face // ~15° lean back when jumping
        } else if !player.is_on_floor && player.vy < -50.0 {
            -0.17 * face // ~10° lean forward when falling
        } else {
            0.0
        };
        sprite_tf.rotation = Quat::from_rotation_z(tilt);
    }

    // Weapon sprite: position, flip, and image based on equipped weapon.
    if let Ok((mut w_sprite, mut w_tf, mut w_vis)) = weapon_q.get_single_mut() {
        let equipped_weapon = equip_q.get(player_entity).ok().and_then(|e| e.weapon);
        match equipped_weapon {
            None => {
                *w_vis = Visibility::Hidden;
            }
            Some(item_id) => {
                *w_vis = Visibility::Inherited;
                // Offset to the right of the player in local space; root scale handles flip.
                w_tf.translation.x = 12.0;
                w_tf.translation.y = -2.0;
                w_tf.translation.z = 0.2;
                // Counter-flip so the weapon sprite stays unmirrored in world space
                // when the player root is flipped.
                w_sprite.flip_x = player.facing < 0.0;
                if let Some(handle) = weapon_assets.handle_for(item_id) {
                    w_sprite.image = handle.clone();
                }
            }
        }
    }
}

/// Cycle spritesheet frames based on player movement state.
fn animate_player_sprite(
    player_q: Query<&Player>,
    mut anim_q: Query<(&mut PlayerAnimState, &mut Sprite), With<PlayerSprite>>,
    time: Res<Time>,
) {
    let Ok(player) = player_q.get_single() else { return };
    let Ok((mut anim, mut sprite)) = anim_q.get_single_mut() else { return };

    // Determine desired animation kind
    let desired = if !player.is_on_floor && player.vy > 50.0 {
        AnimKind::Jump
    } else if !player.is_on_floor && player.vy < -50.0 {
        AnimKind::Fall
    } else if player.is_on_floor && player.vx.abs() > 30.0 {
        AnimKind::Run
    } else {
        AnimKind::Idle
    };

    // Reset frame on state change
    if anim.state != desired {
        anim.state = desired;
        anim.frame = 0;
        anim.timer = 0.0;
    }

    // Advance timer
    anim.timer += time.delta_secs();
    let (row, frame_count, freeze) = match anim.state {
        AnimKind::Idle => (IDLE_ROW, IDLE_FRAMES, false),
        AnimKind::Run  => (RUN_ROW, RUN_FRAMES, false),
        // Jump/Fall: use last run frame, frozen (no animation loop)
        AnimKind::Jump | AnimKind::Fall => (RUN_ROW, RUN_FRAMES, true),
    };
    if freeze {
        anim.frame = frame_count - 1; // freeze on last run frame
    } else {
        while anim.timer >= ANIM_FPS {
            anim.timer -= ANIM_FPS;
            anim.frame = (anim.frame + 1) % frame_count;
        }
    }

    // Update atlas index
    let index = row * SATIRO_COLS as usize + anim.frame;
    if let Some(ref mut atlas) = sprite.texture_atlas {
        atlas.index = index;
    }
}

/// Rotates the weapon sprite through a ±90° arc over `duration` seconds, then snaps back.
/// The component is removed when the animation completes.
fn animate_weapon_swing(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut WeaponSwingAnim), With<WeaponSprite>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut anim) in &mut query {
        anim.elapsed += dt;
        let t = (anim.elapsed / anim.duration).clamp(0.0, 1.0);

        // First half: swing forward 90°; second half: return to rest.
        let angle = if t < 0.5 {
            anim.direction * std::f32::consts::FRAC_PI_2 * (t / 0.5)
        } else {
            anim.direction * std::f32::consts::FRAC_PI_2 * (1.0 - (t - 0.5) / 0.5)
        };
        tf.rotation = Quat::from_rotation_z(anim.base_angle + angle);

        if t >= 1.0 {
            tf.rotation = Quat::IDENTITY;
            commands.entity(entity).remove::<WeaponSwingAnim>();
        }
    }
}

fn move_toward(current: f32, target: f32, step: f32) -> f32 {
    if current < target { (current + step).min(target) } else { (current - step).max(target) }
}
