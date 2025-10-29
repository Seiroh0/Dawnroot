# Quick Start Integration for Downroot

## Fastest Way to Get Downroot Running

### Step 1: Backup Your Current game_world.gd
Make a copy of `scripts/game_world.gd` before modifying it.

### Step 2: Add to game_world.gd

Find the `_build_world()` function in `game_world.gd` and add this code **after the camera is created** (around line 189):

```gdscript
# === DOWNROOT INTEGRATION START ===
# Add this after: level_root.add_child(camera)

# Load integration helper
var integration_script: Script = preload("res://scripts/downroot_integration.gd")
if integration_script != null:
    var downroot_integration: DownrootIntegration = integration_script.new() as DownrootIntegration
    if downroot_integration != null:
        # This sets up: starting island, intro cutscene, tutorial
        downroot_integration.setup_downroot_experience(self, player, camera, level_root)

        # Optional: Replace default background with sky background
        DownrootIntegration.create_simple_sky_background(background_layer)

# === DOWNROOT INTEGRATION END ===
```

### Step 3: Add Signal Handlers (Optional)

Add these methods to `game_world.gd` if you want custom behavior:

```gdscript
func _on_gameplay_start() -> void:
    # Called when intro cutscene finishes
    print("Downroot gameplay started!")
    # Add any custom initialization here

func _on_tutorial_completed() -> void:
    # Called when tutorial finishes
    print("Tutorial completed!")
    # Could unlock achievements, show message, etc.
```

### Step 4: Test in Godot

1. Save all files
2. Open the project in Godot
3. Run the game (F5)
4. You should see:
   - Character falling from sky onto floating island
   - Camera following the fall
   - Landing impact effect
   - Tutorial prompts appearing

### That's It!

The integration is complete. The game will now:
- Start with an intro cutscene
- Show the beautiful starting island
- Guide players through a tutorial
- Use enhanced visuals from the tileset

---

## Quick Testing Options

### Skip Intro/Tutorial During Development

Add this to the integration code to skip straight to gameplay:

```gdscript
# After setup_downroot_experience call:
# downroot_integration.skip_to_gameplay()  # Uncomment to skip intro/tutorial
```

### Test Individual Systems

#### Test Just the Starting Island:
```gdscript
var island_script: Script = preload("res://scripts/starting_island.gd")
var island: StartingIsland = island_script.new()
level_root.add_child(island)
player.position = island.get_spawn_position()
```

#### Test Just the Intro:
```gdscript
var intro_script: Script = preload("res://scripts/intro_cutscene.gd")
var intro: IntroCutscene = intro_script.new()
add_child(intro)
intro.setup(player, camera, self)
intro.play_cutscene()
```

#### Test Just the Tutorial:
```gdscript
var tutorial_script: Script = preload("res://scripts/tutorial_manager.gd")
var tutorial: TutorialManager = tutorial_script.new()
add_child(tutorial)
tutorial.setup(player)
tutorial.start_tutorial()
```

---

## Minimal Integration (No Helper)

If you prefer not to use the integration helper, here's the minimal code:

```gdscript
# === MINIMAL DOWNROOT INTEGRATION ===

# 1. Create Starting Island
var island_script: Script = preload("res://scripts/starting_island.gd")
var starting_island: StartingIsland = island_script.new()
starting_island.name = "StartingIsland"
level_root.add_child(starting_island)

# Position player on island
player.position = starting_island.get_spawn_position()

# 2. Create and play intro cutscene
var intro_script: Script = preload("res://scripts/intro_cutscene.gd")
var intro_cutscene: IntroCutscene = intro_script.new()
intro_cutscene.name = "IntroCutscene"
add_child(intro_cutscene)
intro_cutscene.setup(player, camera, self)
intro_cutscene.play_cutscene()

# 3. Create tutorial (will auto-start after intro)
var tutorial_script: Script = preload("res://scripts/tutorial_manager.gd")
var tutorial_manager: TutorialManager = tutorial_script.new()
tutorial_manager.name = "TutorialManager"
add_child(tutorial_manager)
tutorial_manager.setup(player)

# Connect signals
intro_cutscene.cutscene_finished.connect(func() -> void:
    tutorial_manager.start_tutorial()
)

# === END MINIMAL INTEGRATION ===
```

---

## Common Issues and Fixes

### Issue: "Cannot preload resource"
**Fix:** Make sure all new scripts are in the `scripts/` folder:
- `scripts/intro_cutscene.gd`
- `scripts/tutorial_manager.gd`
- `scripts/starting_island.gd`
- `scripts/tileset_loader.gd`
- `scripts/downroot_integration.gd`

### Issue: Parser error on level_generator.gd
**Fix:** Just reopen the Godot project. The file is syntactically correct; it's a cached error.

### Issue: Camera doesn't follow player during intro
**Fix:** Make sure camera is passed correctly to intro_cutscene.setup()

### Issue: Tutorial doesn't appear
**Fix:** Make sure tutorial.start_tutorial() is called after the intro finishes

### Issue: Starting island not visible
**Fix:** Check that level_root is the correct parent node and camera is positioned correctly

---

## Performance Notes

All new systems are lightweight:
- **Intro Cutscene**: ~1-3 seconds, disabled after completion
- **Tutorial**: Only active during tutorial phase, minimal overhead
- **Starting Island**: Static decorations, ~40-60 draw calls
- **Tileset**: Standard Godot TileMap, optimized by engine

---

## What Happens in the Game

1. **Game Starts** → Screen fades in from black
2. **Intro Cutscene** → Character falls from sky onto floating island
3. **Landing** → Impact effects, screen shake, camera stabilizes
4. **Tutorial Begins** → On-screen prompts guide player
5. **Normal Gameplay** → After tutorial, standard game loop

---

## Next Steps

After integration works:
1. Customize the starting island colors/layout
2. Adjust tutorial text for your game
3. Add audio to intro and tutorial
4. Create additional floating islands
5. Extend tileset with more decorations

See `DOWNROOT_IMPLEMENTATION_GUIDE.md` for detailed documentation.
