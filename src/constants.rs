#![allow(dead_code)]

// Viewport (widescreen for horizontal scrolling)
pub const VIEWPORT_W: f32 = 960.0;
pub const VIEWPORT_H: f32 = 540.0;

// Room grid
pub const TILE_SIZE: f32 = 40.0;
pub const ROOM_COLUMNS: i32 = 24;
pub const ROOM_ROWS: i32 = 14;
pub const ROOM_W: f32 = ROOM_COLUMNS as f32 * TILE_SIZE; // 960
pub const ROOM_H: f32 = ROOM_ROWS as f32 * TILE_SIZE; // 560

// Roguelike structure
pub const ROOMS_PER_FLOOR: i32 = 6;
pub const FLOORS_PER_RUN: i32 = 4;

// Player physics
pub const GRAVITY: f32 = 1400.0;
pub const TERMINAL_VELOCITY: f32 = 700.0;
pub const MOVE_SPEED: f32 = 260.0;
pub const AIR_SPEED: f32 = 240.0;
pub const ACCEL_GROUND: f32 = 2000.0;
pub const ACCEL_AIR: f32 = 1000.0;
pub const FRICTION: f32 = 1600.0;
pub const JUMP_SPEED: f32 = 620.0;
pub const MAX_JUMP_HOLD: f32 = 0.25;
pub const JUMP_HOLD_BOOST: f32 = 500.0;
pub const COYOTE_TIME: f32 = 0.1;
pub const JUMP_BUFFER_TIME: f32 = 0.12;
pub const INVULN_TIME: f32 = 1.0;

// Dash
pub const DASH_SPEED: f32 = 600.0;
pub const DASH_DURATION: f32 = 0.15;
pub const DASH_COOLDOWN: f32 = 0.8;

// Melee attack
pub const MELEE_RANGE: f32 = 36.0;
pub const MELEE_WIDTH: f32 = 28.0;
pub const MELEE_DAMAGE: i32 = 1;
pub const MELEE_COOLDOWN: f32 = 0.35;
pub const MELEE_ACTIVE_TIME: f32 = 0.1;

// Ranged attack (energy bolt)
pub const RANGED_SPEED: f32 = 420.0;
pub const RANGED_DAMAGE: i32 = 1;
pub const RANGED_COOLDOWN: f32 = 0.45;
pub const RANGED_LIFETIME: f32 = 1.2;

// Spells
pub const SPELL_SLOT_COUNT: usize = 4;
pub const MANA_MAX: f32 = 100.0;
pub const MANA_REGEN_RATE: f32 = 8.0;

// Spell: Fireball
pub const FIREBALL_SPEED: f32 = 500.0;
pub const FIREBALL_DAMAGE: i32 = 3;
pub const FIREBALL_MANA_COST: f32 = 25.0;
pub const FIREBALL_COOLDOWN: f32 = 1.0;
pub const FIREBALL_LIFETIME: f32 = 1.5;

// Spell: Ice Shards
pub const ICE_SHARD_SPEED: f32 = 600.0;
pub const ICE_SHARD_DAMAGE: i32 = 1;
pub const ICE_SHARD_MANA_COST: f32 = 20.0;
pub const ICE_SHARD_COOLDOWN: f32 = 0.6;
pub const ICE_SHARD_COUNT: i32 = 3;

// Spell: Lightning
pub const LIGHTNING_DAMAGE: i32 = 4;
pub const LIGHTNING_MANA_COST: f32 = 35.0;
pub const LIGHTNING_COOLDOWN: f32 = 1.5;
pub const LIGHTNING_RADIUS: f32 = 120.0;

// Spell: Shield
pub const SHIELD_DURATION: f32 = 2.0;
pub const SHIELD_MANA_COST: f32 = 30.0;
pub const SHIELD_COOLDOWN: f32 = 5.0;

// Collision layers (bitmask)
pub const LAYER_WORLD: u32 = 1;
pub const LAYER_PLAYER: u32 = 2;
pub const LAYER_ENEMY: u32 = 4;
pub const LAYER_PLAYER_SHOT: u32 = 8;
pub const LAYER_ENEMY_SHOT: u32 = 16;
pub const LAYER_PICKUP: u32 = 32;

// Z-ordering
pub const Z_BACKGROUND: f32 = -100.0;
pub const Z_TILES: f32 = 0.0;
pub const Z_PICKUPS: f32 = 3.0;
pub const Z_ENEMIES: f32 = 5.0;
pub const Z_PLAYER: f32 = 10.0;
pub const Z_PROJECTILES: f32 = 15.0;
pub const Z_EFFECTS: f32 = 20.0;
pub const Z_HUD: f32 = 100.0;
