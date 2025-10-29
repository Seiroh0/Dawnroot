# UI/UX Enhancement Implementation Guide

**Step-by-step guide for implementing all HUD improvements**

---

## Current vs Enhanced HUD Comparison

### BEFORE (Current)
```
┌─────────────────────────────────────┐
│ ♥♥♥♥  ▬▬▬▬▬▬▬▬        💎 042       │
│              012345                 │
│                                     │
│                                     │
│                                     │
│               【 8 】               │ ← Combo only
│                                     │
└─────────────────────────────────────┘

Issues:
- Ammo bars too small (10x6px)
- No depth indicator
- No combo multiplier shown
- Score not emphasized
- Static elements (no animation feedback)
```

### AFTER (Enhanced)
```
┌─────────────────────────────────────┐
│ ♥♥♥♥  ●●●●●●●●        💎 042  🌟   │ ← Larger bullets, gem sparkle
│              【012345】              │ ← Score with brackets
│ ┃                                   │
│ ┃█              【 8 】              │ ← Combo number (large)
│ ┃██              x1.5               │ ← NEW: Multiplier shown
│ ┃███                                │
│ ┃████                               │
│ ┃████                               │
│ ┃█████                              │
│120m  ← Depth meter                  │
│                                     │
│          【 COMBO STOMP! 】         │ ← Animated messages
└─────────────────────────────────────┘

Improvements:
✓ Circular bullet icons (14x14px instead of 10x6)
✓ Vertical depth bar with meter display
✓ Combo multiplier display (x1.0, x1.5, x2.0)
✓ Score emphasized with brackets
✓ All elements animate on change
✓ Better visual hierarchy
```

---

## Implementation Steps

### Step 1: Enhanced Ammo Display (30 minutes)

**File:** `scripts/game_hud.gd`
**Lines to modify:** 129-158

**Before:**
```gdscript
func _build_ammo() -> void:
    _clear_container(ammo_container)
    ammo_nodes.clear()
    for i in range(ammo_capacity):
        var slot: ColorRect = ColorRect.new()
        slot.color = palette.get("muted", Color(0.2, 0.2, 0.25))
        slot.custom_minimum_size = Vector2(10, 6)  # Too small!
        slot.size_flags_vertical = Control.SIZE_SHRINK_CENTER
        ammo_container.add_child(slot)
        ammo_nodes.append(slot)
```

**After:**
```gdscript
func _build_ammo() -> void:
    _clear_container(ammo_container)
    ammo_nodes.clear()
    for i in range(ammo_capacity):
        var bullet_icon := TextureRect.new()
        bullet_icon.texture = _make_bullet_texture(false)
        bullet_icon.custom_minimum_size = Vector2(14, 14)  # Larger!
        bullet_icon.stretch_mode = TextureRect.STRETCH_KEEP
        bullet_icon.expand_mode = TextureRect.EXPAND_FIT_WIDTH_PROPORTIONAL
        ammo_container.add_child(bullet_icon)
        ammo_nodes.append(bullet_icon)

# NEW function to add:
func _make_bullet_texture(filled: bool) -> Texture2D:
    var size := 14
    var image: Image = Image.create(size, size, false, Image.FORMAT_RGBA8)
    var color: Color = palette.get("accent", Color(0.9, 0.15, 0.2)) if filled else palette.get("muted", Color(0.2, 0.2, 0.25))

    # Draw circular bullet
    for x in range(size):
        for y in range(size):
            var dx := x - size / 2.0
            var dy := y - size / 2.0
            var dist := sqrt(dx * dx + dy * dy)
            if dist < size / 2.5:
                image.set_pixel(x, y, color)
            elif dist < size / 2.5 + 1.0:
                image.set_pixel(x, y, Color(0, 0, 0, 0.8))
            else:
                image.set_pixel(x, y, Color(0, 0, 0, 0))

    return ImageTexture.create_from_image(image)

# Update this function:
func update_ammo(current: int, maximum: int) -> void:
    if maximum != ammo_capacity:
        ammo_capacity = clamp(maximum, 1, 16)
        _build_ammo()
    var filled: int = clamp(current, 0, ammo_capacity)
    for i in range(ammo_nodes.size()):
        var bullet_rect: TextureRect = ammo_nodes[i] as TextureRect
        if bullet_rect != null:
            bullet_rect.texture = _make_bullet_texture(i < filled)  # Use texture instead of color
    if current < maximum and maximum > 0:
        _flash_node(ammo_container)
```

**Test:** Run game, check ammo display is larger circular bullets that fill/empty correctly.

---

### Step 2: Add Combo Multiplier Display (30 minutes)

**File:** `scripts/game_hud.gd`
**Lines to add:** After line 108 (combo_label setup)

**New variable to declare at top:**
```gdscript
var multiplier_label: Label = null
```

**Add to _build_ui():**
```gdscript
# After combo_label setup (around line 108)
multiplier_label = _make_label("x1.0", 20)
multiplier_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
multiplier_label.modulate = Color(1, 1, 1, 0)
multiplier_label.set_anchors_preset(Control.PRESET_CENTER)
multiplier_label.offset_left = -60
multiplier_label.offset_right = 60
multiplier_label.offset_top = 20
multiplier_label.offset_bottom = 44
root.add_child(multiplier_label)
```

**Update update_combo():**
```gdscript
func update_combo(combo: int, multiplier: float) -> void:
    if combo <= 0:
        _hide_combo()
        return

    combo_label.text = "%d" % combo
    combo_label.modulate = Color(1, 1, 1, 1)

    # NEW: Show multiplier
    if multiplier_label != null:
        multiplier_label.text = "x%.1f" % multiplier
        multiplier_label.modulate = Color(1, 0.9, 0.4, 1)

        var mult_tween := create_tween()
        mult_tween.tween_property(multiplier_label, "scale", Vector2.ONE * 1.3, 0.1)
        mult_tween.tween_property(multiplier_label, "scale", Vector2.ONE, 0.15)

    # Existing combo animation...
    if combo_tween:
        combo_tween.kill()
    combo_tween = create_tween()
    combo_tween.tween_property(combo_label, "scale", Vector2.ONE * 1.4, 0.1).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
    combo_tween.tween_property(combo_label, "scale", Vector2.ONE, 0.15).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
```

**Update _hide_combo():**
```gdscript
func _hide_combo() -> void:
    if combo_tween:
        combo_tween.kill()
    combo_label.text = ""
    combo_label.scale = Vector2.ONE
    combo_label.modulate = Color(1, 1, 1, 0)

    # NEW: Hide multiplier
    if multiplier_label != null:
        multiplier_label.modulate = Color(1, 1, 1, 0)
```

**Test:** Stomp enemies, verify "x1.5" or "x2.0" appears below combo number.

---

### Step 3: Add Depth Meter (45 minutes)

**File:** `scripts/game_hud.gd`

**New variables to declare:**
```gdscript
var depth_bar: ProgressBar = null
var depth_label: Label = null
```

**Add to _build_ui():** (after top_left setup, before top_right)
```gdscript
# Depth meter (left side vertical bar)
var depth_margin := MarginContainer.new()
depth_margin.set_anchors_preset(Control.PRESET_LEFT_WIDE)
depth_margin.offset_left = 4
depth_margin.offset_right = 24
depth_margin.offset_top = 80
depth_margin.offset_bottom = -40
root.add_child(depth_margin)

var depth_vbox := VBoxContainer.new()
depth_margin.add_child(depth_vbox)

depth_label = _make_label("0m", 12)
depth_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
depth_vbox.add_child(depth_label)

depth_bar = ProgressBar.new()
depth_bar.orientation = ProgressBar.VERTICAL
depth_bar.fill_mode = ProgressBar.FILL_TOP_TO_BOTTOM
depth_bar.min_value = 0.0
depth_bar.max_value = 10000.0
depth_bar.value = 0.0
depth_bar.show_percentage = false
depth_bar.custom_minimum_size = Vector2(16, 200)

# Style the bar
var stylebox := StyleBoxFlat.new()
stylebox.bg_color = Color(0.1, 0.1, 0.15, 0.8)
stylebox.border_width_left = 1
stylebox.border_width_right = 1
stylebox.border_color = Color(0.3, 0.3, 0.4)
depth_bar.add_theme_stylebox_override("background", stylebox)

var fill_style := StyleBoxFlat.new()
fill_style.bg_color = palette.get("accent", Color(0.9, 0.15, 0.2))
depth_bar.add_theme_stylebox_override("fill", fill_style)

depth_vbox.add_child(depth_bar)
```

**Update update_depth():**
```gdscript
func update_depth(value: float) -> void:
    if depth_bar != null:
        depth_bar.value = value
        # Dynamically expand max as player goes deeper
        if value > depth_bar.max_value * 0.9:
            depth_bar.max_value = value * 1.2

    if depth_label != null:
        depth_label.text = "%dm" % int(value / 40.0)  # 40px = 1m
```

**Update _reapply_palette():**
```gdscript
func _reapply_palette() -> void:
    # ... existing code ...

    # NEW: Update depth bar color
    if depth_bar != null:
        var fill_style := StyleBoxFlat.new()
        fill_style.bg_color = palette.get("accent", Color(0.9, 0.15, 0.2))
        depth_bar.add_theme_stylebox_override("fill", fill_style)
```

**Test:** Run game, check left side shows vertical bar filling as you descend + meter in "meters".

---

### Step 4: Animated Gem Counter (20 minutes)

**File:** `scripts/game_hud.gd`

**New variable:**
```gdscript
var last_gem_count: int = 0
```

**Update update_score():**
```gdscript
func update_score(score: int, gems: int) -> void:
    score_label.text = "%06d" % score

    # NEW: Animate gem increase
    if gems > last_gem_count:
        _animate_gem_increase(gems)

    gem_label.text = "%03d" % gems
    last_gem_count = gems

# NEW function:
func _animate_gem_increase(new_count: int) -> void:
    if gem_icon == null or gem_label == null:
        return

    # Icon bounce
    var icon_tween := create_tween()
    icon_tween.tween_property(gem_icon, "scale", Vector2.ONE * 1.3, 0.08)
    icon_tween.tween_property(gem_icon, "scale", Vector2.ONE, 0.12).set_ease(Tween.EASE_BOUNCE)

    # Number pop
    var label_tween := create_tween()
    label_tween.tween_property(gem_label, "scale", Vector2.ONE * 1.2, 0.08)
    label_tween.tween_property(gem_label, "scale", Vector2.ONE, 0.12)

    # Color flash
    var flash_color := Color(1, 1, 0.6)
    var color_tween := create_tween()
    color_tween.tween_property(gem_label, "modulate", flash_color, 0.05)
    color_tween.tween_property(gem_label, "modulate", Color(1, 1, 1), 0.2)
```

**Test:** Collect gems, verify icon bounces and number flashes yellow.

---

### Step 5: Animated Heart Display (30 minutes)

**File:** `scripts/game_hud.gd`

**Update update_health():**
```gdscript
func update_health(value: int) -> void:
    var hearts: int = clamp(value, 0, MAX_HEARTS)
    var previous_health := 0

    # Count current filled hearts
    for i in range(heart_nodes.size()):
        var heart_rect := heart_nodes[i]
        if heart_rect != null and heart_rect.texture != null:
            # Check if texture is filled by comparing to a filled texture
            var test_filled := _make_heart_texture(true)
            if heart_rect.texture.get_width() == test_filled.get_width():
                previous_health += 1

    # Update textures
    for i in range(heart_nodes.size()):
        var heart_rect: TextureRect = heart_nodes[i]
        if heart_rect != null:
            var should_fill := i < hearts
            heart_rect.texture = _make_heart_texture(should_fill)

            # Animate change
            if previous_health > 0:
                if i == hearts - 1 and hearts > previous_health:
                    _animate_heart_gain(heart_rect)
                elif i == previous_health - 1 and hearts < previous_health:
                    _animate_heart_loss(heart_rect)

# NEW functions:
func _animate_heart_gain(heart: TextureRect) -> void:
    var tween := create_tween()
    heart.scale = Vector2.ZERO
    tween.tween_property(heart, "scale", Vector2.ONE * 1.3, 0.15).set_ease(Tween.EASE_OUT)
    tween.tween_property(heart, "scale", Vector2.ONE, 0.1).set_ease(Tween.EASE_IN)

func _animate_heart_loss(heart: TextureRect) -> void:
    var tween := create_tween()
    tween.tween_property(heart, "scale", Vector2.ONE * 1.4, 0.1)
    tween.tween_property(heart, "scale", Vector2.ONE, 0.15).set_ease(Tween.EASE_BOUNCE)

    # Flash red
    var flash_tween := create_tween()
    flash_tween.tween_property(heart, "modulate", Color(1, 0.3, 0.3), 0.1)
    flash_tween.tween_property(heart, "modulate", Color(1, 1, 1), 0.3)
```

**Test:** Take damage or heal, verify hearts animate in/out with bounce effect.

---

### Step 6: Score Emphasis with Brackets (10 minutes)

**File:** `scripts/game_hud.gd`

**Simple text formatting change in update_score():**
```gdscript
func update_score(score: int, gems: int) -> void:
    score_label.text = "【%06d】" % score  # Add brackets

    # ... rest of function ...
```

**Optional: Add score increase animation:**
```gdscript
func update_score(score: int, gems: int) -> void:
    var old_score := int(score_label.text.replace("【", "").replace("】", ""))
    score_label.text = "【%06d】" % score

    # NEW: Pulse on score increase
    if score > old_score:
        var tween := create_tween()
        tween.tween_property(score_label, "scale", Vector2.ONE * 1.1, 0.08)
        tween.tween_property(score_label, "scale", Vector2.ONE, 0.12)

    # ... gem animation code ...
```

**Test:** Score should show as 【012345】 and pulse when increasing.

---

## Visual Design Specifications

### Color Palette for UI Elements

```gdscript
# In game_hud.gd palette dictionary:
var palette: Dictionary = {
    "bg": Color(0.05, 0.05, 0.08, 0.8),        # Dark semi-transparent background
    "ui": Color(1, 1, 1),                       # White text
    "accent": Color(0.9, 0.15, 0.2),            # Red accent (hearts, ammo, depth)
    "muted": Color(0.2, 0.2, 0.25),             # Gray (empty states)
    "highlight": Color(1, 0.9, 0.4),            # Gold (combo multiplier, gem flash)
    "warning": Color(1, 0.4, 0.1),              # Orange (low health warning)
    "success": Color(0.4, 1, 0.6)               # Green (healing)
}
```

### Typography Specifications

```
Element          | Font Size | Alignment | Weight  | Outline
-----------------|-----------|-----------|---------|----------
Score            | 18pt      | Center    | Bold    | 1px black
Gem Count        | 16pt      | Right     | Normal  | 1px black
Combo Number     | 48pt      | Center    | Bold    | 2px black
Multiplier       | 20pt      | Center    | Normal  | 1px black
Depth Meter      | 12pt      | Center    | Normal  | 1px black
Message Banner   | 18pt      | Center    | Bold    | 1px black
```

### Spacing & Layout Grid

```
Grid: 8px baseline

Margins:
- Screen edge to UI: 8px
- UI element spacing: 4-6px
- Group separation: 12px

Sizes:
- Heart icon: 18x16px
- Bullet icon: 14x14px
- Gem icon: 14x14px
- Depth bar: 16x200px
```

---

## Testing Checklist

After implementing all steps:

- [ ] **Ammo bullets** are circular, 14x14px, clearly visible
- [ ] **Ammo bullets** fill/empty correctly on shoot/stomp
- [ ] **Combo number** displays large in center
- [ ] **Combo multiplier** shows below combo (x1.0, x1.5, x2.0)
- [ ] **Multiplier** scales up when combo increases
- [ ] **Depth bar** fills from top to bottom as player descends
- [ ] **Depth meter** shows correct "meters" based on pixel depth
- [ ] **Gem counter** bounces and flashes yellow on collection
- [ ] **Hearts** bounce in when gaining health
- [ ] **Hearts** flash red and shake when losing health
- [ ] **Score** displays with brackets 【】
- [ ] **Score** pulses slightly on increase (optional)
- [ ] All animations complete without errors
- [ ] No tween conflicts or jittering
- [ ] UI remains readable during intense gameplay
- [ ] Performance is smooth (60 FPS maintained)

---

## Troubleshooting

### Issue: Ammo bullets not showing
**Fix:** Check `_make_bullet_texture()` is creating valid ImageTexture. Add debug print to verify texture dimensions.

### Issue: Combo multiplier not appearing
**Fix:** Verify `multiplier_label` is initialized in `_ready()` and added to scene tree. Check modulate alpha is not 0.

### Issue: Depth bar not filling
**Fix:** Ensure `game_world.gd` is calling `hud.update_depth(depth)` every frame in `_physics_process()`.

### Issue: Gem animation stuttering
**Fix:** Kill previous tweens before starting new ones. Add tween conflict check.

### Issue: Hearts not animating on first health change
**Fix:** Initialize `previous_health` counter properly. Skip animation on first frame.

---

## Advanced Enhancements (Optional)

### Critical Health Pulsing Hearts
```gdscript
# In update_health(), add at end:
if hearts == 1:  # Critical health
    for heart in heart_nodes:
        if heart != null and heart.visible:
            _pulse_heart_critical(heart)

func _pulse_heart_critical(heart: TextureRect) -> void:
    var tween := create_tween()
    tween.set_loops()
    tween.tween_property(heart, "modulate", Color(1, 0.5, 0.5), 0.5)
    tween.tween_property(heart, "modulate", Color(1, 1, 1), 0.5)
```

### Combo Streak Message
```gdscript
# In update_combo(), add:
if combo == 10:
    show_message("COMBO x10!", 1.0)
elif combo == 20:
    show_message("INCREDIBLE!", 1.2)
elif combo == 50:
    show_message("UNSTOPPABLE!", 1.5)
```

### Rainbow Gem Counter at High Count
```gdscript
# In update_score(), add:
if gems >= 100:
    # Rainbow cycle effect
    var rainbow_tween := create_tween()
    rainbow_tween.set_loops()
    rainbow_tween.tween_property(gem_label, "modulate", Color(1, 0.5, 0.5), 1.0)
    rainbow_tween.tween_property(gem_label, "modulate", Color(1, 1, 0.5), 1.0)
    rainbow_tween.tween_property(gem_label, "modulate", Color(0.5, 1, 0.5), 1.0)
    rainbow_tween.tween_property(gem_label, "modulate", Color(0.5, 0.5, 1), 1.0)
    rainbow_tween.tween_property(gem_label, "modulate", Color(1, 0.5, 1), 1.0)
```

---

## Final Result Preview

After all enhancements, your HUD will:
- Show larger, clearer circular bullet indicators
- Display depth progression with visual bar
- Emphasize combo multiplier for score awareness
- Animate all value changes for better feedback
- Maintain clean pixel art aesthetic
- Feel responsive and "juicy"

**Estimated implementation time:** 2.5 - 3 hours for all steps

---

**Next Steps:** After UI is complete, move to visual effects integration (smoke sprites) for maximum game feel improvement!
