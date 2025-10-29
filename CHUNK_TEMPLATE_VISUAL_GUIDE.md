# Chunk Template Visual Design Guide

**Understanding Downroot's level generation system**

---

## Grid System Basics

Each chunk is a 9x16 grid (360x640 pixels):
- **Columns:** 0-8 (9 total, 40px each)
- **Rows:** 0-15 (16 total, 40px each)
- **Cell:** Vector2i(column, row)
- **Cell Center:** (column * 40 + 20, row * 40 + 20) pixels

```
Column:  0   1   2   3   4   5   6   7   8
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
Row 0    │   │   │   │   │   │   │   │   │   │
         ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
Row 1    │   │   │   │   │   │   │   │   │   │
         ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
Row 2    │   │   │   │   │   │   │   │   │   │
         ├───┼───┼───┼───┼───┼───┼───┼───┼───┤
Row 3    │   │   │   │   │   │   │   │   │   │
         .   .   .   .   .   .   .   .   .
         .   .   .   .   .   .   .   .   .   .
Row 15   │   │   │   │   │   │   │   │   │   │
         └───┴───┴───┴───┴───┴───┴───┴───┴───┘

         Each cell = 40x40 pixels
```

---

## Existing Templates Visualized

### 1. Dense Terraces (DENSE, Difficulty 0)
**Design:** Multiple horizontal platforms with short vertical gaps
**Player Path:** Jump or fall between platforms, shoot enemies

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][💎][ ][ ][ ][ ][ ]
 2   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 3   [█][█][█][█][█][█][█][ ][ ]  ← Solid platform
 4   [ ][ ][─][─][─][─][─][ ][ ]  ← One-way platform
 5   [ ][ ][░][░][ ][ ][ ][ ][ ]  ← Destructible block
 6   [ ][ ][█][█][█][█][█][█][█]  ← Ground enemy walks here
 7   [ ][ ][ ][─][─][─][ ][ ][ ]
 8   [ ][ ][ ][ ][💎][ ][ ][ ][ ]
 9   [█][█][█][█][ ][ ][ ][ ][ ]  ← Platform gap
10   [ ][ ][ ][ ][ ][░][░][ ][ ]
11   [ ][─][─][─][─][ ][ ][ ][ ]
12   [ ][ ][ ][ ][█][█][█][█][█]  ← Heavy enemy
13   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
14   [ ][█][█][█][█][█][█][█][ ]  ← Bottom platform
15   [☠][☠][☠][☠][☠][☠][☠][☠][☠]  ← Hazard spikes

Legend: █=solid ─=one-way ░=destructible 💎=gem ☠=spike
```

**Gameplay:** Safe, beginner-friendly, multiple landing spots

---

### 2. Open Fall (OPEN, Difficulty 0)
**Design:** Wide vertical gaps, sparse platforms
**Player Path:** Must use gunboot to slow descent

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][💎][ ][ ][ ][ ][ ]
 2   [ ][ ][ ][─][─][─][ ][ ][ ]  ← Small platform
 3   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 4   [█][█][█][ ][ ][ ][ ][ ][ ]  ← Left ledge
 5   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 6   [ ][ ][ ][ ][ ][ ][█][█][█]  ← Right ledge
 7   [ ][ ][ ][ ][🦇][ ][ ][ ][ ]  ← Flying enemy
 8   [ ][─][─][─][ ][ ][ ][ ][ ]
 9   [█][█][█][█][ ][ ][ ][ ][ ]
10   [ ][ ][ ][🦇][ ][ ][ ][ ][ ]  ← Flying enemy
11   [ ][ ][ ][ ][ ][░][░][ ][ ]
12   [ ][ ][ ][ ][ ][ ][💎][ ][ ]
13   [ ][ ][ ][ ][█][█][█][█][ ]
14   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
15   [☠][☠][☠][☠][☠][☠][☠][☠][☠]

Legend: 🦇=flying enemy
```

**Gameplay:** Requires gunboot mastery, punishes panic

---

### 3. Combat Gauntlet (COMBAT, Difficulty 2)
**Design:** Enemy-dense with defensive positions
**Player Path:** Fight through multiple enemy types

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 2   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 3   [█][█][█][█][█][█][█][█][█]  ← Full platform
 4   [ ][ ][ ][ ][💎][ ][ ][ ][ ]
 5   [ ][ ][─][─][─][─][─][ ][ ]
 6   [ ][ ][░][ ][ ][ ][ ][ ][ ]
 7   [ ][👾][█][█][█][█][█][🎯][ ]  ← Ground enemy + turret
 8   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 9   [ ][ ][ ][─][─][─][ ][ ][ ]  ← Flying enemy zone
10   [ ][💎][ ][ ][🦇][ ][░][ ][ ]
11   [█][█][█][█][ ][ ][ ][ ][ ]
12   [ ][ ][ ][ ][█][█][█][ ][ ]
13   [ ][ ][ ][🐘][█][█][█][█][█]  ← Heavy enemy
14   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
15   [☠][☠][ ][ ][ ][ ][☠][☠][☠]

Legend: 👾=ground enemy 🎯=turret 🐘=heavy enemy
```

**Gameplay:** Must manage multiple threats, high risk/reward

---

### 4. Hazard Drop (HAZARD, Difficulty 2)
**Design:** Spike placement forces narrow paths
**Player Path:** Precise movement required

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 2   [ ][ ][ ][💎][ ][ ][ ][ ][ ]
 3   [ ][ ][ ][─][─][─][ ][ ][ ]
 4   [█][█][ ][ ][ ][ ][ ][█][█]  ← Narrow corridor
 5   [ ][ ][ ][ ][🦇][ ][ ][ ][ ]
 6   [ ][ ][─][─][─][─][ ][ ][ ]
 7   [ ][ ][█][█][█][█][█][ ][ ]
 8   [☠][☠][ ][ ][ ][ ][ ][☠][☠]  ← Side spikes!
 9   [ ][ ][💎][ ][ ][ ][ ][ ][ ]
10   [█][█][█][ ][ ][ ][ ][ ][ ]
11   [ ][ ][ ][ ][░][░][ ][ ][ ]
12   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
13   [ ][ ][ ][ ][ ][█][█][█][█]  ← Turret position
14   [ ][ ][ ][ ][ ][ ][🎯][ ][ ]
15   [☠][☠][☠][☠][☠][☠][☠][☠][☠]
```

**Gameplay:** Test player precision, instant death risks

---

## Recommended New Templates

### 5. Bonus Vault (REWARD, Difficulty 1) - NEW
**Design:** Walled chamber with destructible block cluster
**Player Path:** Break through blocks for gems

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 2   [█][ ][ ][ ][ ][ ][ ][ ][█]  ← Side walls
 3   [█][ ][ ][ ][ ][ ][ ][ ][█]
 4   [█][ ][ ][ ][ ][ ][ ][ ][█]
 5   [█][ ][ ][ ][ ][ ][ ][ ][█]
 6   [█][ ][ ][░][░][░][ ][ ][█]  ← Block cluster
 7   [█][ ][ ][░][💎][░][ ][ ][█]  ← Gems inside!
 8   [█][ ][ ][░][💎][💎][ ][ ][█]
 9   [█][ ][ ][ ][💎][ ][ ][ ][█]
10   [█][ ][ ][ ][ ][ ][ ][ ][█]
11   [█][ ][ ][ ][ ][ ][ ][ ][█]
12   [█][ ][ ][ ][ ][ ][ ][ ][█]
13   [█][ ][ ][ ][ ][ ][ ][ ][█]
14   [█][🎯][ ][ ][ ][ ][ ][🎯][█]  ← Guarded by turrets
15   [█][█][█][█][█][█][█][█][█]
```

**Gameplay:** High reward but requires ammo to break blocks
**Purpose:** Creates exciting "loot room" moments

---

### 6. Vertical Gauntlet (COMBAT, Difficulty 2) - NEW
**Design:** Narrow shaft with flying enemies
**Player Path:** Dodge and shoot downward

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 2   [█][█][ ][ ][ ][ ][ ][█][█]  ← Narrow shaft starts
 3   [█][█][ ][ ][🦇][ ][ ][█][█]  ← Flying enemy
 4   [█][█][ ][ ][ ][ ][ ][█][█]
 5   [█][█][ ][ ][ ][ ][ ][█][█]
 6   [ ][ ][ ][ ][🦇][ ][ ][ ][ ]  ← Flying enemy
 7   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 8   [█][█][ ][ ][ ][ ][█][█][ ]  ← Offset ledges
 9   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
10   [ ][ ][ ][ ][🦇][ ][ ][ ][ ]  ← Flying enemy
11   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
12   [ ][█][█][█][█][█][█][█][ ]  ← Bottom platform
13   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
14   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
15   [☠][☠][☠][☠][☠][☠][☠][☠][☠]
```

**Gameplay:** Tests vertical gunboot dodging skills
**Purpose:** Unique challenge type, requires different strategy

---

### 7. Split Path Choice (OPEN, Difficulty 1) - NEW
**Design:** Two diverging paths with different challenges
**Player Path:** Choose left (enemy) or right (hazard)

```
Row:
 0   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 1   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 2   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 3   [ ][ ][ ][💎][ ][ ][ ][ ][ ]  ← Pre-split gem
 4   [ ][💎][ ][ ][ ][ ][ ][💎][ ]  ← Incentive for each path
 5   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 6   [█][█][█][ ][ ][ ][█][█][█]  ← Paths diverge
 7   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
 8   [👾][ ][ ][ ][ ][ ][ ][ ][🎯]  ← Left: enemy | Right: turret
 9   [█][ ][░][ ][ ][ ][░][ ][█]  ← Each has blocks
10   [█][ ][ ][ ][ ][ ][ ][ ][█]
11   [█][ ][ ][ ][ ][ ][ ][ ][█]
12   [█][█][█][█][█][█][█][█][█]  ← Paths converge
13   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
14   [ ][ ][ ][ ][ ][ ][ ][ ][ ]
15   [☠][☠][☠][☠][☠][☠][☠][☠][☠]
```

**Gameplay:** Player agency, choose preferred challenge style
**Purpose:** Adds variety, breaks linear feeling

---

## Template Design Principles

### Rule 1: Safe Landing Zones
Every chunk needs 1-2 landing spots within 6-8 rows:
```
Row 0-3:  Optional platform
Row 4-8:  MUST have safe landing
Row 9-12: Optional platform
Row 13-15: Bottom platform (row 14-15 often deadly)
```

### Rule 2: Player Flow
Chunks should guide player naturally downward:
```
Good Flow:              Bad Flow:
    [██]                    [█████████]
      │                           │
      ▼                           │ (player trapped)
     [██]                         │
      │                           ▼
      ▼                         [█████]
    [████]
```

### Rule 3: Challenge Variety Per Chunk
Mix 2-3 challenge types:
- **Navigation:** Platforms, one-ways
- **Combat:** Enemies
- **Obstacles:** Destructibles, hazards
- **Rewards:** Gems

```
Example Mix:
Row 0-5:   Navigation (platforms)
Row 6-9:   Combat (2 enemies)
Row 10-13: Obstacles (destructible + hazard)
Row 14:    Reward (gems)
```

### Rule 4: Variation Distribution
Don't repeat same variation back-to-back:
```
Good Sequence:         Bad Sequence:
Dense                  Dense
Open                   Dense
Combat                 Dense
Dense                  Combat
Hazard                 Combat
Open                   Combat
```

---

## Creating Your Own Template

### Step 1: Choose Variation Type
```
DENSE   - Focus on platforms, minimal empty space
OPEN    - Wide gaps, test gunboot skill
COMBAT  - Enemy-heavy, 3-5 enemies
HAZARD  - Spikes, narrow corridors
REWARD  - Gems, destructibles, low threat
```

### Step 2: Sketch on Grid
Use this blank template:
```
Column:  0   1   2   3   4   5   6   7   8
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
Row 0    │   │   │   │   │   │   │   │   │   │
Row 1    │   │   │   │   │   │   │   │   │   │
Row 2    │   │   │   │   │   │   │   │   │   │
Row 3    │   │   │   │   │   │   │   │   │   │
Row 4    │   │   │   │   │   │   │   │   │   │
Row 5    │   │   │   │   │   │   │   │   │   │
Row 6    │   │   │   │   │   │   │   │   │   │
Row 7    │   │   │   │   │   │   │   │   │   │
Row 8    │   │   │   │   │   │   │   │   │   │
Row 9    │   │   │   │   │   │   │   │   │   │
Row 10   │   │   │   │   │   │   │   │   │   │
Row 11   │   │   │   │   │   │   │   │   │   │
Row 12   │   │   │   │   │   │   │   │   │   │
Row 13   │   │   │   │   │   │   │   │   │   │
Row 14   │   │   │   │   │   │   │   │   │   │
Row 15   │   │   │   │   │   │   │   │   │   │
         └───┴───┴───┴───┴───┴───┴───┴───┴───┘
```

### Step 3: Convert to Code
```gdscript
static func _my_new_template() -> Dictionary:
    return {
        "name": "my_template_name",
        "variation": VARIATION_COMBAT,  # Or DENSE, OPEN, HAZARD
        "difficulty": 1,  # 0=easy, 1=medium, 2=hard, 3=very hard

        "solids": [
            # Solid platforms (can't jump through)
            {"start": Vector2i(column, row), "end": Vector2i(column, row)}
        ],

        "one_way": [
            # One-way platforms (can jump through from below)
            {"start": Vector2i(column, row), "end": Vector2i(column, row)}
        ],

        "hazards": [
            # Spike hazards (instant damage)
            {"start": Vector2i(column, row), "end": Vector2i(column, row)}
        ],

        "destructibles": [
            # Breakable blocks (drop gems when shot)
            {"cell": Vector2i(column, row), "size": Vector2i(width, height)}
        ],

        "enemies": [
            # Enemy spawns
            {"type": "ground", "cell": Vector2i(column, row), "dir": 1},
            {"type": "flying", "cell": Vector2i(column, row), "amp": 60},
            {"type": "turret", "cell": Vector2i(column, row)},
            {"type": "heavy", "cell": Vector2i(column, row), "dir": -1}
        ],

        "gems": [
            # Gem pickups
            {"cell": Vector2i(column, row)}
        ]
    }
```

### Step 4: Add to Templates Array
```gdscript
# In chunk_templates.gd _create_templates()
static func _create_templates() -> Array:
    var result: Array = []
    result.append(_dense_terraces())
    result.append(_dense_vertical_shaft())
    result.append(_open_fall())
    result.append(_open_crossfall())
    result.append(_combat_gauntlet())
    result.append(_combat_barricade())
    result.append(_hazard_drop())
    result.append(_my_new_template())  # ADD YOUR TEMPLATE HERE
    return result
```

---

## Testing Your Template

### In-Game Testing
1. Set `LevelGenerator.START_CHUNKS = 1` to skip initial chunks
2. Temporarily force your template:
   ```gdscript
   func _pick_template(index: int) -> Dictionary:
       return _my_new_template()  # Force yours
   ```
3. Play and observe:
   - Can player land safely?
   - Is challenge fair?
   - Are gems reachable?
   - Do enemies behave correctly?

### Debug Visualization
Add temporary visual markers:
```gdscript
# In level_generator.gd _spawn_template_chunk()
func _spawn_template_chunk(chunk: Node2D, index: int) -> void:
    var template_data: Dictionary = _pick_template(index)

    # DEBUG: Show template name
    var debug_label := Label.new()
    debug_label.text = template_data.get("name", "unknown")
    debug_label.position = Vector2(100, 20)
    chunk.add_child(debug_label)

    # ... rest of function
```

---

## Template Balancing Guidelines

### Difficulty 0 (Early Game)
- 2-3 platforms minimum
- 1-2 enemies max
- No hazards except bottom row
- Generous gem placement
- Wide platforms (3+ cells)

### Difficulty 1 (Mid Game)
- 1-2 platforms (force gunboot use)
- 2-3 enemies
- Optional hazards (not required path)
- Standard gem placement
- Mixed platform sizes

### Difficulty 2 (Late Game)
- 0-1 platforms (vertical skill test)
- 3-4 enemies
- Hazards on main path
- Conditional gem placement (behind blocks)
- Narrow platforms (1-2 cells)

### Difficulty 3 (Expert)
- Minimal platforms
- 4-6 enemies
- Multiple hazard zones
- Gems require risk
- Single-cell platforms

---

## Common Mistakes to Avoid

### Mistake 1: No Safe Landing
```
Bad:                    Good:
Row 0  [█████████]      Row 0  [██]  [██]
Row 1  [ ][ ][ ]        Row 1  [ ][ ][ ]
Row 2  [ ][ ][ ]        Row 2  [ ][ ][ ]
Row 3  [ ][ ][ ]        Row 3    [████]  ← Landing here!
Row 4  [ ][ ][ ]        Row 4  [ ][ ][ ]
```

### Mistake 2: Unreachable Gems
```
Bad:                    Good:
[█][█][💎][█][█]        [█][ ][💎][ ][█]
[ ][ ][ ][ ][ ]         [ ][─][─][─][ ]  ← Can reach via platform
```

### Mistake 3: Impossible Enemy Placement
```
Bad:                    Good:
[█][█][👾][█][█]        [█][█][█][█][█]
                        [👾][ ][ ][ ][ ]  ← Enemy has room to move
```

### Mistake 4: Inescapable Death Trap
```
Bad:                    Good:
[█][ ][ ][ ][█]         [█][ ][ ][ ][█]
[☠][☠][☠][☠][☠]        [☠][ ][ ][ ][☠]  ← Gap to escape
```

---

## Variation Type Examples

### DENSE Template Characteristics
- Platform coverage: 60-80% of chunk
- Vertical gaps: 1-3 cells max
- Horizontal gaps: 0-2 cells max
- Enemy density: Low (1-2)
- Player feel: Safe, methodical

### OPEN Template Characteristics
- Platform coverage: 20-40% of chunk
- Vertical gaps: 4-8 cells
- Horizontal gaps: 3-6 cells
- Enemy density: Low (flying enemies)
- Player feel: Freefall, gunboot-focused

### COMBAT Template Characteristics
- Platform coverage: 40-60% of chunk
- Enemy density: High (3-5)
- Enemy variety: 2-3 types
- Defensive positions: Turrets, heavies
- Player feel: Under pressure, tactical

### HAZARD Template Characteristics
- Spike coverage: 15-30% of chunk
- Platform coverage: 30-50% (narrow paths)
- Enemy density: Medium (1-3)
- Forced precision moments: 2-3
- Player feel: Tense, careful

### REWARD Template Characteristics
- Gem count: 3-5 (vs normal 1-2)
- Destructible blocks: 5-10 cells
- Enemy density: Low-Medium (2-3)
- Challenge type: Obstacle removal
- Player feel: Excited, opportunistic

---

## Final Checklist for New Templates

Before adding a template to production:

- [ ] Name is unique and descriptive
- [ ] Variation type is appropriate
- [ ] Difficulty rating matches challenge level
- [ ] At least 1 safe landing spot exists
- [ ] Player can reach all gems
- [ ] Enemies have room to function
- [ ] No inescapable death traps
- [ ] Tested in-game at least 3 times
- [ ] No game-breaking bugs (stuck players)
- [ ] Fits game's difficulty curve
- [ ] Adds something new (not redundant)

---

**Ready to create?** Start with a simple DENSE template, test it, then work up to complex COMBAT or HAZARD designs!
