# Dawnroot

A horizontal side-scrolling roguelike platformer built with **Bevy 0.15** (Rust).

Descend through the ancient well into procedurally generated dungeon floors. Fight enemies with sword and spells, collect loot, buy upgrades, and survive through floors of increasing difficulty.

---

## Gameplay

Press SPACE on the title screen to watch your character approach the well and leap into the depths. Clear rooms from left to right, defeat enemies, gather gold, visit shops, and face the boss at the end of each floor. Death means starting over -- roguelike style.

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

- **Well Intro Cutscene** -- animated sequence of the player jumping into the well before gameplay begins
- **Multi-part Procedural Sprites** -- player, enemies, and bosses built from layered colored rectangles with animated body parts
- **Tight Platforming** -- coyote time, jump buffering, variable jump height, squash/stretch on land
- **Melee + 4 Spells** -- sword with swing arc, Fireball (trail particles), Ice Shards (spread shot), Lightning (AoE bolt lines), Shield (rotating barrier)
- **Dash** with i-frames and afterimage trail
- **5 Enemy Types** -- Goblin (patrol + chase), Bat (wave + wing flap), Stone Turret (aim + shoot), Boar (detect + charge), Boss (large multi-part)
- **Decorated Dungeons** -- torches with flickering glow, crystals, stalactites, mushrooms, moss/cracks, varied palettes per room type
- **8 Combat Room Templates** -- staircase, multi-level arena, pit bridges, towers, zigzag, floating islands, elevated walkways, tunnel
- **Loot & Shop** -- gold/health/mana drops with magnet pickup, in-room shop for upgrades
- **Particle Effects** -- enemy death bursts, damage flash, landing dust, confetti on room clear, spell trails
- **Screen Shake** on combat hits
- **Meta-Progression** -- persistent upgrades between runs

---

## Tech Stack

- **Engine:** Bevy 0.15
- **Language:** Rust (Edition 2024)

---

## Project Structure

```
src/
├── main.rs       # App setup, GameState, RunData, MetaProgression
├── constants.rs  # All numeric constants (physics, spells, layout)
├── title.rs      # Title screen + WellIntro cutscene animation
├── player.rs     # Multi-part player, movement, melee, dash, physics
├── room.rs       # Room generation, 8+ templates, decorations (torches, crystals)
├── enemy.rs      # 5 enemy types with multi-part sprites and AI animations
├── combat.rs     # Melee/spell/projectile collision and damage
├── spell.rs      # 4 spells with enhanced visuals (fire trail, frost, bolts, shield ring)
├── effects.rs    # Particles, afterimages, dust, confetti, flash effects
├── hud.rs        # UI overlay (health, mana, gold, floor, spells)
├── camera.rs     # Camera follow + screen shake
├── animation.rs  # Generic frame-based animation support
├── loot.rs       # Drop and pickup system with magnet
└── shop.rs       # In-room shop purchases
```

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
