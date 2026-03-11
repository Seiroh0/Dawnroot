<p align="center">
  <img src="https://img.shields.io/badge/Engine-Bevy%200.15-232326?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Language-Rust-B7410E?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Status-In%20Development-yellow?style=for-the-badge" />
  <img src="https://img.shields.io/github/license/Seiroh0/Dawnroot?style=for-the-badge" />
</p>

<h1 align="center">Dawnroot</h1>

<p align="center">
  <em>Descend into the depths. Fight through the dark. Rise as legend.</em>
</p>

<p align="center">
  A horizontal side-scrolling <strong>roguelike platformer</strong> where you leap into an ancient well<br/>
  and battle through procedurally generated dungeon floors filled with enemies, spells, and loot.
</p>

---

## How to Play

Press **SPACE** on the title screen -- your character approaches the well and leaps into the unknown. Fight through rooms from left to right, defeat all enemies to unlock the exit door, gather gold, buy upgrades in shops, and face the floor boss. Death means starting over.

### Room Progression

```
 Start  -->  Combat  -->  Combat  -->  Treasure  -->  Combat  -->  Boss
                                                          |
                                                        Shop (even floors)
```

---

## Controls

| Action | Key |
|:-------|:----|
| Move | `A` / `D` or `Arrow Keys` |
| Jump | `Space` / `W` / `Up` |
| Melee Attack | `J` / `Left Click` |
| Dash (i-frames) | `Left Shift` |
| Fireball | `1` |
| Ice Shards | `2` |
| Lightning | `3` |
| Shield | `4` |
| Buy Item (Shop) | `E` |

---

## Features

<table>
<tr>
<td width="50%">

### Combat & Movement
- Tight platforming with **coyote time**, **jump buffering**, and **variable jump height**
- Melee sword with animated swing arc
- **Dash** with invincibility frames and afterimage trail
- **4 Spells** -- Fireball (flame trail), Ice Shards (spread shot), Lightning (AoE bolts), Shield (rotating barrier)
- Squash & stretch on jump/land, landing dust puffs

</td>
<td width="50%">

### Enemies & Boss
- **Goblin** -- patrols and chases, animated legs
- **Bat** -- wave movement with flapping wings
- **Stone Turret** -- aims and shoots, rotating eye
- **Boar** -- detects and charges, horns tilt forward
- **Floor Boss** -- large multi-part sprite with crown and claws

</td>
</tr>
<tr>
<td>

### World & Atmosphere
- **Well intro cutscene** -- animated descent into the dungeon
- **8 combat room templates** -- staircase, arena, pit bridges, towers, zigzag, floating islands, walkways, tunnel
- **Visual decorations** -- flickering torches, pulsing crystals, stalactites, mushrooms, moss & cracks
- Unique color palette per room type

</td>
<td>

### Progression & Loot
- **Gold, Health, Mana** drops with magnet pickup
- **In-room shop** -- heal, upgrade max HP, expand mana pool
- **Meta-progression** -- persistent upgrades between runs
- **Score system** with combo potential
- Room-cleared confetti celebration

</td>
</tr>
</table>

---

## Tech Stack

| | |
|:--|:--|
| **Engine** | [Bevy 0.15](https://bevyengine.org/) |
| **Language** | Rust (Edition 2024) |
| **Rendering** | Procedural pixel sprites from layered colored rectangles |
| **Architecture** | ECS with state machine (Title, WellIntro, Playing, Paused, Shop, GameOver) |
| **Physics** | Custom AABB tile collision |
| **Dependencies** | `bevy 0.15`, `rand 0.8`, `serde 1` |

---

## Project Structure

```
src/
 |- main.rs        App setup, GameState, RunData, MetaProgression
 |- constants.rs   All numeric constants (physics, spells, layout)
 |- title.rs       Title screen + animated WellIntro cutscene
 |- player.rs      Multi-part player sprite, movement, melee, dash
 |- room.rs        Room generation, 8+ templates, decorations
 |- enemy.rs       5 enemy types with animated multi-part sprites
 |- combat.rs      Melee / spell / projectile collision & damage
 |- spell.rs       4 spells with trails, frost, bolt lines, shield ring
 |- effects.rs     Particles, afterimages, dust, confetti, flash
 |- hud.rs         UI overlay (HP, mana, gold, floor, spell cooldowns)
 |- camera.rs      Smooth follow camera + screen shake
 |- animation.rs   Generic frame-based animation support
 |- loot.rs        Loot drops with magnet pickup
 |- shop.rs        In-room shop purchases
```

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/Seiroh0/Dawnroot.git
cd Dawnroot

# Run the game
cargo run
```

> Requires the [Rust toolchain](https://rustup.rs/). Uses `dynamic_linking` for fast dev builds.

---

## Roadmap

- [x] Multi-part procedural player sprite with walk, jump, dash animations
- [x] 5 enemy types with animated sprites (Goblin, Bat, Turret, Boar, Boss)
- [x] 4 spell system with visual trails and effects
- [x] Well intro cutscene with walk + jump animation
- [x] 8 combat room templates with decorations
- [x] Room lock mechanic (enemies must be defeated to proceed)
- [x] Particle effects (death, damage, confetti, dust, afterimages)
- [ ] Spritesheet art (replace procedural rectangles)
- [ ] Audio engine (background music + SFX per action)
- [ ] Save / Load for meta-progression
- [ ] More floor templates & enemy variants
- [ ] Advanced room decorations (water, lava, moving platforms)
- [ ] NPC dialogue system
- [ ] Game balance pass & difficulty scaling
- [ ] Controller / gamepad support
- [ ] Screen transitions & fade effects between rooms

---

## License

MIT License -- see [LICENSE](./LICENSE).

---

<p align="center">
  <em>Enter the roots of dawn.</em>
</p>
