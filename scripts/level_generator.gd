extends Node2D
class_name LevelGenerator

signal shop_chunk_spawned(index: int, chunk: Node)

const CHUNK_HEIGHT := 640
const CHUNK_WIDTH := 360
const TILE_SIZE := 40
const START_CHUNKS := 4
const CHUNKS_AHEAD := 3
const CLEANUP_DISTANCE := 2
const MAX_ACTIVE_CHUNKS := 10
const SHOP_INTERVAL_MIN := 10
const SHOP_INTERVAL_MAX := 15

var ground_enemy_script: Script = preload("res://scripts/enemies/ground_enemy.gd")
var flying_enemy_script: Script = preload("res://scripts/enemies/flying_enemy.gd")
var turret_enemy_script: Script = preload("res://scripts/enemies/turret_enemy.gd")
var heavy_enemy_script: Script = preload("res://scripts/enemies/heavy_enemy.gd")
var block_script: Script = preload("res://scripts/destructible_block.gd")
var gem_script: Script = preload("res://scripts/gem.gd")
var hazard_script: Script = preload("res://scripts/hazard_spike.gd")

var rng: RandomNumberGenerator = RandomNumberGenerator.new()
var player: Player = null
var world: Node = null

var lowest_index: int = -1
var active_chunks: Dictionary = {}
var palette: Dictionary = {
	"platform": Color(0.25, 0.22, 0.3),
	"one_way": Color(0.35, 0.32, 0.45),
	"wall": Color(0.1, 0.08, 0.12),
	"block": Color(0.35, 0.32, 0.4),
	"gem": Color(0.6, 1.0, 0.85),
	"enemy": Color(0.85, 0.3, 0.35),
	"hazard": Color(0.8, 0.2, 0.25),
	"shop_text": Color(1, 0.95, 0.8)
}

var tile_set: TileSet = TileSet.new()
var tile_source_ids: Dictionary = {}
var next_shop_index: int = 10
var variation_history: Array[String] = []

func setup(game_world: Node, player_ref: Player) -> void:
	world = game_world
	player = player_ref
	rng.randomize()
	_build_tile_set()
	next_shop_index = _roll_next_shop_index(6)
	for i in range(START_CHUNKS):
		_spawn_chunk(i)
	lowest_index = START_CHUNKS - 1

func set_palette(data: Dictionary) -> void:
	for key in data.keys():
		palette[key] = data[key]
	_build_tile_set()

	# Update all existing TileMaps to use the new TileSet
	for chunk_value in active_chunks.values():
		var chunk_node: Node2D = chunk_value as Node2D
		if chunk_node == null:
			continue
		for child in chunk_node.get_children():
			if child is TileMap:
				child.tile_set = tile_set

func update_chunks() -> void:
	if player == null:
		return
	var player_y := player.global_position.y
	while player_y + float(CHUNK_HEIGHT * CHUNKS_AHEAD) > float(lowest_index + 1) * CHUNK_HEIGHT:
		lowest_index += 1
		_spawn_chunk(lowest_index)
	_cleanup_chunks(player_y)

func _cleanup_chunks(player_y: float) -> void:
	var removal_indices: Array[int] = []
	for chunk_key in active_chunks.keys():
		var chunk_index: int = int(chunk_key)
		var chunk_node: Node2D = active_chunks[chunk_index] as Node2D
		if chunk_node == null:
			continue
		var limit: float = player_y - float(CHUNK_HEIGHT * CLEANUP_DISTANCE)
		if chunk_node.global_position.y + float(CHUNK_HEIGHT) < limit:
			removal_indices.append(chunk_index)
	for chunk_index in removal_indices:
		var chunk_node: Node2D = active_chunks.get(chunk_index, null)
		active_chunks.erase(chunk_index)
		if chunk_node != null and is_instance_valid(chunk_node):
			chunk_node.queue_free()
	_cull_out_of_bounds_enemies(player_y)

func _cull_out_of_bounds_enemies(player_y: float) -> void:
	var upper_limit: float = player_y - float(CHUNK_HEIGHT * CLEANUP_DISTANCE)
	var lower_limit: float = player_y + float(CHUNK_HEIGHT * (CHUNKS_AHEAD + 1))
	for chunk_value in active_chunks.values():
		var chunk_node: Node2D = chunk_value as Node2D
		if chunk_node == null:
			continue
		for child_node in chunk_node.get_children():
			var enemy_instance: EnemyBase = child_node as EnemyBase
			if enemy_instance == null:
				continue
			var culled_upper: float = upper_limit - float(TILE_SIZE)
			if enemy_instance.global_position.y < culled_upper or enemy_instance.global_position.y > lower_limit:
				if is_instance_valid(enemy_instance):
					enemy_instance.queue_free()

func _spawn_chunk(index: int) -> void:
	var chunk_node: Node2D = Node2D.new()
	chunk_node.name = "Chunk_%d" % index
	chunk_node.position = Vector2(-float(CHUNK_WIDTH) * 0.5, float(index * CHUNK_HEIGHT))
	add_child(chunk_node)
	active_chunks[index] = chunk_node

	if index == 0:
		_spawn_start_chunk(chunk_node)
	elif index >= next_shop_index:
		_spawn_shop_chunk(chunk_node, index)
		next_shop_index = _roll_next_shop_index(index)
	else:
		_spawn_template_chunk(chunk_node, index)

	if active_chunks.size() > MAX_ACTIVE_CHUNKS:
		var keys: Array = active_chunks.keys()
		if keys.is_empty():
			return
		var oldest_index: int = int(keys.min())
		if oldest_index < index - MAX_ACTIVE_CHUNKS:
			var obsolete_node: Node2D = active_chunks.get(oldest_index, null)
			active_chunks.erase(oldest_index)
			if obsolete_node != null and is_instance_valid(obsolete_node):
				obsolete_node.queue_free()

func _spawn_start_chunk(chunk: Node2D) -> void:
	var tile_map: TileMap = _create_tile_map()
	chunk.add_child(tile_map)
	_fill_horizontal(tile_map, 0, 13, 8, "solid")
	_fill_horizontal(tile_map, 0, 10, 6, "solid")
	_fill_horizontal(tile_map, 0, 7, 8, "solid")
	_fill_horizontal(tile_map, 0, 4, 6, "solid")
	_spawn_platform_enemy(chunk, Vector2i(4, 10), "ground")

func _spawn_template_chunk(chunk: Node2D, index: int) -> void:
	var template_data: Dictionary = _pick_template(index)
	if template_data.is_empty():
		return
	chunk.set_meta("template_name", template_data.get("name", "unknown"))
	var tile_map: TileMap = _create_tile_map()
	chunk.add_child(tile_map)
	_fill_from_template(chunk, tile_map, template_data, index)
	var variation_name: String = String(template_data.get("variation", "none"))
	variation_history.append(variation_name)
	if variation_history.size() > 6:
		variation_history.pop_front()

func _fill_from_template(chunk: Node2D, tile_map: TileMap, template: Dictionary, index: int) -> void:
	var solid_ranges: Array = template.get("solids", [])
	for solid_range in solid_ranges:
		_fill_range(tile_map, 0, solid_range, "solid")
	var one_way_ranges: Array = template.get("one_way", [])
	for one_way_range in one_way_ranges:
		_fill_range(tile_map, 1, one_way_range, "one_way")
	var hazard_ranges: Array = template.get("hazards", [])
	for hazard_range in hazard_ranges:
		_spawn_hazard_range(chunk, hazard_range)
	var destructible_defs: Array = template.get("destructibles", [])
	for destructible_def in destructible_defs:
		_spawn_destructible(chunk, destructible_def, index)
	var enemy_defs: Array = template.get("enemies", [])
	for enemy_def in enemy_defs:
		_spawn_enemy(chunk, enemy_def, index)
	var gem_defs: Array = template.get("gems", [])
	for gem_def in gem_defs:
		_spawn_gem(chunk, gem_def, index)

	_apply_dynamic_scaling(chunk, tile_map, template, index)

func _apply_dynamic_scaling(chunk: Node2D, tile_map: TileMap, _template: Dictionary, index: int) -> void:
	var depth_factor: int = max(0, index - 3)
	if depth_factor <= 0:
		return

	if depth_factor % 4 == 0:
		var extra_enemy: Dictionary = {
			"type": "ground" if rng.randf() < 0.5 else "flying",
			"cell": Vector2i(rng.randi_range(1, 7), rng.randi_range(3, 13)),
			"speed": 1.0 + rng.randf_range(0.0, 0.6),
			"amp": 40 + rng.randi_range(0, 30)
		}
		_spawn_enemy(chunk, extra_enemy, index)

	if depth_factor % 6 == 0:
		var hazard := {
			"start": Vector2i(rng.randi_range(0, 2), rng.randi_range(10, 15)),
			"end": Vector2i(rng.randi_range(6, 8), rng.randi_range(10, 15))
		}
		_spawn_hazard_range(chunk, hazard)

	if depth_factor % 5 == 0:
		var drywall := {
			"start": Vector2i(rng.randi_range(0, 4), rng.randi_range(6, 11)),
			"end": Vector2i(rng.randi_range(4, 8), rng.randi_range(6, 11))
		}
		_fill_range(tile_map, 0, drywall, "solid")

func _fill_range(tile_map: TileMap, layer: int, range_data: Dictionary, source_key: String) -> void:
	var source_id: int = int(tile_source_ids.get(source_key, -1))
	if source_id == -1:
		push_warning("LevelGenerator: Tile source '%s' not found in tile_source_ids" % source_key)
		return

	var start: Vector2i = range_data.get("start", Vector2i.ZERO)
	var end: Vector2i = range_data.get("end", start)
	var min_x: int = min(start.x, end.x)
	var max_x: int = max(start.x, end.x)
	var min_y: int = min(start.y, end.y)
	var max_y: int = max(start.y, end.y)
	for x in range(min_x, max_x + 1):
		for y in range(min_y, max_y + 1):
			tile_map.set_cell(layer, Vector2i(x, y), source_id, Vector2i.ZERO, 0)

func _fill_horizontal(tile_map: TileMap, layer: int, row: int, width: int, source_key: String) -> void:
	var source_id: int = int(tile_source_ids.get(source_key, -1))
	if source_id == -1:
		push_warning("LevelGenerator: Tile source '%s' not found in tile_source_ids" % source_key)
		return
	var start_x: int = int(floor((ChunkTemplates.GRID_COLUMNS - width) / 2.0))
	var end_x := start_x + width
	for x in range(start_x, end_x):
		tile_map.set_cell(layer, Vector2i(x, row), source_id, Vector2i.ZERO, 0)

func _spawn_destructible(chunk: Node2D, def: Dictionary, index: int) -> void:
	var block: DestructibleBlock = block_script.new() as DestructibleBlock
	if block == null:
		return
	var cell: Vector2i = def.get("cell", Vector2i(3, 6))
	var size_cells: Vector2i = def.get("size", Vector2i.ONE)
	var size_px: Vector2 = Vector2(float(size_cells.x * TILE_SIZE), float(size_cells.y * TILE_SIZE))
	var bottom: Vector2 = Vector2(float(cell.x * TILE_SIZE), float((cell.y + size_cells.y) * TILE_SIZE))
	block.position = bottom
	var toughness: int = rng.randi_range(2, 4 + int(float(index) / 6.0))
	var reward_data: Dictionary = {
		"gems": rng.randi_range(6, 14),
		"color": palette.get("block", Color(0.35, 0.32, 0.4))
	}
	block.configure(size_px, toughness, reward_data)
	chunk.add_child(block)
	block.broken.connect(Callable(world, "_on_block_broken"))

func _spawn_enemy(chunk: Node2D, def: Dictionary, index: int) -> void:
	var enemy_type: String = String(def.get("type", "ground"))
	var enemy_instance: EnemyBase = null
	var spawn_params: Dictionary = def.duplicate(true)
	match enemy_type:
		"ground":
			enemy_instance = ground_enemy_script.new() as EnemyBase
			spawn_params["dir"] = spawn_params.get("dir", -1 if rng.randf() < 0.5 else 1)
			spawn_params["speed"] = spawn_params.get("speed", 90.0 + float(index) * 2.0)
		"flying":
			enemy_instance = flying_enemy_script.new() as EnemyBase
			spawn_params["amp"] = spawn_params.get("amp", 60 + index * 2)
			spawn_params["speed"] = spawn_params.get("speed", 1.2 + float(index) * 0.04)
			spawn_params["fall"] = spawn_params.get("fall", 160.0 + float(index) * 6.0)
		"turret":
			enemy_instance = turret_enemy_script.new() as EnemyBase
			spawn_params["interval"] = clamp(spawn_params.get("interval", 1.6 - min(0.9, float(index) * 0.03)), 0.6, 2.5)
			spawn_params["projectile_speed"] = spawn_params.get("projectile_speed", 420.0 + float(index) * 6.0)
		"heavy":
			enemy_instance = heavy_enemy_script.new() as EnemyBase
			spawn_params["dir"] = spawn_params.get("dir", -1 if rng.randf() < 0.5 else 1)
			spawn_params["speed"] = spawn_params.get("speed", 60.0 + float(index) * 1.5)
		_:
			enemy_instance = ground_enemy_script.new() as EnemyBase
			spawn_params["dir"] = spawn_params.get("dir", -1 if rng.randf() < 0.5 else 1)
			spawn_params["speed"] = spawn_params.get("speed", 90.0 + float(index) * 2.0)

	if enemy_instance == null:
		return

	if enemy_instance.has_method("set_spawn_params"):
		enemy_instance.call("set_spawn_params", spawn_params)

	var cell: Vector2i = def.get("cell", Vector2i(4, 8))
	var enemy_position: Vector2 = _cell_center(cell)
	if enemy_type == "ground":
		enemy_position.y = float(cell.y * TILE_SIZE - 4)
	elif enemy_type == "heavy":
		enemy_position.y = float(cell.y * TILE_SIZE - 6)
	elif enemy_type == "turret":
		enemy_position.y = float(cell.y * TILE_SIZE - 8)

	enemy_instance.position = enemy_position
	var enemy_palette: Dictionary = {
		"primary": palette.get("enemy", Color(0.85, 0.3, 0.35)),
		"outline": palette.get("wall", Color(0.1, 0.08, 0.12)),
		"accent": palette.get("gem", Color(0.6, 1.0, 0.85))
	}
	enemy_instance.configure_palette(enemy_palette)
	chunk.add_child(enemy_instance)

	if enemy_instance.has_signal("defeated"):
		enemy_instance.defeated.connect(Callable(world, "_on_enemy_defeated"))
	if enemy_instance.has_signal("spawn_gems"):
		enemy_instance.spawn_gems.connect(func(spawn_position: Vector2, amount: int) -> void:
			_on_enemy_spawn_gems(chunk, spawn_position, amount)
		)

func _on_enemy_spawn_gems(chunk: Node2D, spawn_position: Vector2, amount: int) -> void:
	for _i in range(amount):
		var gem_instance: Gem = gem_script.new() as Gem
		if gem_instance == null:
			continue
		var gem_data: Dictionary = {
			"value": 1 + rng.randi_range(0, 1),
			"color": palette.get("gem", Color(0.6, 1.0, 0.85))
		}
		gem_instance.configure(gem_data)
		gem_instance.position = spawn_position + Vector2(rng.randf_range(-12.0, 12.0), rng.randf_range(-4.0, 8.0))
		chunk.add_child(gem_instance)
		gem_instance.collected.connect(Callable(world, "_on_gem_collected"))

func _spawn_gem(chunk: Node2D, def: Dictionary, index: int) -> void:
	var gem_instance: Gem = gem_script.new() as Gem
	if gem_instance == null:
		return
	var cell: Vector2i = def.get("cell", Vector2i(4, 6))
	var gem_config: Dictionary = {
		"value": def.get("value", rng.randi_range(2, 4 + int(float(index) / 6.0))),
		"color": palette.get("gem", Color(0.6, 1.0, 0.85))
	}
	gem_instance.configure(gem_config)
	gem_instance.position = _cell_center(cell)
	chunk.add_child(gem_instance)
	gem_instance.collected.connect(Callable(world, "_on_gem_collected"))

func _spawn_hazard_range(chunk: Node2D, def: Dictionary) -> void:
	var start: Vector2i = def.get("start", Vector2i(0, 15))
	var end: Vector2i = def.get("end", start)
	var min_x: int = min(start.x, end.x)
	var max_x: int = max(start.x, end.x)
	var min_y: int = min(start.y, end.y)
	var max_y: int = max(start.y, end.y)
	for x in range(min_x, max_x + 1):
		for y in range(min_y, max_y + 1):
			var hazard_instance: HazardSpike = hazard_script.new() as HazardSpike
			if hazard_instance == null:
				continue
			hazard_instance.position = _cell_center(Vector2i(x, y)) + Vector2(0.0, float(TILE_SIZE) * 0.25)
			hazard_instance.set_palette(palette)
			chunk.add_child(hazard_instance)
			hazard_instance.body_damaged.connect(Callable(self, "_on_hazard_body_damaged"))

func _on_hazard_body_damaged(body_node: Node, hazard_node: Node) -> void:
	if body_node != null and body_node.has_method("take_damage"):
		body_node.take_damage(1, hazard_node)

func _spawn_platform_enemy(chunk: Node2D, cell: Vector2i, enemy_type: String) -> void:
	_spawn_enemy(chunk, {"type": enemy_type, "cell": cell}, 0)

func _spawn_shop_chunk(chunk: Node2D, index: int) -> void:
	var tile_map := _create_tile_map()
	chunk.add_child(tile_map)
	_fill_range(tile_map, 0, {"start": Vector2i(0, 14), "end": Vector2i(8, 14)}, "solid")
	_fill_range(tile_map, 1, {"start": Vector2i(1, 10), "end": Vector2i(7, 10)}, "one_way")

	var shop_label := Label.new()
	shop_label.text = "SHOP"
	shop_label.modulate = palette.get("shop_text", Color(1, 0.95, 0.8))
	shop_label.position = Vector2(CHUNK_WIDTH * 0.5 - 30, 320)
	chunk.add_child(shop_label)

	var area := Area2D.new()
	area.collision_layer = GameConstants.LAYER_SHOP
	area.collision_mask = GameConstants.LAYER_PLAYER
	var shape := RectangleShape2D.new()
	shape.size = Vector2(180, 200)
	var collision := CollisionShape2D.new()
	collision.shape = shape
	collision.position = Vector2(CHUNK_WIDTH * 0.5, 300)
	area.add_child(collision)
	area.body_entered.connect(func(body: Node) -> void:
		if body is Player:
			world._on_shop_triggered({"index": index})
	)
	chunk.add_child(area)
	emit_signal("shop_chunk_spawned", index, chunk)

func _pick_template(index: int) -> Dictionary:
	var pool := ChunkTemplates.templates_for_depth(index)
	if pool.is_empty():
		return {}

	var weights: Array[float] = []
	for template in pool:
		var variation: String = template.get("variation", "dense")
		var base_weight := 1.0
		match variation:
			ChunkTemplates.VARIATION_DENSE:
				base_weight = 1.0
			ChunkTemplates.VARIATION_OPEN:
				base_weight = 1.25
			ChunkTemplates.VARIATION_COMBAT:
				base_weight = 0.8 + float(index) * 0.05
			ChunkTemplates.VARIATION_HAZARD:
				base_weight = 0.6 + float(index) * 0.04
			_:
				base_weight = 1.0

		if variation_history.count(variation) >= 2:
			base_weight *= 0.5

		weights.append(max(0.1, base_weight))

	var choice_index := _weighted_choice(weights)
	return pool[choice_index]

func _weighted_choice(weights: Array[float]) -> int:
	var total := 0.0
	for weight in weights:
		total += max(weight, 0.0)
	var pick := rng.randf_range(0.0, total)
	var cumulative := 0.0
	for i in weights.size():
		cumulative += max(weights[i], 0.0)
		if pick <= cumulative:
			return i
	return max(weights.size() - 1, 0)

func _cell_center(cell: Vector2i) -> Vector2:
	return Vector2(cell.x * TILE_SIZE + TILE_SIZE * 0.5, cell.y * TILE_SIZE + TILE_SIZE * 0.5)

func _create_tile_map() -> TileMap:
	var tile_map := TileMap.new()
	tile_map.tile_set = tile_set

	# Collision layers are set per-tile in the TileData (see _add_tile_source)
	# TileMap itself doesn't have collision_layer/collision_mask properties in Godot 4.x

	# Add two layers: solid and one_way
	tile_map.add_layer(0)
	tile_map.add_layer(1)
	tile_map.set_layer_name(0, "solid")
	tile_map.set_layer_name(1, "one_way")
	tile_map.set_layer_y_sort_enabled(0, false)
	tile_map.set_layer_y_sort_enabled(1, false)
	return tile_map

func _build_tile_set() -> void:
	tile_set = TileSet.new()
	tile_source_ids.clear()

	# Add physics layer to TileSet and configure collision layers/masks
	# In Godot 4.x, physics layers are defined on the TileSet, not on individual TileMaps
	tile_set.add_physics_layer()
	tile_set.set_physics_layer_collision_layer(0, GameConstants.LAYER_TERRAIN)
	tile_set.set_physics_layer_collision_mask(0, GameConstants.LAYER_PLAYER | GameConstants.LAYER_ENEMY | GameConstants.LAYER_PLAYER_SHOT)

	_add_tile_source("solid", palette.get("platform", Color(0.25, 0.22, 0.3)), false)
	_add_tile_source("one_way", palette.get("one_way", Color(0.35, 0.32, 0.45)), true)

func _add_tile_source(source_name: String, color: Color, one_way: bool) -> void:
	var source_id: int = tile_set.get_next_source_id()
	var atlas := TileSetAtlasSource.new()
	atlas.texture = GameConstants.make_rect_texture(color, Vector2i(TILE_SIZE, TILE_SIZE))

	# Verify texture was created successfully
	if atlas.texture == null:
		push_error("LevelGenerator: Failed to create texture for tile source '%s'" % source_name)
		return

	atlas.texture_region_size = Vector2i(TILE_SIZE, TILE_SIZE)
	atlas.create_tile(Vector2i.ZERO)

	# Verify tile was created
	if not atlas.has_tile(Vector2i.ZERO):
		push_error("LevelGenerator: Failed to create tile at Vector2i.ZERO for source '%s'" % source_name)
		return

	tile_set.add_source(atlas, source_id)

	var tile_data: TileData = atlas.get_tile_data(Vector2i.ZERO, 0)
	if tile_data != null:
		tile_data.set_collision_polygons_count(0, 1)
		tile_data.set_collision_polygon_points(0, 0, PackedVector2Array([
			Vector2(0, 0),
			Vector2(TILE_SIZE, 0),
			Vector2(TILE_SIZE, TILE_SIZE),
			Vector2(0, TILE_SIZE)
		]))
		tile_data.set_collision_polygon_one_way(0, 0, one_way)
		tile_data.set_collision_polygon_one_way_margin(0, 0, 6.0)
		# Note: set_collision_polygon_one_way_direction() does not exist in Godot 4.x
		# One-way platforms work from above by default

	tile_source_ids[source_name] = source_id

func _roll_next_shop_index(current_index: int) -> int:
	return current_index + rng.randi_range(SHOP_INTERVAL_MIN, SHOP_INTERVAL_MAX)
