# Gunboot Descent - Quick Start Guide

## Project Status: READY TO RUN

All scene structure and configurations are properly set up. The project is ready to launch in Godot 4.5.

## How to Launch

### Method 1: From Godot Editor
1. Open Godot 4.5
2. Click **Import**
3. Navigate to: `C:\Spiele\downwell-clone`
4. Select `project.godot`
5. Click **Import & Edit**
6. Press **F5** or click the **Play** button

### Method 2: Command Line
```bash
godot --path "C:\Spiele\downwell-clone"
```

## What to Expect on Launch

### Console Output
```
Main scene initialized
GameRoot autoload is handling game initialization
```

### Game View
- **Background:** Dark blue/black color
- **Player:** Red character spawns at center
- **Level:** Platform chunks generate automatically
- **HUD:** Score, gems, health, and combo display in top-left
- **Controls:** Immediately responsive

## Controls

| Action | Keys |
|--------|------|
| Move Left | A or Left Arrow |
| Move Right | D or Right Arrow |
| Shoot (Gunboot) | Space, K, or Left Mouse Button |
| Pause/Menu | Escape |

## Game Mechanics

1. **Fall Down:** Player continuously falls (Downwell-style)
2. **Shoot Down:** Press shoot while in air to fire gunboots downward
3. **Stomp Enemies:** Land on enemies to defeat them and build combo
4. **Collect Gems:** Pick up gems to purchase shop upgrades
5. **Combo System:** Chain kills without touching ground for multiplier
6. **Shop Stops:** Every 5-7 chunks, encounter shop platforms for upgrades

## Scene Architecture Summary

```
Entry Flow:
┌─────────────────┐
│   main.tscn     │  ← Godot loads this first
│   (Entry Point) │
└─────────────────┘
         │
         │ (Autoload initializes in parallel)
         ▼
┌─────────────────┐
│   GameRoot      │  ← Singleton, created automatically
│  (game_root.gd) │
└─────────────────┘
         │
         │ _ready() calls _start_run()
         ▼
┌─────────────────┐
│   GameWorld     │  ← Core game container
│ (game_world.gd) │
└─────────────────┘
         │
         │ _build_world() creates:
         ▼
┌─────────────────────────────────────────┐
│ • Player (CharacterBody2D)              │
│ • Camera2D (follows player)             │
│ • LevelGenerator (chunks)               │
│ • GameHud (UI overlay)                  │
│ • ShopUI (upgrade interface)            │
│ • Audio systems                         │
│ • Effects (particles, screen shake)     │
└─────────────────────────────────────────┘
```

## File Locations

### Critical Files
- **Project Config:** `C:\Spiele\downwell-clone\project.godot`
- **Entry Scene:** `C:\Spiele\downwell-clone\scenes\main.tscn`
- **Autoload Singleton:** `C:\Spiele\downwell-clone\scripts\game_root.gd`
- **Core Game Logic:** `C:\Spiele\downwell-clone\scripts\game_world.gd`

### All Scripts
```
C:\Spiele\downwell-clone\scripts\
├── game_root.gd           # Autoload singleton (meta-game manager)
├── main.gd                # Entry point script
├── game_world.gd          # Core game logic
├── player.gd              # Player character
├── level_generator.gd     # Procedural level system
├── game_hud.gd            # UI overlay
├── shop_ui.gd             # Shop interface
├── game_constants.gd      # Shared utilities
├── chunk_templates.gd     # Level templates
├── gem.gd                 # Collectible gem
├── gunboot_bullet.gd      # Player projectile
├── destructible_block.gd  # Breakable blocks
├── hazard_spike.gd        # Spike hazards
└── enemies\
    ├── enemy_base.gd      # Base enemy class
    ├── ground_enemy.gd    # Walking enemy
    ├── flying_enemy.gd    # Flying enemy
    ├── heavy_enemy.gd     # Tank enemy
    ├── turret_enemy.gd    # Stationary shooter
    └── turret_projectile.gd # Enemy bullet
```

## Verification Checklist

Before launching, verify:

- [ ] Godot 4.5 or newer installed
- [ ] Project folder: `C:\Spiele\downwell-clone`
- [ ] `project.godot` exists and is readable
- [ ] `scenes\main.tscn` exists
- [ ] `scripts\game_root.gd` exists
- [ ] `scripts\game_world.gd` exists
- [ ] All enemy scripts in `scripts\enemies\` present

**All verified!** ✓

## Configuration Details

### Project Settings (project.godot)

#### Window Size
- **Viewport:** 480x840 pixels (portrait)
- **Stretch Mode:** viewport (pixel-perfect)

#### Autoload
- **GameRoot:** `*res://scripts/game_root.gd`
  - The `*` makes it a singleton accessible via `GameRoot`

#### Input Actions
- `move_left`: A, Left Arrow
- `move_right`: D, Right Arrow
- `shoot`: Space, K, Left Mouse Button
- `ui_cancel`: Escape

#### Rendering
- **Texture Filter:** Nearest (pixel art style)

## Troubleshooting

### Issue: "Failed to load resource" errors
**Cause:** Script paths incorrect or files missing
**Solution:** Verify all files exist at specified paths

### Issue: Black screen on launch
**Cause:** Main scene not loading or autoload not initializing
**Solution:**
1. Check console for errors
2. Verify `run/main_scene` in project.godot points to `res://scenes/main.tscn`
3. Ensure GameRoot autoload is configured with `*` prefix

### Issue: Game doesn't respond to input
**Cause:** Input actions not configured or player not initialized
**Solution:** Check that project.godot contains all input action definitions

### Issue: Parse errors on launch
**Cause:** GDScript syntax errors (Godot 4.x requires typed variables)
**Solution:** All syntax errors have been fixed; reimport project

## Next Steps

### To Add Sprites/Animations
1. Create assets in `assets/sprites/`
2. Import into Godot (will auto-generate .import files)
3. Reference in scripts via `ResourceLoader.load()`
4. Optional: Create .tscn animation scenes in `scenes/animations/`

### To Modify Game Balance
Edit constants in respective scripts:
- **Player mechanics:** `scripts/player.gd` (top constants)
- **Enemy stats:** `scripts/enemies/*.gd`
- **Level generation:** `scripts/chunk_templates.gd`
- **Game tuning:** `scripts/game_constants.gd`

### To Add New Enemy Types
1. Create new script in `scripts/enemies/`
2. Extend `EnemyBase` class
3. Override `_setup()`, `_ai_behavior()`, `take_damage()`
4. Add to spawn pool in `level_generator.gd`

### To Test Specific Systems
Run in editor (F5) and use console commands:
- `get_node("/root/GameRoot")` - Access singleton
- `GameRoot.world` - Access current GameWorld instance
- `GameRoot.world.player` - Access player object

## Documentation

For detailed architecture information, see:
- **SCENE_STRUCTURE.md** - Complete scene architecture documentation
- **README.md** - Project overview and features (if present)

## Support

Common questions answered in SCENE_STRUCTURE.md:
- Why script-based instead of .tscn scenes?
- How to convert to scene-based architecture
- How autoload singletons work
- Procedural generation approach

---

**Status:** Project configured and ready to run! 🎮

Press F5 in Godot to start playing.
