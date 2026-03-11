# Dawnroot

A horizontal side-scrolling roguelike platformer built with **Bevy 0.15** (Rust).

Explore procedurally generated rooms, fight enemies with melee attacks and spells, collect loot, and survive through floors of increasing difficulty.

---

## Gameplay

Run through rooms from left to right, clear enemies, collect gold, buy upgrades in shops, and defeat the boss at the end of each floor. Death means starting over -- roguelike style.

**Room Types:** Start, Combat, Treasure, Shop, Boss

---

## Controls

| Action | Key |
|--------|-----|
| Move | `A` / `D` or Arrow Keys |
| Jump | `Space` / `W` / `Up` |
| Melee Attack | `J` / Left Click |
| Dash | `Left Shift` |
| Spell 1 (Fireball) | `1` |
| Spell 2 (Ice Shards) | `2` |
| Spell 3 (Lightning) | `3` |
| Spell 4 (Shield) | `4` |
| Buy (Shop) | `E` |

---

## Features

- Horizontal platforming with coyote time, jump buffering, and variable jump height
- Melee combat + 4 spell system (Fireball, Ice Shards, Lightning, Shield) with mana and cooldowns
- Dash with invincibility frames
- 4 enemy types: Ground (patrol + chase), Flying (wave + follow), Turret (aim + shoot), Charger (detect + rush)
- Boss fights at the end of each floor
- Loot drops (gold, health, mana) with magnet pickup
- In-room shop for upgrades
- Particle effects for combat feedback
- Screen shake on hits
- Procedural room generation with multiple templates

---

## Tech Stack

- **Engine:** Bevy 0.15
- **Language:** Rust
- **AI Tools:** Claude (coding assistance)

---

## Project Structure

```
Dawnroot/
├── assets/          # Sprites, audio, backgrounds
├── src/
│   ├── main.rs      # App setup, game states, resources
│   ├── constants.rs  # All game constants
│   ├── player.rs    # Player movement, melee, dash, physics
│   ├── room.rs      # Room generation, templates, transitions
│   ├── enemy.rs     # Enemy types and AI
│   ├── combat.rs    # Collision and damage systems
│   ├── spell.rs     # Spell system (4 spells)
│   ├── hud.rs       # UI overlay
│   ├── camera.rs    # Camera follow + screen shake
│   ├── effects.rs   # Particle effects
│   ├── animation.rs # Sprite animation state machine
│   ├── loot.rs      # Drop and pickup system
│   ├── shop.rs      # In-room shop
│   └── title.rs     # Title screen
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## Development Status

Work in progress. Currently using placeholder colored rectangles -- sprite art, audio, and meta-progression save/load are next.

---

## Building

```bash
cargo run
```

Requires Rust toolchain. Uses `dynamic_linking` for fast dev builds.

---

## License

MIT License. See [LICENSE](./LICENSE).

---

> *Enter the roots of dawn.*
