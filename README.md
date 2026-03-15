<p align="center">
  <img src="https://img.shields.io/badge/Engine-Bevy%200.15-232326?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Language-Rust-B7410E?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/Status-In%20Development-yellow?style=for-the-badge" />
  <img src="https://img.shields.io/github/license/Seiroh0/Dawnroot?style=for-the-badge" />
</p>

<h1 align="center">Dawnroot</h1>

<p align="center">
  <em>Descend into the depths. Fight through the dark. Rise as a legend.</em>
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
 Start --> Combat x N --> Treasure --> Combat --> [Altar] --> Shop --> Boss
           (scales with floor)              (floor 2+)
```

---

## Controls

| Action | Key |
|:-------|:----|
| Move | `A` / `D` or `Arrow Keys` |
| Jump | `Space` / `W` / `Up` |
| Melee Attack | `J` / `Left Click` |
| Block (70% dmg reduction) | `K` / `Right Click` |
| Dash (i-frames) | `Left Shift` |
| Fireball | `1` |
| Ice Shards | `2` |
| Lightning | `3` |
| Shield | `4` |
| Open Shop / Buy | `E` / `Enter` |
| Navigate Shop | `Up` / `Down` |
| Close Shop | `Escape` |
| Pause Menu | `Escape` (in-game) |

---

## Features

<table>
<tr>
<td width="50%">

### Combat & Movement
- Tight platforming with **coyote time**, **jump buffering**, and **variable jump height**
- Melee sword with animated swing arc
- **Dash** with invincibility frames and afterimage trail
- **Block** -- 70% damage reduction with 3s cooldown, shield flash effect
- **Traps** -- Arrow traps (wall-mounted, periodic fire), spike floors (retractable), poison clouds (DOT area)
- **4 Spells** -- Fireball (flame trail), Ice Shards (spread shot), Lightning (AoE bolts), Shield (rotating barrier)
- Squash & stretch on jump/land, landing dust puffs

</td>
<td width="50%">

### Enemies & Bosses
- **Goblin** -- patrols and chases, animated legs, **leap attack**
- **Bat** -- wave movement with flapping wings, **dive bomb**
- **Stone Turret** -- aims and shoots, rotating eye, **burst fire**
- **Boar** -- detects and charges, horns tilt forward, **ground shockwave**
- **Mage** -- teleports when player approaches, casts purple fireballs, goes invisible
- **Slime** -- hops toward player, **splits into 2 small slimes on death**
- **Ghost** -- phases in/out of existence (intangible when phased), floats toward player
- **4 Unique Floor Bosses** -- Warlord (floor 1), Mushroom King (floor 2), Lava Golem (floor 3), Root Guardian (floor 4) -- each with distinct AI and multi-part sprites
- **Boss phases** -- 50% HP aggressive, 25% HP enraged with AoE slam shockwave
- **Elite Enemies** -- rare glowing variants (Armored, Swift, Brutal) with 2x HP, bonus loot, pulsing aura
- **Floating damage numbers** on all hits (white, yellow crit, red player damage, blue block)
- **Enemy health bars** above each enemy (green / yellow / red)

</td>
</tr>
<tr>
<td>

### World & Atmosphere
- **Well intro cutscene** -- animated descent into the dungeon
- **4 Biomes** -- Dark Dungeon (floor 1), Mushroom Cave (floor 2), Lava Caverns (floor 3), Root Depths (floor 4) -- each with unique tile colors, backgrounds, and ambient decorations
- **16 combat room templates** -- staircase, arena, pit bridges, towers, zigzag, floating islands, walkways, tunnel, lava gauntlet, swamp marsh, elevator shaft, split path, pillared hall, crumbling ruins, the pit, alternating hazards
- **Visual decorations** -- flickering torches, pulsing crystals, stalactites, mushrooms, glowing moss, embers, root tendrils
- **Secret rooms** -- cracked walls (30% spawn chance, floor 2+) can be broken with 3 melee hits to reveal hidden loot
- **3-part platform visuals** -- beveled edge caps, surface highlights, bottom shadows
- **Altar room** -- curse/blessing choice with crystal flanks, rune floor, mystical candles
- **Treasure room** -- gold coin piles, goblets, gemstones, golden banners, animated glow
- **Boss arena** -- ritual circle, skull decorations, crimson banners, chains, pulsing floor glow, pillar capitals
- Unique color palette per room type

</td>
<td>

### Progression & Loot
- **Gold, Health, Mana** drops with magnet pickup
- **Cuphead-style shop UI** -- stone merchant NPC, overlay panel with item list, keyboard/gamepad navigation
- **Tiered shop** -- 30+ items across 3 tiers, milestone-gated unlocks
- **Equipment system** -- 20 items across 4 slots (Weapon, Armor, Relic, Charm)
- **3 item sets** (Fire, Ice, Storm) with 2-piece and 3-piece bonuses
- **Stat upgrades** -- Attack, Defense, Speed purchasable in shop
- **Meta-progression** -- persistent upgrades between runs
- **Score system** with gold bonus from equipment
- Room-cleared confetti celebration
- **Relic choice after boss** -- pick 1 of 3 random relics (10 relics with procedural pixel art icons and passive effects like crit, lifesteal, speed, defense)
- **Curse/Blessing Altars** (floor 2+) -- risk/reward choice: pick a blessing (HP, mana, heal, crit, gold) or accept a curse for a powerful tradeoff (e.g. +3 ATK / -2 HP)
- **Minimap** -- bottom-right HUD indicator showing room progress with color-coded room types and current room highlight
- **Pause menu** (ESC) -- Resume, Save & Quit, Quit without Saving

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
 |- main.rs           App setup, GameState, RunData, save/load
 |- constants.rs      All numeric constants (physics, spells, layout)
 |- title.rs          Title screen, save slots, WellIntro cutscene
 |- player.rs         Movement, melee, dash, gamepad input
 |- room.rs           Room generation, 16 templates, decorations
 |- enemy.rs          7 enemy types + 4 unique bosses, elite variants, health bars
 |- combat.rs         Melee / spell / projectile collision & damage
 |- spell.rs          4 spells with trails, frost, bolts, shield ring
 |- pause_menu.rs     ESC pause menu with save & quit
 |- effects.rs        Particles, afterimages, dust, confetti, flash, damage numbers
 |- altar.rs          Curse/blessing altar choice system
 |- hud.rs            UI overlay (HP, mana, gold, floor, spells, minimap)
 |- camera.rs         Smooth follow camera + screen shake
 |- animation.rs      Generic frame-based animation support
 |- loot.rs           Drops, magnet pickup, treasure chest
 |- shop.rs           Cuphead-style shop UI, stone merchant NPC, overlay panel
 |- equipment.rs      20 items, 4 slots, 3 sets, stat calculation
 |- dialogue.rs       NPC dialogue with typewriter effect
 |- relic.rs          10 relics with passive run effects, post-boss choice UI
 |- hazards.rs        Lava, water, moving platforms, arrow traps, spikes, poison
 |- death_screen.rs   Game over screen with run stats
 |- floor_complete.rs Floor victory screen + confetti
 |- audio.rs          Audio system (stub, pending valid assets)
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
- [x] Screen transitions & fade effects between rooms
- [x] Save / Load for meta-progression (JSON file)
- [x] Game balance pass & difficulty scaling (floor-based enemy count/HP, shop prices, gold drops)
- [x] Spells locked at start -- buy with gold at shop
- [x] Enemy counter in HUD
- [x] Randomized room layouts per run (seed-based)
- [x] Advanced room decorations (lava, water, moving platforms)
- [x] NPC dialogue system with typewriter effect and floor-specific lore
- [x] Controller / gamepad support (full mapping across all screens)
- [x] 16 combat room templates with hazard integration
- [x] Save slot system (3 slots, per-floor checkpoints)
- [x] Pixel font (Press Start 2P) across all UI
- [x] Treasure chest auto-open on proximity
- [x] Death screen with run statistics
- [x] Floor complete screen with confetti celebration
- [ ] Spritesheet art (replace procedural rectangles)
- [ ] Audio engine (background music + SFX)
- [x] Equipment & set-bonus system (20 items, 4 slots, 3 sets with 2pc/3pc bonuses)
- [x] Economy rebalance with tiered item unlocks (3 tiers, milestone gating)
- [x] Platform visual polish (3-part caps, surface highlights, boss arena glow)
- [x] Stat upgrades in shop (Attack, Defense, Speed)
- [x] Purchase feedback (floating text)
- [x] Windows executable icon
- [x] Shop UI overhaul (Cuphead-style stone merchant NPC with overlay panel)
- [x] Block mechanic (K/RMB, 70% damage reduction, 3s cooldown, shield flash VFX)
- [x] Unique enemy abilities (Goblin leap, Bat dive bomb, Turret burst fire, Boar shockwave)
- [x] Tunnel room layout fix (open overhangs replacing sealed ceiling)
- [x] Pause menu (ESC -- Resume, Save & Quit, Quit without Saving)
- [x] Floating damage numbers (white/yellow crit/red player/blue block)
- [x] Enemy health bars (green/yellow/red based on HP ratio)
- [x] Boss phases (50% aggressive, 25% enraged with AoE slam shockwave)
- [x] Elite enemies (Armored, Swift, Brutal -- glowing aura, 2x HP, bonus loot)
- [x] Traps -- arrow traps (wall-mounted, periodic fire), spike floors (retractable), poison clouds (DOT area)
- [x] Relic choice after boss -- pick 1 of 3 random relics (10 relics with procedural pixel art icons)
- [x] Curse/Blessing altars -- risk/reward choice rooms (5 blessings, 5 curses)
- [x] 3 new enemy types -- Mage (teleport + cast), Slime (hop + split on death), Ghost (phase in/out)
- [x] 4 biomes with unique tile colors, backgrounds, and ambient decorations
- [x] 4 unique floor bosses -- Warlord, Mushroom King, Lava Golem, Root Guardian
- [x] Secret rooms with destructible walls (break with melee, reveal hidden loot)
- [x] Minimap HUD (room progress indicator with color-coded room types)

---

## License

MIT License -- see [LICENSE](./LICENSE).

---

<p align="center">
  <em>Enter the roots of dawn.</em>
</p>
