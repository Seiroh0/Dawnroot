extends Resource
class_name ChunkTemplates

const TILE_SIZE := 40
const CHUNK_WIDTH := 360
const CHUNK_HEIGHT := 640
const GRID_COLUMNS := 9
const GRID_ROWS := 16
const VARIATION_DENSE := "dense"
const VARIATION_OPEN := "open"
const VARIATION_COMBAT := "combat"
const VARIATION_HAZARD := "hazard"
const VARIATION_SHOP := "shop"

static var templates: Array = _create_templates()

static func _create_templates() -> Array:
	var result: Array = []
	result.append(_dense_terraces())
	result.append(_dense_vertical_shaft())
	result.append(_open_fall())
	result.append(_open_crossfall())
	result.append(_combat_gauntlet())
	result.append(_combat_barricade())
	result.append(_hazard_drop())
	return result

static func _dense_terraces() -> Dictionary:
	return {
		"name": "dense_terraces",
		"variation": VARIATION_DENSE,
		"difficulty": 0,
		"solids": [
			{"start": Vector2i(0, 3), "end": Vector2i(6, 3)},
			{"start": Vector2i(2, 6), "end": Vector2i(8, 6)},
			{"start": Vector2i(0, 9), "end": Vector2i(4, 9)},
			{"start": Vector2i(4, 12), "end": Vector2i(8, 12)},
			{"start": Vector2i(1, 14), "end": Vector2i(7, 14)}
		],
		"one_way": [
			{"start": Vector2i(2, 4), "end": Vector2i(6, 4)},
			{"start": Vector2i(3, 7), "end": Vector2i(5, 7)},
			{"start": Vector2i(1, 11), "end": Vector2i(4, 11)}
		],
		"hazards": [
			{"start": Vector2i(0, 15), "end": Vector2i(8, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(2, 5), "size": Vector2i(2, 1)},
			{"cell": Vector2i(5, 10), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "ground", "cell": Vector2i(2, 6), "dir": 1},
			{"type": "ground", "cell": Vector2i(6, 9), "dir": -1},
			{"type": "flying", "cell": Vector2i(4, 4), "amp": 40, "speed": 1.2}
		],
		"gems": [
			{"cell": Vector2i(4, 2)},
			{"cell": Vector2i(5, 8)}
		]
	}

static func _dense_vertical_shaft() -> Dictionary:
	return {
		"name": "dense_vertical_shaft",
		"variation": VARIATION_DENSE,
		"difficulty": 1,
		"solids": [
			{"start": Vector2i(0, 2), "end": Vector2i(2, 2)},
			{"start": Vector2i(6, 2), "end": Vector2i(8, 2)},
			{"start": Vector2i(1, 5), "end": Vector2i(5, 5)},
			{"start": Vector2i(3, 8), "end": Vector2i(7, 8)},
			{"start": Vector2i(0, 11), "end": Vector2i(3, 11)},
			{"start": Vector2i(5, 13), "end": Vector2i(8, 13)}
		],
		"one_way": [
			{"start": Vector2i(3, 3), "end": Vector2i(5, 3)},
			{"start": Vector2i(2, 7), "end": Vector2i(6, 7)}
		],
		"hazards": [
			{"start": Vector2i(0, 15), "end": Vector2i(3, 15)},
			{"start": Vector2i(5, 15), "end": Vector2i(8, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(1, 6), "size": Vector2i(1, 1)},
			{"cell": Vector2i(6, 10), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "ground", "cell": Vector2i(1, 5), "dir": 1},
			{"type": "ground", "cell": Vector2i(6, 8), "dir": -1},
			{"type": "heavy", "cell": Vector2i(4, 13), "dir": 1}
		],
		"gems": [
			{"cell": Vector2i(4, 4)},
			{"cell": Vector2i(3, 10)}
		]
	}

static func _open_fall() -> Dictionary:
	return {
		"name": "open_fall",
		"variation": VARIATION_OPEN,
		"difficulty": 0,
		"solids": [
			{"start": Vector2i(0, 4), "end": Vector2i(2, 4)},
			{"start": Vector2i(6, 6), "end": Vector2i(8, 6)},
			{"start": Vector2i(2, 9), "end": Vector2i(4, 9)},
			{"start": Vector2i(4, 13), "end": Vector2i(7, 13)}
		],
		"one_way": [
			{"start": Vector2i(3, 2), "end": Vector2i(5, 2)},
			{"start": Vector2i(1, 8), "end": Vector2i(3, 8)}
		],
		"hazards": [
			{"start": Vector2i(0, 15), "end": Vector2i(8, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(5, 11), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "flying", "cell": Vector2i(4, 6), "amp": 80, "speed": 1.4},
			{"type": "flying", "cell": Vector2i(3, 10), "amp": 60, "speed": 1.1}
		],
		"gems": [
			{"cell": Vector2i(4, 1)},
			{"cell": Vector2i(6, 12)}
		]
	}

static func _open_crossfall() -> Dictionary:
	return {
		"name": "open_crossfall",
		"variation": VARIATION_OPEN,
		"difficulty": 1,
		"solids": [
			{"start": Vector2i(0, 5), "end": Vector2i(2, 5)},
			{"start": Vector2i(6, 5), "end": Vector2i(8, 5)},
			{"start": Vector2i(3, 9), "end": Vector2i(5, 9)},
			{"start": Vector2i(0, 12), "end": Vector2i(4, 12)},
			{"start": Vector2i(5, 14), "end": Vector2i(8, 14)}
		],
		"one_way": [
			{"start": Vector2i(2, 3), "end": Vector2i(6, 3)},
			{"start": Vector2i(1, 7), "end": Vector2i(3, 7)}
		],
		"hazards": [
			{"start": Vector2i(3, 15), "end": Vector2i(5, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(6, 10), "size": Vector2i(1, 1)}
		],
		"enemies": [
			{"type": "flying", "cell": Vector2i(4, 4), "amp": 70, "speed": 1.5},
			{"type": "turret", "cell": Vector2i(2, 12)},
			{"type": "ground", "cell": Vector2i(6, 14), "dir": -1}
		],
		"gems": [
			{"cell": Vector2i(4, 2)},
			{"cell": Vector2i(2, 11)},
			{"cell": Vector2i(7, 13)}
		]
	}

static func _combat_gauntlet() -> Dictionary:
	return {
		"name": "combat_gauntlet",
		"variation": VARIATION_COMBAT,
		"difficulty": 2,
		"solids": [
			{"start": Vector2i(0, 3), "end": Vector2i(8, 3)},
			{"start": Vector2i(1, 7), "end": Vector2i(7, 7)},
			{"start": Vector2i(0, 11), "end": Vector2i(4, 11)},
			{"start": Vector2i(4, 13), "end": Vector2i(8, 13)}
		],
		"one_way": [
			{"start": Vector2i(2, 5), "end": Vector2i(6, 5)},
			{"start": Vector2i(3, 9), "end": Vector2i(5, 9)}
		],
		"hazards": [
			{"start": Vector2i(0, 15), "end": Vector2i(2, 15)},
			{"start": Vector2i(6, 15), "end": Vector2i(8, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(2, 6), "size": Vector2i(1, 1)},
			{"cell": Vector2i(6, 10), "size": Vector2i(1, 1)},
			{"cell": Vector2i(4, 12), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "ground", "cell": Vector2i(2, 7), "dir": 1},
			{"type": "turret", "cell": Vector2i(6, 7)},
			{"type": "heavy", "cell": Vector2i(3, 13), "dir": 1},
			{"type": "flying", "cell": Vector2i(5, 9), "amp": 50, "speed": 1.2}
		],
		"gems": [
			{"cell": Vector2i(4, 4)},
			{"cell": Vector2i(1, 10)},
			{"cell": Vector2i(7, 12)}
		]
	}

static func _combat_barricade() -> Dictionary:
	return {
		"name": "combat_barricade",
		"variation": VARIATION_COMBAT,
		"difficulty": 3,
		"solids": [
			{"start": Vector2i(0, 2), "end": Vector2i(8, 2)},
			{"start": Vector2i(0, 6), "end": Vector2i(3, 6)},
			{"start": Vector2i(5, 6), "end": Vector2i(8, 6)},
			{"start": Vector2i(2, 9), "end": Vector2i(6, 9)},
			{"start": Vector2i(0, 12), "end": Vector2i(8, 12)}
		],
		"one_way": [
			{"start": Vector2i(3, 4), "end": Vector2i(5, 4)},
			{"start": Vector2i(1, 8), "end": Vector2i(7, 8)}
		],
		"hazards": [
			{"start": Vector2i(0, 14), "end": Vector2i(1, 14)},
			{"start": Vector2i(7, 14), "end": Vector2i(8, 14)},
			{"start": Vector2i(2, 15), "end": Vector2i(6, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(4, 5), "size": Vector2i(1, 1)},
			{"cell": Vector2i(2, 10), "size": Vector2i(2, 1)},
			{"cell": Vector2i(5, 10), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "turret", "cell": Vector2i(1, 6)},
			{"type": "turret", "cell": Vector2i(7, 6)},
			{"type": "ground", "cell": Vector2i(3, 9), "dir": 1},
			{"type": "ground", "cell": Vector2i(5, 9), "dir": -1},
			{"type": "heavy", "cell": Vector2i(4, 12), "dir": -1},
			{"type": "flying", "cell": Vector2i(4, 5), "amp": 60, "speed": 1.6}
		],
		"gems": [
			{"cell": Vector2i(4, 1)},
			{"cell": Vector2i(2, 7)},
			{"cell": Vector2i(6, 7)},
			{"cell": Vector2i(4, 11)}
		]
	}

static func _hazard_drop() -> Dictionary:
	return {
		"name": "hazard_drop",
		"variation": VARIATION_HAZARD,
		"difficulty": 2,
		"solids": [
			{"start": Vector2i(0, 4), "end": Vector2i(1, 4)},
			{"start": Vector2i(7, 4), "end": Vector2i(8, 4)},
			{"start": Vector2i(2, 7), "end": Vector2i(6, 7)},
			{"start": Vector2i(0, 10), "end": Vector2i(3, 10)},
			{"start": Vector2i(5, 13), "end": Vector2i(8, 13)}
		],
		"one_way": [
			{"start": Vector2i(3, 3), "end": Vector2i(5, 3)},
			{"start": Vector2i(2, 6), "end": Vector2i(4, 6)}
		],
		"hazards": [
			{"start": Vector2i(0, 8), "end": Vector2i(1, 8)},
			{"start": Vector2i(7, 8), "end": Vector2i(8, 8)},
			{"start": Vector2i(0, 15), "end": Vector2i(8, 15)}
		],
		"destructibles": [
			{"cell": Vector2i(4, 11), "size": Vector2i(2, 1)}
		],
		"enemies": [
			{"type": "ground", "cell": Vector2i(3, 7), "dir": -1},
			{"type": "flying", "cell": Vector2i(5, 5), "amp": 70, "speed": 1.3},
			{"type": "turret", "cell": Vector2i(6, 13)}
		],
		"gems": [
			{"cell": Vector2i(4, 2)},
			{"cell": Vector2i(2, 9)},
			{"cell": Vector2i(6, 12)}
		]
	}

static func all_templates() -> Array:
	return templates.duplicate(true)

static func templates_for_depth(chunk_index: int) -> Array:
	var difficulty: int = int(chunk_index / 4)
	var filtered: Array = []
	for template in templates:
		var template_dict: Dictionary = template
		if template_dict.get("difficulty", 0) <= difficulty + 1:
			filtered.append(template_dict)
	return filtered if not filtered.is_empty() else templates.duplicate(true)

static func filter_by_variation(template_list: Array, variation: String) -> Array:
	var filtered: Array = []
	for template in template_list:
		var template_dict: Dictionary = template
		if template_dict.get("variation") == variation:
			filtered.append(template_dict)
	return filtered
