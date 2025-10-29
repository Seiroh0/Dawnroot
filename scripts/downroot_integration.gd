extends Node
class_name DownrootIntegration

## Integration helper for Downroot game transformation
## This script provides helper methods to integrate:
## - Intro cutscene system
## - Starting island
## - Tutorial system
## - Enhanced tileset visuals

## How to use this integration:
##
## In game_world.gd _build_world() function, add after camera creation:
## ```
## var downroot_helper: DownrootIntegration = DownrootIntegration.new()
## downroot_helper.setup_downroot_experience(self, player, camera, level_root)
## ```

var intro_cutscene_script: Script = preload("res://scripts/intro_cutscene.gd")
var tutorial_script: Script = preload("res://scripts/tutorial_manager.gd")
var starting_island_script: Script = preload("res://scripts/starting_island.gd")

var intro_cutscene: IntroCutscene = null
var tutorial_manager: TutorialManager = null
var starting_island: StartingIsland = null

## Setup the complete Downroot experience
func setup_downroot_experience(world: GameWorld, player: Player, camera: Camera2D, level_root_node: Node2D) -> void:
	if world == null or player == null or camera == null or level_root_node == null:
		push_error("DownrootIntegration: Missing required references")
		return

	# Create starting island
	starting_island = starting_island_script.new() as StartingIsland
	if starting_island != null:
		starting_island.name = "StartingIsland"
		level_root_node.add_child(starting_island)

		# Position player on the island
		var spawn_pos: Vector2 = starting_island.get_spawn_position()
		player.position = spawn_pos
		camera.position = spawn_pos + Vector2(0, -120)

	# Create intro cutscene
	intro_cutscene = intro_cutscene_script.new() as IntroCutscene
	if intro_cutscene != null:
		intro_cutscene.name = "IntroCutscene"
		world.add_child(intro_cutscene)
		intro_cutscene.setup(player, camera, world)
		intro_cutscene.cutscene_finished.connect(_on_intro_finished.bind(world, player))

		# Start the intro cutscene
		intro_cutscene.play_cutscene()

	# Create tutorial manager (activated after intro)
	tutorial_manager = tutorial_script.new() as TutorialManager
	if tutorial_manager != null:
		tutorial_manager.name = "TutorialManager"
		world.add_child(tutorial_manager)
		tutorial_manager.setup(player)
		tutorial_manager.tutorial_completed.connect(_on_tutorial_completed)

## Called when intro cutscene finishes
func _on_intro_finished(world: GameWorld, _player: Player) -> void:
	print("Downroot: Intro cutscene completed")

	# Start tutorial after intro
	if tutorial_manager != null:
		tutorial_manager.start_tutorial()

	# Begin normal gameplay
	if world != null and world.has_method("_on_gameplay_start"):
		world._on_gameplay_start()

## Called when tutorial completes
func _on_tutorial_completed() -> void:
	print("Downroot: Tutorial completed")

## Load the enhanced background from example.png
static func create_enhanced_background(parent: CanvasLayer) -> void:
	if parent == null:
		return

	var background_texture_path: String = "res://assets/background/example.png"
	if not ResourceLoader.exists(background_texture_path):
		push_warning("Enhanced background not found at: " + background_texture_path)
		return

	var background_texture: Texture2D = ResourceLoader.load(background_texture_path) as Texture2D
	if background_texture == null:
		return

	# Create sprite for background
	var background_sprite: Sprite2D = Sprite2D.new()
	background_sprite.name = "EnhancedBackground"
	background_sprite.texture = background_texture
	background_sprite.centered = true
	background_sprite.position = Vector2(0, 0)

	# Scale to fill screen if needed
	var viewport_size: Vector2 = Vector2(480, 840)  # From project.godot
	var texture_size: Vector2 = Vector2(background_texture.get_width(), background_texture.get_height())
	var scale_x: float = viewport_size.x / texture_size.x
	var scale_y: float = viewport_size.y / texture_size.y
	var scale: float = max(scale_x, scale_y)
	background_sprite.scale = Vector2(scale, scale)

	parent.add_child(background_sprite)

## Load the simple blue sky background
static func create_simple_sky_background(parent: CanvasLayer) -> void:
	if parent == null:
		return

	var sky_texture_path: String = "res://assets/background/background_0.png"
	if not ResourceLoader.exists(sky_texture_path):
		push_warning("Sky background not found at: " + sky_texture_path)
		return

	var sky_texture: Texture2D = ResourceLoader.load(sky_texture_path) as Texture2D
	if sky_texture == null:
		return

	var sky_sprite: Sprite2D = Sprite2D.new()
	sky_sprite.name = "SkyBackground"
	sky_sprite.texture = sky_texture
	sky_sprite.centered = false
	sky_sprite.position = Vector2(0, 0)

	# Tile the background if needed
	var viewport_size: Vector2 = Vector2(480, 840)
	var texture_size: Vector2 = Vector2(sky_texture.get_width(), sky_texture.get_height())
	var scale_x: float = viewport_size.x / texture_size.x
	var scale_y: float = viewport_size.y / texture_size.y
	sky_sprite.scale = Vector2(scale_x, scale_y)

	parent.add_child(sky_sprite)

## Enhance level generator with tileset visuals
static func enhance_level_generator_with_tileset(level_gen: LevelGenerator) -> void:
	if level_gen == null:
		return

	# Load and apply the enhanced tileset
	var enhanced_tileset: TileSet = TilesetLoader.create_enhanced_tileset()
	if enhanced_tileset != null and level_gen.has_method("set_custom_tileset"):
		level_gen.set_custom_tileset(enhanced_tileset)
		print("Downroot: Enhanced tileset applied to level generator")

## Skip intro and tutorial (for testing)
func skip_to_gameplay() -> void:
	if intro_cutscene != null and intro_cutscene.has_method("skip_cutscene"):
		intro_cutscene.skip_cutscene()

	if tutorial_manager != null and tutorial_manager.has_method("skip_tutorial"):
		tutorial_manager.skip_tutorial()
