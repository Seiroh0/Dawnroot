extends EnemyBase
class_name GroundEnemy

const GRAVITY := 1300.0
const TERMINAL := 1200.0

var move_speed := 90.0
var direction := 1.0
var sprite_frames: SpriteFrames

func _ready() -> void:
	max_health = 1
	contact_damage = 1
	stompable = true
	stomp_resistance = 1
	gem_drop_range = Vector2i(1, 3)
	stomp_bounce = -420.0
	score_reward = 80
	super()
	_build_visuals()

func _build_visuals() -> void:
	sprite_frames = _create_frames()
	anim_sprite = AnimatedSprite2D.new()
	anim_sprite.name = "AnimSprite"
	anim_sprite.sprite_frames = sprite_frames
	anim_sprite.animation = "walk"
	anim_sprite.play()
	anim_sprite.position = Vector2(0, -12)
	add_child(anim_sprite)
	_update_palette()

	collision_shape = CollisionShape2D.new()
	var shape := CapsuleShape2D.new()
	shape.radius = 9
	shape.height = 22
	collision_shape.shape = shape
	collision_shape.position = Vector2(0, -11)
	add_child(collision_shape)

func _physics_process(delta: float) -> void:
	velocity.y = min(TERMINAL, velocity.y + GRAVITY * delta)
	velocity.x = direction * move_speed
	move_and_slide()
	if is_on_wall():
		direction *= -1.0
		velocity.x = direction * move_speed
	if anim_sprite:
		anim_sprite.flip_h = direction < 0.0

func configure_palette(palette_data: Dictionary) -> void:
	super.configure_palette(palette_data)
	_update_palette()

func _update_palette() -> void:
	if anim_sprite:
		anim_sprite.modulate = palette.get("primary", Color(0.85, 0.3, 0.35))

func set_spawn_params(params: Dictionary) -> void:
	direction = params.get("dir", direction)
	move_speed = params.get("speed", move_speed)
	if anim_sprite:
		anim_sprite.flip_h = direction < 0.0

func _create_frames() -> SpriteFrames:
	var frames := SpriteFrames.new()
	_add_animation(frames, "walk", true, 8.0, _frame_paths("enemy_ground_walk_", 4), Vector2i(24, 24))
	death_frames = _load_textures(_frame_paths("enemy_ground_death_", 3), Vector2i(24, 24))
	return frames

func _frame_paths(prefix: String, count: int) -> Array[String]:
	var paths: Array[String] = []
	for i in count:
		paths.append("res://sprites/%s%d.png" % [prefix, i])
	return paths

func _add_animation(frames: SpriteFrames, anim_name: String, loop: bool, fps: float, paths: Array[String], fallback_size: Vector2i) -> void:
	frames.add_animation(anim_name)
	frames.set_animation_loop(anim_name, loop)
	frames.set_animation_speed(anim_name, fps)
	for path in paths:
		frames.add_frame(anim_name, _load_texture(path, fallback_size))

func _load_texture(path: String, size: Vector2i) -> Texture2D:
	if ResourceLoader.exists(path):
		return ResourceLoader.load(path)
	return GameConstants.make_rect_texture(palette.get("primary", Color(0.85, 0.3, 0.35)), size)

func _load_textures(paths: Array[String], size: Vector2i) -> Array[Texture2D]:
	var textures: Array[Texture2D] = []
	for path in paths:
		textures.append(_load_texture(path, size))
	return textures
