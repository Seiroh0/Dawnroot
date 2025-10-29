# Visual Effects Quick Reference Guide

**Quick lookup table for implementing smoke effects in Downroot**

---

## Smoke Asset Usage Table

| Effect Name | Asset File | Frames | FPS | Scale | Duration | Color | Implementation File |
|-------------|-----------|--------|-----|-------|----------|-------|-------------------|
| **Enemy Death Burst** | Smoke 05 | 11 | 18 | 0.5x | 0.61s | Enemy accent color | `scripts/enemies/enemy_base.gd` line 97 |
| **Gunboot Muzzle Trail** | Smoke 06 | 6 | 24 | 0.35x | 0.25s | Orange (1, 0.7, 0.4) | `scripts/player.gd` line 372 |
| **Bullet Impact** | Smoke 07 | 5 | 30 | 0.3x | 0.17s | Yellow-white (1, 0.9, 0.6) | `scripts/gunboot_bullet.gd` (new method) |
| **Player Damage Hit** | Smoke 05 | 8 | 20 | 0.6x | 0.4s | Red (1, 0.3, 0.3) | `scripts/player.gd` line 326 |
| **Stomp Ground Blast** | Smoke 07 | 10 | 22 | 0.7x0.5 | 0.45s | Golden (1, 0.8, 0.4) | `scripts/player.gd` line 401 |
| **Block Destruction** | Smoke 06 | 9 | 16 | 0.8x | 0.56s | Block color | `scripts/destructible_block.gd` |

---

## Animation FPS Changes

| Entity | Animation | Current FPS | New FPS | Reasoning |
|--------|-----------|-------------|---------|-----------|
| Player | Jump | 12 | **14** | More responsive feel |
| Player | Stomp | 14 (2 frames) | **10 (3 frames)** | Increase visual duration from 0.14s to 0.3s |
| Player | Hit | 12 | **16** | Sharper damage feedback |

---

## Screen Shake Values

| Event | Strength | Duration | Implementation |
|-------|----------|----------|----------------|
| Player Stomp | 12.0 | 0.2s | Already implemented |
| Player Damage | 14.0 | 0.3s | Already implemented |
| Block Break | **8.0** | **0.15s** | ADD to `_on_block_broken()` |
| Enemy Defeat | **6.0** | **0.1s** | ADD to `_on_enemy_defeated()` |
| Landing Impact | **velocity-based** | **0.12s** | ADD to player `_on_landed()` |

---

## Particle System Specs

### Gunboot Bullet Trail
```
Amount: 8 particles
Lifetime: 0.2s
Gravity: (0, -50) - slight upward drift
Initial Velocity: 80.0
Spread: 45°
Color: Gradient from yellow (1, 0.9, 0.6) to orange (1, 0.6, 0.3) to transparent
```

### Enemy Death Burst
```
Amount: 32 particles
Lifetime: 0.6s
Explosiveness: 0.9
Gravity: (0, 400)
Initial Velocity: 220.0
Spread: 180°
Color: Enemy primary palette color
```

### Stomp Shockwave Ring
```
Amount: 16 particles
Lifetime: 0.3s
Emission Shape: Ring (radius 8, inner 6)
Initial Velocity: 300.0
Linear Accel: -400.0 (decelerate quickly)
Color: Golden (1, 0.9, 0.5)
```

### Gem Collection Sparkle
```
Amount: 24 particles (already implemented)
Lifetime: 0.4s
Gravity: (0, 220)
Initial Velocity: 160.0
Spread: 120°
Color: Gem palette color
```

---

## Code Snippets for Common Tasks

### Extract Frames from Smoke Spritesheet
```gdscript
# For 64x64 frame grid
var smoke_sheet: Texture2D = preload("res://assets/effects/Free Smoke Fx  Pixel 05.png")
const FRAME_SIZE := 64

for i in range(frame_count):
    var atlas := AtlasTexture.new()
    atlas.atlas = smoke_sheet
    atlas.region = Rect2(i * FRAME_SIZE, 0, FRAME_SIZE, FRAME_SIZE)
    frames.add_frame("animation_name", atlas)
```

### Standard Smoke Effect Setup
```gdscript
var smoke := AnimatedSprite2D.new()
var frames := SpriteFrames.new()
frames.add_animation("effect")
frames.set_animation_loop("effect", false)
frames.set_animation_speed("effect", 18.0)  # Adjust FPS here

# Add frames...

smoke.sprite_frames = frames
smoke.animation = "effect"
smoke.global_position = effect_position
smoke.scale = Vector2(0.5, 0.5)  # Adjust scale here
smoke.modulate = Color(1, 1, 1, 1)  # Adjust color here
parent.add_child(smoke)
smoke.play("effect")

# Auto-cleanup
smoke.animation_finished.connect(func(_anim: StringName) -> void:
    smoke.queue_free()
)
```

### Add Screen Shake
```gdscript
# In game_world.gd or accessible from there
_start_screen_shake(strength, duration)

# Examples:
_start_screen_shake(8.0, 0.15)   # Light shake (blocks)
_start_screen_shake(12.0, 0.2)   # Medium shake (stomp)
_start_screen_shake(14.0, 0.3)   # Heavy shake (damage)
```

### Camera Zoom Effect
```gdscript
func _trigger_camera_zoom(zoom_amount: float, duration: float) -> void:
    if camera == null:
        return
    var start_zoom := camera.zoom
    var target_zoom := start_zoom * zoom_amount
    var tween := create_tween()
    tween.tween_property(camera, "zoom", target_zoom, duration * 0.3)
    tween.tween_property(camera, "zoom", start_zoom, duration * 0.7).set_ease(Tween.EASE_OUT)

# Usage:
_trigger_camera_zoom(1.15, 0.2)  # 15% zoom for 0.2 seconds
```

---

## New Chunk Templates to Add

### Bonus Vault (High Reward)
```gdscript
{
    "name": "bonus_vault",
    "variation": "reward",
    "difficulty": 1,
    "solids": [
        {"start": Vector2i(0, 2), "end": Vector2i(0, 14)},
        {"start": Vector2i(8, 2), "end": Vector2i(8, 14)},
        {"start": Vector2i(0, 14), "end": Vector2i(8, 14)}
    ],
    "destructibles": [
        {"cell": Vector2i(3, 7), "size": Vector2i(3, 2)}
    ],
    "gems": [
        {"cell": Vector2i(3, 8)}, {"cell": Vector2i(4, 8)}, {"cell": Vector2i(5, 8)}
    ]
}
```

### Vertical Gauntlet (Skill Challenge)
```gdscript
{
    "name": "vertical_gauntlet",
    "variation": "combat",
    "difficulty": 2,
    "solids": [
        {"start": Vector2i(0, 2), "end": Vector2i(1, 5)},
        {"start": Vector2i(7, 2), "end": Vector2i(8, 5)}
    ],
    "enemies": [
        {"type": "flying", "cell": Vector2i(4, 3), "amp": 50},
        {"type": "flying", "cell": Vector2i(4, 6), "amp": 60},
        {"type": "flying", "cell": Vector2i(4, 10), "amp": 40}
    ]
}
```

---

## File Paths Reference

```
Project Root: C:\Spiele\downwell-clone\

Smoke Effects:
  - assets/effects/Free Smoke Fx  Pixel 05.png
  - assets/effects/Free Smoke Fx  Pixel 06.png
  - assets/effects/Free Smoke Fx  Pixel 07.png

Scripts to Modify:
  - scripts/player.gd (muzzle flash, damage hit, stomp)
  - scripts/enemies/enemy_base.gd (death effect)
  - scripts/gunboot_bullet.gd (bullet impact)
  - scripts/destructible_block.gd (block break)
  - scripts/game_world.gd (screen shake, camera effects)
  - scripts/game_hud.gd (UI enhancements)
  - scripts/chunk_templates.gd (new templates)
  - scripts/level_generator.gd (dynamic scaling)
```

---

## Testing Checklist

After implementing effects, test:
- [ ] Enemy death shows smoke burst in correct palette color
- [ ] Gunboot firing shows orange trail downward
- [ ] Bullets create small impact puff on hit
- [ ] Player damage shows red smoke ring
- [ ] Stomp creates golden ground explosion
- [ ] Block destruction shows colored debris cloud
- [ ] All effects auto-cleanup (no orphaned nodes)
- [ ] Performance is stable (60 FPS maintained)
- [ ] Effects scale correctly with pixel art aesthetic
- [ ] Colors match game palette

---

## Performance Budget

**Maximum concurrent effects:** ~20-30 active sprite animations
**Per-frame particle systems:** Max 3-4 CPUParticles2D active
**Tween count:** Keep under 10 active tweens
**Texture memory:** 3 smoke spritesheets = ~3MB total

All smoke effects reuse same 3 texture assets - efficient!

---

**Quick Start:** Begin with Enemy Death Burst (Smoke 05) - easiest to implement and highest visual impact.
