# Downroot Color Palette Reference

## Overview
This document provides all color values used in Downroot for easy customization and reference.

---

## Starting Island Palette

### Tree Colors (Pink/Orange Foliage)

```gdscript
# Light Pink - Used for brightest foliage areas
Color(1.0, 0.7, 0.6)    # RGB: 255, 178, 153

# Medium Pink - Main foliage color
Color(0.95, 0.55, 0.45) # RGB: 242, 140, 115

# Dark Pink - Shadow areas of pink trees
Color(0.85, 0.4, 0.35)  # RGB: 217, 102, 89

# Light Orange - Alternative tree color
Color(1.0, 0.6, 0.4)    # RGB: 255, 153, 102

# Dark Orange - Shadow areas of orange trees
Color(0.9, 0.45, 0.3)   # RGB: 230, 115, 77
```

### Sky Colors

```gdscript
# Sky Top - Deep blue at top of screen
Color(0.3, 0.45, 0.7)   # RGB: 77, 115, 179

# Sky Bottom - Lighter blue at horizon
Color(0.45, 0.6, 0.85)  # RGB: 115, 153, 217

# Cloud - Semi-transparent cloud color
Color(0.55, 0.65, 0.85, 0.6) # RGB: 140, 166, 217, Alpha: 0.6

# Star - Twinkling star color
Color(0.9, 0.95, 1.0, 0.8)   # RGB: 230, 242, 255, Alpha: 0.8
```

### Island Terrain Colors

```gdscript
# Grass Top Layer - Bright grass surface
Color(0.35, 0.6, 0.45)  # RGB: 89, 153, 115

# Grass Mid Layer - Darker grass layer
Color(0.3, 0.5, 0.4)    # RGB: 77, 128, 102

# Dirt Layer - Brown earth
Color(0.4, 0.35, 0.3)   # RGB: 102, 89, 77

# Rock Base - Dark stone foundation
Color(0.3, 0.25, 0.25)  # RGB: 77, 64, 64

# Tree Trunk - Purple-brown bark
Color(0.3, 0.2, 0.25)   # RGB: 77, 51, 64
```

### Decoration Colors

```gdscript
# Hanging Orbs - Warm glow
Color(1.0, 0.9, 0.6, 0.8) # RGB: 255, 230, 153, Alpha: 0.8

# Particle Glow - Landing effect
Color(1, 0.9, 0.7, 0.7)   # RGB: 255, 230, 179, Alpha: 0.7
```

---

## Existing Game Palette (from game_world.gd)

### DEFAULT Palette

```gdscript
"background": Color(0.05, 0.05, 0.08)    # RGB: 13, 13, 20
"platform":   Color(0.25, 0.22, 0.3)     # RGB: 64, 56, 77
"one_way":    Color(0.32, 0.28, 0.38)    # RGB: 82, 71, 97
"wall":       Color(0.1, 0.08, 0.12)     # RGB: 26, 20, 31
"block":      Color(0.35, 0.32, 0.4)     # RGB: 89, 82, 102
"hazard":     Color(0.9, 0.25, 0.3)      # RGB: 230, 64, 77
"gem":        Color(0.6, 1.0, 0.85)      # RGB: 153, 255, 217
"enemy":      Color(0.85, 0.3, 0.35)     # RGB: 217, 77, 89
"player":     Color(0.85, 0.15, 0.25)    # RGB: 217, 38, 64
"shop_text":  Color(1, 0.95, 0.85)       # RGB: 255, 242, 217
```

### SUNSET Palette (Unlockable)

```gdscript
"background": Color(0.12, 0.04, 0.08)    # RGB: 31, 10, 20
"platform":   Color(0.45, 0.2, 0.18)     # RGB: 115, 51, 46
"one_way":    Color(0.55, 0.28, 0.22)    # RGB: 140, 71, 56
"wall":       Color(0.2, 0.12, 0.1)      # RGB: 51, 31, 26
"block":      Color(0.55, 0.24, 0.2)     # RGB: 140, 61, 51
"hazard":     Color(1.0, 0.45, 0.3)      # RGB: 255, 115, 77
"gem":        Color(1.0, 0.8, 0.45)      # RGB: 255, 204, 115
"enemy":      Color(0.95, 0.4, 0.35)     # RGB: 242, 102, 89
"player":     Color(1.0, 0.5, 0.25)      # RGB: 255, 128, 64
"shop_text":  Color(1.0, 0.85, 0.7)      # RGB: 255, 217, 179
```

### MINT Palette (Unlockable)

```gdscript
"background": Color(0.02, 0.09, 0.08)    # RGB: 5, 23, 20
"platform":   Color(0.1, 0.25, 0.24)     # RGB: 26, 64, 61
"one_way":    Color(0.14, 0.32, 0.3)     # RGB: 36, 82, 77
"wall":       Color(0.06, 0.18, 0.16)    # RGB: 15, 46, 41
"block":      Color(0.12, 0.3, 0.28)     # RGB: 31, 77, 71
"hazard":     Color(0.75, 0.25, 0.35)    # RGB: 191, 64, 89
"gem":        Color(0.7, 1.0, 0.8)       # RGB: 179, 255, 204
"enemy":      Color(0.7, 0.2, 0.3)       # RGB: 179, 51, 77
"player":     Color(0.9, 0.3, 0.4)       # RGB: 230, 77, 102
"shop_text":  Color(0.8, 1.0, 0.9)       # RGB: 204, 255, 230
```

### NEON Palette (Unlockable)

```gdscript
"background": Color(0.02, 0.02, 0.08)    # RGB: 5, 5, 20
"platform":   Color(0.15, 0.08, 0.4)     # RGB: 38, 20, 102
"one_way":    Color(0.24, 0.16, 0.55)    # RGB: 61, 41, 140
"wall":       Color(0.08, 0.05, 0.25)    # RGB: 20, 13, 64
"block":      Color(0.2, 0.1, 0.45)      # RGB: 51, 26, 115
"hazard":     Color(1.0, 0.25, 0.7)      # RGB: 255, 64, 179
"gem":        Color(0.4, 1.0, 0.9)       # RGB: 102, 255, 230
"enemy":      Color(0.9, 0.2, 0.7)       # RGB: 230, 51, 179
"player":     Color(0.3, 0.9, 0.8)       # RGB: 77, 230, 204
"shop_text":  Color(0.7, 0.9, 1.0)       # RGB: 179, 230, 255
```

---

## Tutorial UI Colors

### Tutorial Prompt UI

```gdscript
# Background Panel
Color(0.1, 0.1, 0.15, 0.92)  # RGB: 26, 26, 38, Alpha: 0.92

# Icon Text (default white)
Color(1, 1, 1, 1)            # RGB: 255, 255, 255

# Prompt Text (default white)
Color(1, 1, 1, 1)            # RGB: 255, 255, 255
```

---

## Intro Cutscene Colors

### Landing Effects

```gdscript
# Dust Particles
Color(0.9, 0.8, 0.7, 0.7)    # RGB: 230, 204, 179, Alpha: 0.7

# Impact Wave
Color(1, 0.9, 0.7, 0.5)      # RGB: 255, 230, 179, Alpha: 0.5

# Fade Overlay
Color.BLACK                   # RGB: 0, 0, 0
```

---

## Customization Guide

### How to Change Starting Island Colors

Edit `scripts/starting_island.gd`:

```gdscript
# Find these constants at the top of the file:
const TREE_COLORS := {
    "pink_light": Color(1.0, 0.7, 0.6),     # Change this
    "pink_medium": Color(0.95, 0.55, 0.45), # Change this
    # ... etc
}

const SKY_COLORS := {
    "sky_top": Color(0.3, 0.45, 0.7),       # Change this
    # ... etc
}
```

### How to Change Tutorial UI Colors

Edit `scripts/tutorial_manager.gd`:

```gdscript
# Find _build_ui() method:
prompt_background.color = Color(0.1, 0.1, 0.15, 0.92) # Change this
```

### How to Create Custom Palette

Add to `game_world.gd` palette_data:

```gdscript
"MY_PALETTE": {
    "background": Color(0.X, 0.X, 0.X),
    "platform": Color(0.X, 0.X, 0.X),
    # ... etc (follow existing pattern)
}
```

---

## Color Theory Notes

### Starting Island Harmony
- **Pink/Orange trees**: Warm, welcoming, dreamy
- **Blue sky**: Cool, calming, creates contrast
- **Green terrain**: Natural, grounding

### Complementary Colors
- Pink trees + Blue sky = Complementary contrast
- Orange accents + Blue = Warm/cool balance
- Green terrain = Triadic harmony

### Suggested Variations

#### Autumn Theme:
```gdscript
"red_light": Color(0.9, 0.4, 0.3)
"red_dark": Color(0.7, 0.3, 0.2)
"yellow": Color(1.0, 0.8, 0.3)
```

#### Winter Theme:
```gdscript
"white_snow": Color(0.95, 0.95, 1.0)
"ice_blue": Color(0.7, 0.85, 1.0)
"frost": Color(0.8, 0.9, 0.95)
```

#### Night Theme:
```gdscript
"dark_blue": Color(0.1, 0.15, 0.3)
"purple_tree": Color(0.4, 0.3, 0.5)
"moon_glow": Color(0.9, 0.95, 1.0)
```

---

## RGB to Godot Color Conversion

### Quick Reference

```
Godot Color = (R/255, G/255, B/255, A)

Examples:
RGB(255, 128, 64) = Color(1.0, 0.502, 0.251)
RGB(100, 50, 200) = Color(0.392, 0.196, 0.784)
```

### Online Tools
- Godot Color Picker (built-in editor)
- colorizer.org
- color.adobe.com

---

## Accessibility Considerations

### Color Contrast Ratios

**Tutorial UI:**
- Background vs Text: High contrast (26,26,38 vs 255,255,255)
- WCAG AAA compliant
- Easily readable

**Starting Island:**
- Trees vs Sky: Good contrast for visibility
- Terrain layers: Distinguishable by color-blind users

### Color-Blind Friendly Adjustments

If needed, adjust to:
- Use brighter values
- Add more saturation differences
- Consider pattern/texture overlays

---

## Performance Notes

### Color Count Impact

- **Low Impact**: Using many colors (no texture memory)
- **Procedural Generation**: Colors generated at runtime
- **No Color Banding**: Gradients use smooth interpolation

### Optimization Tips

1. Reuse Color objects when possible
2. Avoid creating new Colors in _physics_process
3. Store frequently used colors as constants

---

## Color Modification Examples

### Make Island Brighter
```gdscript
# Multiply all color components by 1.2
Color(0.35, 0.6, 0.45) → Color(0.42, 0.72, 0.54)
```

### Add Color Variation
```gdscript
# Add random noise to each tree
var noise = randf_range(0.9, 1.1)
color.r *= noise
color.g *= noise
color.b *= noise
```

### Create Sunset Effect
```gdscript
# Shift towards warmer tones
color.r *= 1.3  # More red
color.g *= 1.1  # Slightly more green
color.b *= 0.7  # Less blue
```

---

## Testing Colors

### In Godot Editor
1. Create a ColorRect node
2. Set color in Inspector
3. Preview in 2D viewport

### In Code
```gdscript
# Quick color test
var test_sprite = Sprite2D.new()
test_sprite.texture = GameConstants.make_rect_texture(
    Color(1.0, 0.7, 0.6),  # Your color here
    Vector2i(100, 100)
)
add_child(test_sprite)
```

---

## Summary

All colors in Downroot are easily customizable through:
- Constants in starting_island.gd
- Dictionaries in game_world.gd
- Direct modifications in tutorial_manager.gd

The color palette creates a:
- Warm, welcoming atmosphere
- Clear visual hierarchy
- Accessible, readable UI
- Performance-optimized rendering

**Remember:** Colors are stored as float values (0.0 to 1.0), not integers (0 to 255).
