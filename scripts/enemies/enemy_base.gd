extends CharacterBody2D
class_name EnemyBase

signal defeated(enemy: EnemyBase, position: Vector2)
signal spawn_gems(position: Vector2, amount: int)

var max_health := 1
var health := 1
var contact_damage := 1
var stompable := true
var stomp_bounce := -420.0
var stomp_resistance := 1
var gem_drop_range := Vector2i(1, 3)
var score_reward := 90
var palette := {
	"primary": Color(0.85, 0.3, 0.35),
	"outline": Color(0.2, 0.05, 0.12),
	"accent": Color(1, 0.9, 0.9)
}

var anim_sprite: AnimatedSprite2D
var collision_shape: CollisionShape2D
var hit_tween: Tween
var death_frames: Array[Texture2D] = []
var death_fps := 12.0

func _ready() -> void:
	collision_layer = GameConstants.LAYER_ENEMY
	collision_mask = GameConstants.LAYER_TERRAIN | GameConstants.LAYER_PLAYER | GameConstants.LAYER_PLAYER_SHOT
	health = max_health
	add_to_group("enemies")

func configure_palette(palette_data: Dictionary) -> void:
	for key in palette_data.keys():
		palette[key] = palette_data[key]
	_update_palette()

func apply_damage(amount: int, source: Node = null) -> void:
	health -= max(1, amount)
	_flash()
	if health <= 0:
		_die(source)

func on_player_collision(player: Player, collision: KinematicCollision2D) -> void:
	if not player:
		return
	var is_stomp := stompable and collision.get_normal().y > 0.2 and player.velocity.y > 0.0
	if is_stomp:
		_handle_stomp(player)
	else:
		if player.has_method("take_damage"):
			player.take_damage(contact_damage, self)

func hit_by_bullet(_bullet: Node, damage: int) -> void:
	apply_damage(damage, _bullet)

func _handle_stomp(player: Player) -> void:
	if player.has_method("on_stomp_success"):
		player.on_stomp_success(self)
	player.velocity.y = stomp_bounce
	stomp_resistance -= 1
	if stomp_resistance <= 0:
		_die(player)
	else:
		_flash_stomp()

func _die(_source: Node = null) -> void:
	if hit_tween:
		hit_tween.kill()
	emit_signal("defeated", self, global_position)
	emit_signal("spawn_gems", global_position, _roll_gems())
	_spawn_death_effect()
	queue_free()

func _roll_gems() -> int:
	return rngi(gem_drop_range.x, gem_drop_range.y)

func _apply_palette() -> void:
	if anim_sprite:
		anim_sprite.modulate = Color(1, 1, 1)

func _flash() -> void:
	var node: CanvasItem = anim_sprite if anim_sprite else self
	if node is CanvasItem:
		if hit_tween:
			hit_tween.kill()
		hit_tween = create_tween()
		hit_tween.tween_property(node, "modulate", Color(1, 0.8, 0.8), 0.05)
		hit_tween.tween_property(node, "modulate", Color(1, 1, 1), 0.2)

func _flash_stomp() -> void:
	if anim_sprite:
		var tween := create_tween()
		tween.tween_property(anim_sprite, "scale", anim_sprite.scale * 0.85, 0.08)
		tween.tween_property(anim_sprite, "scale", Vector2.ONE, 0.12)

func _spawn_death_effect() -> void:
	if not is_inside_tree():
		return
	var parent := get_parent()
	if parent == null:
		return

	# Animated sprite death effect
	var effect := AnimatedSprite2D.new()
	var frames := SpriteFrames.new()
	frames.add_animation("death")
	frames.set_animation_loop("death", false)
	frames.set_animation_speed("death", death_fps)
	if death_frames.is_empty():
		for i in range(3):
			var tex := GameConstants.make_rect_texture(palette.get("accent", Color(1, 0.9, 0.9)), Vector2i(24, 24))
			frames.add_frame("death", tex)
	else:
		for tex in death_frames:
			frames.add_frame("death", tex)

	effect.sprite_frames = frames
	effect.animation = "death"
	effect.global_position = global_position
	parent.add_child(effect)
	effect.play("death")
	var tween := effect.create_tween()
	tween.tween_property(effect, "modulate", Color(1, 1, 1, 0), 0.4)
	tween.tween_callback(effect.queue_free)

	# Add smoke particle effect
	_spawn_smoke_particles(parent)

func rngi(min_value: int, max_value: int) -> int:
	return randi_range(min_value, max_value)

func _spawn_smoke_particles(parent: Node) -> void:
	var particles := CPUParticles2D.new()
	particles.amount = 16
	particles.lifetime = 0.6
	particles.one_shot = true
	particles.explosiveness = 0.8
	particles.gravity = Vector2(0, -80)
	particles.initial_velocity_min = 80.0
	particles.initial_velocity_max = 160.0
	particles.angular_velocity_min = -180.0
	particles.angular_velocity_max = 180.0
	particles.radial_accel_min = -40.0
	particles.radial_accel_max = 40.0
	particles.spread = 180.0
	particles.scale_amount_min = 0.8
	particles.scale_amount_max = 1.5
	particles.scale_amount_curve = _create_scale_curve()

	# Try to load smoke sprite texture
	var smoke_tex_path := "res://assets/effects/Free Smoke Fx  Pixel 05.png"
	if ResourceLoader.exists(smoke_tex_path):
		var smoke_sheet: Texture2D = ResourceLoader.load(smoke_tex_path)
		if smoke_sheet:
			# Create atlas texture for first frame (spritesheets are 704x960, 8x12 frames of 88x80)
			var atlas := AtlasTexture.new()
			atlas.atlas = smoke_sheet
			atlas.region = Rect2(0, 0, 88, 80)
			particles.texture = atlas
	else:
		particles.texture = GameConstants.make_circle_texture(Color(0.8, 0.8, 0.8, 0.6), 12)

	particles.color = palette.get("accent", Color(1, 0.9, 0.9))
	particles.color_ramp = _create_color_ramp()
	particles.global_position = global_position
	parent.add_child(particles)
	particles.emitting = true
	particles.finished.connect(particles.queue_free)

func _create_scale_curve() -> Curve:
	var curve := Curve.new()
	curve.add_point(Vector2(0.0, 0.5))
	curve.add_point(Vector2(0.5, 1.0))
	curve.add_point(Vector2(1.0, 0.2))
	return curve

func _create_color_ramp() -> Gradient:
	var gradient := Gradient.new()
	gradient.set_color(0, Color(1, 1, 1, 1))
	gradient.set_color(1, Color(1, 1, 1, 0))
	return gradient

func _update_palette() -> void:
	_apply_palette()
