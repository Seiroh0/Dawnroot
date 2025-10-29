# Gunboot Descent - Scene Structure Documentation

## Overview

This Downwell clone uses a **script-based architecture** where most game objects are created programmatically rather than using traditional .tscn scene files. This approach provides maximum flexibility and follows a common pattern for procedurally generated games.

## Architecture Hierarchy

```
Main Scene (main.tscn)
    └── Main (Node2D) [main.gd]
        └── Entry point only, minimal logic

GameRoot (Autoload Singleton) [game_root.gd]
    └── GameManager class
        └── GameWorld (instantiated programmatically)
            ├── Player (CharacterBody2D)
            ├── Camera2D
            ├── LevelGenerator
            │   └── Procedurally generated chunks
            ├── GameHud (UI overlay)
            ├── ShopUI (Shop system)
            └── Various game systems
```

## Key Components

### 1. Entry Point: main.tscn

**Location:** `C:\Spiele\downwell-clone\scenes\main.tscn`

- Minimal scene file serving as the project entry point
- Attached script: `main.gd` (mostly informational)
- Configured in `project.godot` as `run/main_scene`

**Purpose:**
- Godot requires a main scene to launch
- This scene loads immediately when the game starts
- The actual game logic is handled by the GameRoot autoload

### 2. Autoload Singleton: GameRoot

**Location:** `C:\Spiele\downwell-clone\scripts\game_root.gd`

- Configured in `project.godot` under `[autoload]` section
- Automatically instantiated before any scene loads
- Manages game lifecycle and meta-progression

**Responsibilities:**
- Instantiates and manages GameWorld instances
- Handles run restarts and game-over logic
- Manages persistent data (high scores, unlocks, save/load)
- Coordinates between runs

### 3. Core Game Logic: GameWorld

**Location:** `C:\Spiele\downwell-clone\scripts\game_world.gd`

- Instantiated by GameRoot in `_ready()`
- Main container for all game systems and entities

**Responsibilities:**
- Creates player, camera, level generator
- Manages game state (score, gems, depth, combo)
- Handles procedural level generation
- Coordinates HUD updates
- Manages shop system
- Controls audio and visual effects
- Implements screen shake and time freeze mechanics

**Child Systems Created:**
- **Player**: Main character with gunboot mechanics
- **LevelGenerator**: Procedural chunk-based level creation
- **GameHud**: Score, health, ammo, combo display
- **ShopUI**: Upgrade purchase interface
- **Camera2D**: Smooth-following camera with screen shake
- **Audio Systems**: Procedural sound effects and background music

## Scene Setup Process

When the game launches:

1. **Godot loads:** `main.tscn` (entry point)
2. **Autoload initializes:** `GameRoot` singleton is created
3. **GameRoot._ready()** calls `_start_run()`
4. **GameWorld instance** is created and added as child to GameRoot
5. **GameWorld._build_world()** creates all game nodes programmatically:
   - Background (CanvasLayer + ColorRect)
   - Level root node hierarchy
   - Player (CharacterBody2D with collision shapes)
   - Camera2D with limits
   - LevelGenerator (procedural chunk system)
   - GameHud (UI elements)
   - ShopUI (upgrade interface)
   - Audio players
6. **Game starts** with player spawned and level generating

## Why Script-Based Architecture?

This project intentionally avoids creating .tscn files for every component because:

1. **Procedural Generation**: Levels are generated at runtime
2. **Dynamic Configuration**: Systems adjust based on game state
3. **Flexibility**: Easy to modify without scene file overhead
4. **Programmatic Control**: Direct access to all properties during creation
5. **Performance**: Efficient instantiation for many dynamic objects

## File Structure

```
C:\Spiele\downwell-clone\
├── project.godot              # Main project configuration
├── icon.svg                   # Project icon
├── scenes\
│   └── main.tscn             # Entry point scene (minimal)
├── scripts\
│   ├── game_root.gd          # Autoload singleton (GameManager)
│   ├── main.gd               # Entry point script
│   ├── game_world.gd         # Core game logic (GameWorld)
│   ├── player.gd             # Player character
│   ├── level_generator.gd    # Procedural level system
│   ├── game_hud.gd           # UI overlay
│   ├── shop_ui.gd            # Shop system
│   ├── game_constants.gd     # Shared constants and utilities
│   ├── chunk_templates.gd    # Level chunk definitions
│   ├── gem.gd                # Collectible gem
│   ├── gunboot_bullet.gd     # Player projectile
│   ├── destructible_block.gd # Breakable blocks
│   ├── hazard_spike.gd       # Spike hazards
│   └── enemies\
│       ├── enemy_base.gd     # Base enemy class
│       ├── ground_enemy.gd   # Walking enemy
│       ├── flying_enemy.gd   # Flying enemy
│       ├── heavy_enemy.gd    # Tank enemy
│       ├── turret_enemy.gd   # Stationary shooter
│       └── turret_projectile.gd # Enemy bullet
└── assets\
    ├── audio\                # Sound effects and music
    └── sprites\              # Visual assets
```

## Project Configuration (project.godot)

### Application Settings
- **Name:** "Gunboot Descent"
- **Main Scene:** `res://scenes/main.tscn`
- **Features:** Godot 4.5
- **Icon:** `res://icon.svg`

### Display Settings
- **Viewport Size:** 480x840 (portrait orientation, Downwell-style)
- **Stretch Mode:** viewport (pixel-perfect scaling)

### Rendering Settings
- **Texture Filter:** Nearest neighbor (pixel art)

### Autoload
- **GameRoot:** `*res://scripts/game_root.gd` (GameManager singleton)

### Input Actions
- **move_left:** A, Left Arrow
- **move_right:** D, Right Arrow
- **shoot:** Space, K, Left Mouse Button
- **ui_cancel:** Escape

## Animation Setup Note

Currently, all visual elements are created programmatically using:
- `AnimatedSprite2D` nodes with runtime-generated `SpriteFrames`
- `ColorRect` for placeholder graphics
- Procedural textures via `GameConstants.make_rect_texture()`

**To add 2D animation scenes:**

If you want to create proper .tscn animation scenes in Godot editor:

1. Create animation scenes in `scenes/animations/`
2. Reference them in respective scripts via preload:
   ```gdscript
   var player_anim_scene = preload("res://scenes/animations/player_anim.tscn")
   ```
3. Instantiate and attach to game objects during `_build_world()`

**Current approach** uses dynamic sprite frame creation, which is perfectly valid for:
- Rapid prototyping
- Placeholder graphics
- Procedural visual effects
- Runtime-generated animations

## Testing the Scene Structure

To verify the setup works:

1. Open project in Godot 4.5: `File > Open Project` → Select `project.godot`
2. Press **F5** or click **Play** button
3. Check output console for:
   ```
   Main scene initialized
   GameRoot autoload is handling game initialization
   ```
4. Game should start with player spawning and level generating

## Common Issues and Solutions

### Issue: "Failed to load script" errors
**Solution:** Verify all script paths in `project.godot` use forward slashes and correct `res://` prefix

### Issue: Game doesn't start
**Solution:** Check that `GameRoot` autoload is configured with asterisk (*) prefix for singleton

### Issue: Black screen on launch
**Solution:** Ensure `main.tscn` exists and is set as main scene in project settings

### Issue: Parse errors
**Solution:** All scripts must have proper GDScript syntax with typed variables for Godot 4.x

## Advanced: Converting to Scene-Based Architecture

If you want to convert this to use .tscn files:

1. **Create Player Scene:**
   - Create `player.tscn` with CharacterBody2D root
   - Add collision shapes and AnimatedSprite2D in editor
   - Attach `player.gd` script
   - In `game_world.gd`, change:
     ```gdscript
     var player_scene = preload("res://scenes/player.tscn")
     player = player_scene.instantiate()
     ```

2. **Create Enemy Scenes:**
   - Similar process for each enemy type
   - Set up animations in SpriteFrames editor

3. **Maintain Autoload Structure:**
   - Keep GameRoot as autoload
   - Keep procedural level generation
   - Only convert entities that benefit from visual editing

## Summary

The project is fully configured and ready to run. The scene structure is intentionally minimal with main.tscn as the entry point and GameRoot autoload handling all game initialization. This architecture provides maximum flexibility for the procedural, Downwell-inspired gameplay while maintaining clean separation of concerns.

**Key Takeaway:** This is a script-first architecture where scenes serve as entry points, not containers. The game "builds itself" at runtime through programmatic node creation.
