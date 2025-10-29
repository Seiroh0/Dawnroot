extends Node2D
class_name StartingIsland

# This creates the starting island scene inspired by example.png
# Features: Floating island with pink/orange trees, blue sky, stars

const ISLAND_WIDTH := 320
const ISLAND_HEIGHT := 180
const TREE_COLORS := {
	"pink_light": Color(1.0, 0.7, 0.6),
	"pink_medium": Color(0.95, 0.55, 0.45),
	"pink_dark": Color(0.85, 0.4, 0.35),
	"orange_light": Color(1.0, 0.6, 0.4),
	"orange_dark": Color(0.9, 0.45, 0.3)
}

const SKY_COLORS := {
	"sky_top": Color(0.3, 0.45, 0.7),
	"sky_bottom": Color(0.45, 0.6, 0.85),
	"cloud": Color(0.55, 0.65, 0.85, 0.6),
	"star": Color(0.9, 0.95, 1.0, 0.8)
}

const ISLAND_COLORS := {
	"grass_top": Color(0.35, 0.6, 0.45),
	"grass_mid": Color(0.3, 0.5, 0.4),
	"dirt": Color(0.4, 0.35, 0.3),
	"rock": Color(0.3, 0.25, 0.25),
	"trunk": Color(0.3, 0.2, 0.25)
}

var background_layer: CanvasLayer = null
var island_root: Node2D = null
var decoration_root: Node2D = null
var collision_shapes: Array[StaticBody2D] = []

func _ready() -> void:
	_build_starting_island()

func _build_starting_island() -> void:
	# Create background layer with sky
	_create_sky_background()

	# Create the main island structure
	island_root = Node2D.new()
	island_root.name = "IslandRoot"
	island_root.position = Vector2(0, 0)
	add_child(island_root)

	# Build island terrain
	_create_island_terrain()

	# Add decorative elements
	decoration_root = Node2D.new()
	decoration_root.name = "Decorations"
	decoration_root.z_index = 1
	island_root.add_child(decoration_root)

	_create_trees()
	_create_hanging_decorations()
	_create_stars()

func _create_sky_background() -> void:
	background_layer = CanvasLayer.new()
	background_layer.name = "SkyBackground"
	background_layer.layer = -10
	add_child(background_layer)

	# Gradient sky background
	var sky_gradient: ColorRect = ColorRect.new()
	sky_gradient.name = "SkyGradient"
	sky_gradient.set_anchors_preset(Control.PRESET_FULL_RECT)

	# Create gradient shader for sky
	var gradient: Gradient = Gradient.new()
	gradient.add_point(0.0, SKY_COLORS["sky_top"])
	gradient.add_point(1.0, SKY_COLORS["sky_bottom"])

	var gradient_texture := GradientTexture2D.new()
	gradient_texture.gradient = gradient
	gradient_texture.fill_from = Vector2(0.5, 0.0)
	gradient_texture.fill_to = Vector2(0.5, 1.0)

	sky_gradient.color = SKY_COLORS["sky_top"]
	background_layer.add_child(sky_gradient)

	# Add clouds
	_create_clouds()

func _create_clouds() -> void:
	if background_layer == null:
		return

	var cloud_layer: Control = Control.new()
	cloud_layer.name = "Clouds"
	cloud_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	background_layer.add_child(cloud_layer)

	# Create several cloud sprites at different depths
	for i in range(8):
		var cloud: Sprite2D = Sprite2D.new()
		var cloud_size: Vector2i = Vector2i(randi_range(60, 120), randi_range(25, 40))
		cloud.texture = GameConstants.make_rect_texture(SKY_COLORS["cloud"], cloud_size)
		cloud.centered = true
		cloud.position = Vector2(
			randf_range(-400, 400),
			randf_range(100, 400)
		)
		cloud.modulate = Color(1, 1, 1, randf_range(0.3, 0.6))
		cloud_layer.add_child(cloud)

		# Animate clouds slowly drifting
		var tween: Tween = cloud.create_tween()
		tween.set_loops()
		var drift_amount: float = randf_range(30, 80)
		var drift_duration: float = randf_range(8.0, 15.0)
		tween.tween_property(cloud, "position:x", cloud.position.x + drift_amount, drift_duration)
		tween.tween_property(cloud, "position:x", cloud.position.x - drift_amount, drift_duration)

func _create_stars() -> void:
	if decoration_root == null:
		return

	# Create twinkling star particles
	for i in range(25):
		var star: Sprite2D = Sprite2D.new()
		star.texture = GameConstants.make_circle_texture(SKY_COLORS["star"], 2)
		star.centered = true
		star.position = Vector2(
			randf_range(-240, 240),
			randf_range(-420, -100)
		)
		star.modulate = Color(1, 1, 1, randf_range(0.4, 0.9))
		decoration_root.add_child(star)

		# Twinkle animation
		var tween: Tween = star.create_tween()
		tween.set_loops()
		var delay: float = randf_range(0, 2.0)
		tween.tween_interval(delay)
		tween.tween_property(star, "modulate:a", 0.2, randf_range(0.8, 1.5))
		tween.tween_property(star, "modulate:a", 0.9, randf_range(0.8, 1.5))

func _create_island_terrain() -> void:
	if island_root == null:
		return

	# Create central platform (the floating island base)
	var island_platform: StaticBody2D = StaticBody2D.new()
	island_platform.name = "IslandPlatform"
	island_platform.collision_layer = GameConstants.LAYER_TERRAIN
	island_platform.collision_mask = GameConstants.LAYER_PLAYER | GameConstants.LAYER_ENEMY
	island_root.add_child(island_platform)

	# Island visual sprite (grass and dirt layers)
	var island_sprite: Sprite2D = Sprite2D.new()
	island_sprite.texture = _create_island_texture()
	island_sprite.centered = true
	island_sprite.position = Vector2(0, 0)
	island_platform.add_child(island_sprite)

	# Collision shape for the island top
	var collision_shape: CollisionShape2D = CollisionShape2D.new()
	var shape: RectangleShape2D = RectangleShape2D.new()
	shape.size = Vector2(ISLAND_WIDTH, 40)
	collision_shape.shape = shape
	collision_shape.position = Vector2(0, -20)
	island_platform.add_child(collision_shape)

	collision_shapes.append(island_platform)

func _create_island_texture() -> Texture2D:
	var size: Vector2i = Vector2i(ISLAND_WIDTH, ISLAND_HEIGHT)
	var image: Image = Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)

	# Fill with layered colors to simulate grass/dirt/rock
	for x in size.x:
		for y in size.y:
			var color: Color
			if y < 15:
				# Grass top layer
				color = ISLAND_COLORS["grass_top"]
			elif y < 40:
				# Grass mid layer
				color = ISLAND_COLORS["grass_mid"]
			elif y < 80:
				# Dirt layer
				color = ISLAND_COLORS["dirt"]
			else:
				# Rock base
				color = ISLAND_COLORS["rock"]

			# Add some noise for texture variation
			var noise_factor: float = randf_range(0.9, 1.1)
			color = Color(
				color.r * noise_factor,
				color.g * noise_factor,
				color.b * noise_factor,
				color.a
			)

			image.set_pixel(x, y, color)

	return ImageTexture.create_from_image(image)

func _create_trees() -> void:
	if decoration_root == null:
		return

	# Create 3 floating island trees matching example.png style
	var tree_positions: Array[Vector2] = [
		Vector2(-100, -60),  # Left tree
		Vector2(0, -80),     # Center tree (tallest)
		Vector2(90, -55)     # Right tree
	]

	var tree_colors: Array[String] = ["pink_medium", "orange_light", "pink_light"]

	for i in range(tree_positions.size()):
		_create_tree(tree_positions[i], tree_colors[i])

func _create_tree(pos: Vector2, color_key: String) -> void:
	if decoration_root == null:
		return

	var tree_root: Node2D = Node2D.new()
	tree_root.name = "Tree"
	tree_root.position = pos
	decoration_root.add_child(tree_root)

	# Tree trunk (thin purple/dark trunk like in example.png)
	var trunk: Sprite2D = Sprite2D.new()
	trunk.texture = GameConstants.make_rect_texture(ISLAND_COLORS["trunk"], Vector2i(12, 60))
	trunk.centered = true
	trunk.position = Vector2(0, 30)
	tree_root.add_child(trunk)

	# Tree foliage (pink/orange rounded canopy)
	var foliage_size: Vector2i = Vector2i(
		randi_range(70, 100),
		randi_range(65, 90)
	)
	var foliage: Sprite2D = Sprite2D.new()
	foliage.texture = _create_foliage_texture(foliage_size, color_key)
	foliage.centered = true
	foliage.position = Vector2(0, -10)
	tree_root.add_child(foliage)

	# Add hanging roots/vines
	_create_tree_roots(tree_root, foliage_size)

	# Gentle sway animation
	var tween: Tween = tree_root.create_tween()
	tween.set_loops()
	var sway_amount: float = randf_range(2.0, 4.0)
	var sway_duration: float = randf_range(2.5, 4.0)
	tween.tween_property(tree_root, "rotation", deg_to_rad(sway_amount), sway_duration)
	tween.tween_property(tree_root, "rotation", deg_to_rad(-sway_amount), sway_duration)

func _create_foliage_texture(size: Vector2i, color_key: String) -> Texture2D:
	var image: Image = Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	var center: Vector2 = Vector2(size.x / 2.0, size.y / 2.0)
	var base_color: Color = TREE_COLORS.get(color_key, TREE_COLORS["pink_medium"])

	# Create rounded organic shape
	for x in size.x:
		for y in size.y:
			var dx: float = float(x) - center.x
			var dy: float = float(y) - center.y
			var distance: float = sqrt(dx * dx + dy * dy)
			var max_radius: float = min(size.x, size.y) * 0.45

			if distance < max_radius:
				# Create gradient from center to edge
				var gradient_factor: float = 1.0 - (distance / max_radius)
				var color: Color = base_color
				color = color.darkened(0.3 * (1.0 - gradient_factor))

				# Add some organic noise
				var noise: float = randf_range(0.92, 1.08)
				color.r *= noise
				color.g *= noise
				color.b *= noise

				image.set_pixel(x, y, color)
			else:
				image.set_pixel(x, y, Color(0, 0, 0, 0))

	return ImageTexture.create_from_image(image)

func _create_tree_roots(tree_node: Node2D, foliage_size: Vector2i) -> void:
	# Create hanging roots/vines beneath the tree (like in example.png)
	var num_roots: int = randi_range(3, 6)
	var root_spread: float = float(foliage_size.x) * 0.4

	for i in range(num_roots):
		var root: Line2D = Line2D.new()
		root.default_color = ISLAND_COLORS["trunk"].darkened(0.2)
		root.width = randf_range(1.5, 3.0)

		var start_x: float = randf_range(-root_spread, root_spread)
		var root_length: float = randf_range(40, 80)

		# Create curved root path
		root.add_point(Vector2(start_x, 20))
		root.add_point(Vector2(start_x + randf_range(-10, 10), 20 + root_length * 0.5))
		root.add_point(Vector2(start_x + randf_range(-5, 5), 20 + root_length))

		tree_node.add_child(root)

func _create_hanging_decorations() -> void:
	if decoration_root == null:
		return

	# Create small glowing orbs hanging from trees (like in example.png)
	for i in range(12):
		var orb: Sprite2D = Sprite2D.new()
		orb.texture = GameConstants.make_circle_texture(Color(1.0, 0.9, 0.6, 0.8), 4)
		orb.centered = true
		orb.position = Vector2(
			randf_range(-150, 150),
			randf_range(-20, 40)
		)
		decoration_root.add_child(orb)

		# Gentle floating animation
		var tween: Tween = orb.create_tween()
		tween.set_loops()
		var float_amount: float = randf_range(5, 12)
		var float_duration: float = randf_range(2.0, 3.5)
		tween.tween_property(orb, "position:y", orb.position.y - float_amount, float_duration)
		tween.tween_property(orb, "position:y", orb.position.y + float_amount, float_duration)

func get_spawn_position() -> Vector2:
	# Return the position where the player should spawn on the island
	return Vector2(0, -50)

func cleanup() -> void:
	for body in collision_shapes:
		if is_instance_valid(body):
			body.queue_free()
	collision_shapes.clear()
