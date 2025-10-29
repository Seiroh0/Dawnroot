extends Resource
class_name TilesetLoader

# ============================================================================
# TILESET LOADER - DECORATIVE ELEMENTS ONLY
# ============================================================================
#
# PURPOSE:
# This script provides utilities for loading and using tileset.png for
# DECORATIVE/VISUAL elements only (trees, background platforms, etc).
#
# IMPORTANT - DO NOT USE FOR GAMEPLAY TILES:
# The main game uses TILE_SIZE = 40px for gameplay (see level_generator.gd).
# This tileset uses TILE_SIZE = 16px and is NOT compatible with the gameplay grid.
#
# ARCHITECTURE DECISION:
# - level_generator.gd: 40x40 procedural tiles for gameplay (platforms, walls, collision)
# - tileset_loader.gd: 16x16 decorative tiles (visual enhancement, non-collision)
#
# The 40x40 tile size is deeply integrated into:
# - chunk_templates.gd (9x16 grid = 360x640 chunks)
# - All enemy positioning and collision
# - Player movement and physics
# Changing to 16x16 would require complete gameplay rewrite.
#
# USE CASES:
# - Background decorations (trees, scenery)
# - Visual-only platforms in background layers
# - Aesthetic enhancements without collision
# ============================================================================

const TILESET_PATH := "res://assets/background/tileset.png"
const TILE_SIZE := 16  # Decorative tile size from tileset.png (NOT gameplay tiles)

# Tileset atlas coordinates (based on visual inspection of tileset.png)
# These would need to be adjusted based on actual tileset layout
const ATLAS_COORDS := {
	"platform_left": Vector2i(0, 0),
	"platform_mid": Vector2i(1, 0),
	"platform_right": Vector2i(2, 0),
	"grass_block": Vector2i(0, 1),
	"dirt_block": Vector2i(1, 1),
	"tree_trunk": Vector2i(0, 2),
	"tree_foliage_1": Vector2i(1, 2),
	"tree_foliage_2": Vector2i(2, 2),
	"decoration_1": Vector2i(3, 0),
	"decoration_2": Vector2i(3, 1),
	"small_platform": Vector2i(4, 0)
}

static func create_enhanced_tileset() -> TileSet:
	"""
	Creates a TileSet using the tileset.png atlas
	Returns a configured TileSet ready for use in TileMap nodes
	"""
	var tileset: TileSet = TileSet.new()
	tileset.tile_size = Vector2i(TILE_SIZE, TILE_SIZE)

	# Load the tileset texture
	var tileset_texture: Texture2D = null
	if ResourceLoader.exists(TILESET_PATH):
		tileset_texture = ResourceLoader.load(TILESET_PATH) as Texture2D
	else:
		push_warning("Tileset not found at: " + TILESET_PATH)
		return _create_fallback_tileset()

	if tileset_texture == null:
		return _create_fallback_tileset()

	# Create atlas source
	var source_id: int = 0
	var atlas_source: TileSetAtlasSource = TileSetAtlasSource.new()
	atlas_source.texture = tileset_texture
	atlas_source.texture_region_size = Vector2i(TILE_SIZE, TILE_SIZE)

	# Add the atlas source to the tileset
	tileset.add_source(atlas_source, source_id)

	# Configure individual tiles with collision shapes
	_configure_platform_tiles(tileset, source_id)
	_configure_block_tiles(tileset, source_id)

	return tileset

static func _configure_platform_tiles(tileset: TileSet, source_id: int) -> void:
	"""Configure collision for platform tiles"""
	var platform_coords: Array[Vector2i] = [
		ATLAS_COORDS.get("platform_left", Vector2i(0, 0)),
		ATLAS_COORDS.get("platform_mid", Vector2i(1, 0)),
		ATLAS_COORDS.get("platform_right", Vector2i(2, 0)),
		ATLAS_COORDS.get("small_platform", Vector2i(4, 0))
	]

	for coord in platform_coords:
		_add_tile_collision(tileset, source_id, coord, false)  # One-way platforms

static func _configure_block_tiles(tileset: TileSet, source_id: int) -> void:
	"""Configure collision for solid block tiles"""
	var block_coords: Array[Vector2i] = [
		ATLAS_COORDS.get("grass_block", Vector2i(0, 1)),
		ATLAS_COORDS.get("dirt_block", Vector2i(1, 1))
	]

	for coord in block_coords:
		_add_tile_collision(tileset, source_id, coord, true)  # Solid blocks

static func _add_tile_collision(tileset: TileSet, source_id: int, atlas_coord: Vector2i, solid: bool) -> void:
	"""Add collision shape to a specific tile"""
	var source: TileSetAtlasSource = tileset.get_source(source_id) as TileSetAtlasSource
	if source == null:
		return

	# Create the tile if it doesn't exist
	if not source.has_tile(atlas_coord):
		source.create_tile(atlas_coord)

	var tile_data: TileData = source.get_tile_data(atlas_coord, 0)
	if tile_data == null:
		return

	# Add physics layer
	tile_data.set_collision_polygons_count(0, 1)

	# Define collision polygon (full tile)
	var collision_points: PackedVector2Array = PackedVector2Array([
		Vector2(0, 0),
		Vector2(TILE_SIZE, 0),
		Vector2(TILE_SIZE, TILE_SIZE),
		Vector2(0, TILE_SIZE)
	])

	tile_data.set_collision_polygon_points(0, 0, collision_points)

	if not solid:
		# Configure as one-way platform
		tile_data.set_collision_polygon_one_way(0, 0, true)
		tile_data.set_collision_polygon_one_way_margin(0, 0, 6.0)

static func _create_fallback_tileset() -> TileSet:
	"""Create a simple fallback tileset if the texture can't be loaded"""
	var tileset: TileSet = TileSet.new()
	tileset.tile_size = Vector2i(40, 40)

	var source_id: int = 0
	var atlas_source: TileSetAtlasSource = TileSetAtlasSource.new()

	# Create simple colored texture as fallback
	atlas_source.texture = GameConstants.make_rect_texture(
		Color(0.3, 0.5, 0.4),
		Vector2i(40, 40)
	)
	atlas_source.texture_region_size = Vector2i(40, 40)

	tileset.add_source(atlas_source, source_id)

	return tileset

static func create_tree_decoration(position: Vector2, color_variant: int = 0) -> Node2D:
	"""
	Creates a decorative tree using tiles from the tileset
	color_variant: 0 = pink, 1 = orange, 2 = green
	"""
	var tree: Node2D = Node2D.new()
	tree.name = "TreeDecoration"
	tree.position = position

	# Load tileset texture
	var tileset_texture: Texture2D = null
	if ResourceLoader.exists(TILESET_PATH):
		tileset_texture = ResourceLoader.load(TILESET_PATH) as Texture2D

	if tileset_texture != null:
		# Create trunk sprite
		var trunk: Sprite2D = Sprite2D.new()
		trunk.texture = tileset_texture
		trunk.centered = true
		trunk.region_enabled = true
		trunk.region_rect = Rect2(
			ATLAS_COORDS.get("tree_trunk", Vector2i(0, 2)) * TILE_SIZE,
			Vector2(TILE_SIZE, TILE_SIZE * 2)
		)
		trunk.position = Vector2(0, 0)
		tree.add_child(trunk)

		# Create foliage sprite
		var foliage: Sprite2D = Sprite2D.new()
		foliage.texture = tileset_texture
		foliage.centered = true
		foliage.region_enabled = true
		var foliage_coord: Vector2i = ATLAS_COORDS.get("tree_foliage_1", Vector2i(1, 2))
		if color_variant == 1:
			foliage_coord = ATLAS_COORDS.get("tree_foliage_2", Vector2i(2, 2))
		foliage.region_rect = Rect2(
			foliage_coord * TILE_SIZE,
			Vector2(TILE_SIZE * 2, TILE_SIZE * 2)
		)
		foliage.position = Vector2(0, -TILE_SIZE)
		tree.add_child(foliage)

	return tree

static func create_platform_decoration(position: Vector2, width_tiles: int = 3) -> StaticBody2D:
	"""
	Creates a decorative platform using the tileset
	width_tiles: number of tiles wide (minimum 2 for left + right caps)
	"""
	var platform: StaticBody2D = StaticBody2D.new()
	platform.name = "DecorativePlatform"
	platform.position = position
	platform.collision_layer = GameConstants.LAYER_TERRAIN
	platform.collision_mask = GameConstants.LAYER_PLAYER | GameConstants.LAYER_ENEMY

	# Load tileset texture
	var tileset_texture: Texture2D = null
	if ResourceLoader.exists(TILESET_PATH):
		tileset_texture = ResourceLoader.load(TILESET_PATH) as Texture2D

	if tileset_texture != null:
		# Create platform sprites
		for i in range(max(2, width_tiles)):
			var tile: Sprite2D = Sprite2D.new()
			tile.texture = tileset_texture
			tile.region_enabled = true

			var atlas_coord: Vector2i
			if i == 0:
				atlas_coord = ATLAS_COORDS.get("platform_left", Vector2i(0, 0))
			elif i == width_tiles - 1:
				atlas_coord = ATLAS_COORDS.get("platform_right", Vector2i(2, 0))
			else:
				atlas_coord = ATLAS_COORDS.get("platform_mid", Vector2i(1, 0))

			tile.region_rect = Rect2(
				atlas_coord * TILE_SIZE,
				Vector2(TILE_SIZE, TILE_SIZE)
			)
			tile.centered = false
			tile.position = Vector2(i * TILE_SIZE, 0)
			platform.add_child(tile)

		# Add collision shape
		var collision: CollisionShape2D = CollisionShape2D.new()
		var shape: RectangleShape2D = RectangleShape2D.new()
		shape.size = Vector2(width_tiles * TILE_SIZE, 8)
		collision.shape = shape
		collision.position = Vector2(float(width_tiles * TILE_SIZE) / 2.0, 4)
		platform.add_child(collision)

	return platform

static func get_tileset_texture() -> Texture2D:
	"""Returns the loaded tileset texture or null if not found"""
	if ResourceLoader.exists(TILESET_PATH):
		return ResourceLoader.load(TILESET_PATH) as Texture2D
	return null
