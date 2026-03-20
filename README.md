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
| Ranged Attack | `F` |
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
| Settings (title screen) | `S` |
| Toggle Fullscreen | `F11` |

---

## Features

<table>
<tr>
<td width="50%">

### Combat & Movement
- Tight platforming with **coyote time**, **jump buffering**, and **variable jump height**
- Melee sword with animated swing arc and **weapon sprite** from equipped item
- **Dash** with invincibility frames and afterimage trail
- **Ranged Attack** -- golden energy bolt projectile (F key)
- **Block** -- 70% damage reduction with 3s cooldown, shield flash effect
- **Traps** -- Arrow traps (wall-mounted, periodic fire), spike floors (retractable), poison clouds (DOT area)
- **4 Spells** -- Fireball (flame trail), Ice Shards (spread shot), Lightning (AoE bolts), Shield (rotating barrier)
- **Crit system** -- all damage types (melee, ranged, spells, lightning) can crit for 1.5x damage
- **Lifesteal** -- equipment/set bonus heals player on dealing damage
- Squash & stretch on jump/land, landing dust puffs

</td>
<td width="50%">

### Enemies & Bosses
- **Goblin** -- patrols and chases, animated legs, **leap attack**
- **Bat** -- wave movement with flapping wings, **dive bomb** with smooth ascending recovery
- **Stone Turret** -- aims and shoots, rotating eye, **burst fire**
- **Boar** -- detects and charges, horns tilt forward, **ground shockwave**
- **Mage** -- teleports when player approaches, casts purple fireballs, goes invisible
- **Slime** -- hops toward player, **splits into 2 small slimes on death**
- **Ghost** -- phases in/out of existence (intangible when phased), floats toward player
- **4 Unique Floor Bosses** -- Warlord (floor 1), Mushroom King (floor 2), Lava Golem (floor 3), Root Guardian (floor 4) -- each with distinct AI and animated tileset sprites
- **Boss phases** -- 50% HP aggressive, 25% HP enraged with AoE slam shockwave
- **Elite Enemies** -- rare glowing variants (Armored, Swift, Brutal) with 2x HP, bonus loot, pulsing aura
- **Floating damage numbers** on all hits (white, yellow crit, red player damage, blue block) with drop shadow, pop-scale animation, and ease-out movement
- **Enemy health bars** above each enemy (green / yellow / red)
- **Player health bar** -- dual-layer bar (instant fill + delayed trailing damage) with color shifts (green/yellow/red)

</td>
</tr>
<tr>
<td>

### World & Atmosphere
- **Title screen parallax** -- mouse-tracking depth layers on trees, hills, stars, moon, and well
- **Well glow & particles** -- pulsing magical glow above the well with golden sparkle particles floating upward
- **Well intro cutscene** -- animated player sprite descent with afterimage trail, speed lines, dust puffs on landing, and impact flash
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
- **Equipment system** -- 20 items across 4 slots (Weapon, Armor, Relic, Charm) with active stat application (attack%, defense%, max HP/MP, lifesteal, crit, speed)
- **Visible weapon sprites** -- equipped weapon shown on player character with swing animation on attack (16x16 weapon asset pack)
- **3 item sets** (Fire, Ice, Storm) with 2-piece and 3-piece bonuses -- actively calculated and applied
- **Stat upgrades** -- Attack, Defense, Speed purchasable in shop
- **Meta-progression** -- persistent upgrades between runs
- **Score system** with gold bonus from equipment
- Room-cleared confetti celebration
- **Relic choice after boss** -- pick 1 of 3 random relics (10 relics with procedural pixel art icons and active passive effects: crit, lifesteal per 5 kills, spell cooldown reduction, auto-block on room enter)
- **Curse/Blessing Altars** (floor 2+) -- risk/reward choice: pick a blessing (HP, mana, heal, crit, gold) or accept a curse for a powerful tradeoff (e.g. +3 ATK / -2 HP)
- **Minimap** -- bottom-right HUD indicator showing room progress with color-coded room types and current room highlight
- **Equipment HUD** -- 4-slot icon display (bottom-right) showing equipped Weapon, Armor, Relic, and Charm with Raven Fantasy Icons

</td>
</tr>
<tr>
<td>

### Audio
- **Background music** -- looping dungeon soundtrack with state-aware start/stop
- **Sound effects** -- melee hit (pitch variation), coin pickup, spell cast SFX (fireball, shield, lightning)
- **Volume system** -- Master, SFX, and Music volume controls with persistent settings
- **Safe audio loading** -- LoadState guards prevent crashes from unloaded assets

</td>
<td>

### UI & Settings
- **Spell icon bar** -- 4 spell slots with Raven Fantasy icon images, key number labels, and cooldown overlay (dark fill shrinking upward as cooldown drains)
- **Equipment icon HUD** -- 4-slot display with item-specific icons from Raven Fantasy Icons pack
- **Full settings menu** (Pause Menu > Settings, or `S` on title screen):
  - **Audio tab** -- Master/SFX/Music volume sliders (0-100%, live preview)
  - **Graphics tab** -- Fullscreen toggle (Windowed / Borderless Fullscreen)
  - **Controls tab** -- Read-only key binding reference table
- **Pause menu** (ESC) -- Resume, Settings, Save & Quit with `ResumingFromPause` state guard
- Settings persist to `dawnroot_settings.json`

</td>
</tr>
</table>

---

## Tech Stack

| | |
|:--|:--|
| **Engine** | [Bevy 0.15](https://bevyengine.org/) |
| **Language** | Rust (Edition 2024) |
| **Rendering** | Spritesheet-based characters (satiro-Sheet) + 0x72 Dungeon Tileset + animated spell effects |
| **Architecture** | ECS with state machine (Title, WellIntro, Playing, Paused, GameOver) |
| **Physics** | Custom AABB tile collision |
| **Audio** | Bevy audio with Symphonia backend (WAV, OGG, MP3) |
| **Dependencies** | `bevy 0.15`, `rand 0.8`, `serde 1`, `serde_json 1` |

---

## Asset Credits

| Asset | Author | License |
|:------|:-------|:--------|
| Player sprites | [satiro-Sheet](https://pixeldudesmaker.com/) | Pixeldudesmaker |
| Dungeon tileset | [0x72 DungeonTilesetII](https://0x72.itch.io/dungeontileset-ii) | CC0 |
| Enemy & boss sprites | 0x72 DungeonTilesetII | CC0 |
| Spell effects | Super Pixel Effects Gigapack (Free Version) | Free license |
| Weapon sprites | Weapons Asset 16x16 | Asset pack |
| UI icons | [Raven Fantasy Icons](https://clockworkraven.itch.io/) | Free version |
| Pixel font | [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) | OFL |
| SFX: Sword hit | CogfireStudios (Freesound.org) | CC0 |
| SFX: Coin pickup | DavidSraba (Freesound.org) | CC0 |
| SFX: Thunder strike | AyaDrevis (Freesound.org) | CC0 |
| SFX: Shield | Metzik (Freesound.org) | CC0 |
| BGM: Dungeon of Fate | Music asset | Free license |

---

## Project Structure

```
src/
 |- main.rs           App setup, GameState, RunData, save/load, settings persistence
 |- constants.rs      All numeric constants (physics, spells, layout)
 |- title.rs          Title screen, save slots, WellIntro cutscene, settings access
 |- player.rs         Movement, melee, dash, weapon sprites, spritesheet animation
 |- room.rs           Room generation, 16 templates, tileset rendering, decorations
 |- enemy.rs          7 enemy types + 4 unique bosses, tileset sprites, elite variants
 |- combat.rs         Melee / spell / projectile collision & damage
 |- spell.rs          4 spells with animated effect sprites, trails, frost, bolts
 |- pause_menu.rs     ESC pause menu, full settings UI (audio/graphics/controls)
 |- effects.rs        Particles, afterimages, dust, confetti, flash, damage numbers
 |- altar.rs          Curse/blessing altar choice system
 |- hud.rs            UI overlay (HP, mana, gold, floor, spell icons, equipment, minimap)
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
 |- audio.rs          Audio system with SFX, BGM, volume settings, safe loading
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
- [x] Spritesheet art (replace procedural rectangles with satiro-Sheet + 0x72 tileset)
- [x] Audio engine (BGM + SFX with volume controls and safe loading)
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
- [x] Pause menu (ESC -- Resume, Settings, Save & Quit)
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
- [x] Title screen parallax (mouse-tracking depth layers on all scene elements)
- [x] Well glow & golden particle emitter on title screen
- [x] Well intro juice (afterimage trail, enhanced speed lines, dust puffs, impact flash)
- [x] Bat smooth dive recovery (ascending state with lerp deceleration)
- [x] Player health bar overhaul (dual-layer bar with delayed damage trailing effect)
- [x] Damage numbers polish (larger font, drop shadow, pop-scale animation, ease-out)
- [x] Shop room visual upgrade (warm palette, carpet, crates, shelves, lanterns)
- [x] Pause/resume state safety (ResumingFromPause resource guards all OnEnter systems)
- [x] Block mechanic fix (resolved input conflict with ranged attack)
- [x] Shop overlay input blocking (movement + spells disabled during shop UI)
- [x] Player spritesheet integration (satiro-Sheet, idle/run animation with frame cycling)
- [x] Dungeon tileset integration (0x72 DungeonTilesetII for floor/wall/ceiling tiles)
- [x] Enemy sprite integration (7 types + 4 bosses using tileset animated sprites)
- [x] Spell effect sprites (fireball, ice, lightning from Super Pixel Effects Gigapack)
- [x] Audio system with SFX events, BGM looping, pitch variation
- [x] Weapon sprite system (equipped weapon visible on player with swing animation)
- [x] Equipment HUD (4-slot icon display with Raven Fantasy Icons)
- [x] Spell icon bar (icon-based spell slots with cooldown overlay)
- [x] Full settings menu (Audio sliders, Graphics toggle, Controls reference)
- [x] Volume system (Master/SFX/Music with persistent settings)
- [ ] Key rebinding UI
- [ ] Localization (German/English)

---

## License

MIT License -- see [LICENSE](./LICENSE).

---

<p align="center">
  <em>Enter the roots of dawn.</em>
</p>
