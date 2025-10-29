extends EnemyBase
class_name FlyingEnemy

var amplitude := 60.0
var wave_speed := 1.2
var vertical_speed := 160.0
var phase := 0.0
var sprite_frames: SpriteFrames

func _ready() -> void:
	stompable = false
	max_health = 2
	contact_damage = 1
	gem_drop_range = Vector2i(2, 4)
	stomp_resistance = 1
	score_reward = 110
	super()
	_build_visuals()

func _build_visuals() -> void:
	sprite_frames = _create_frames()
	anim_sprite = AnimatedSprite2D.new()
	anim_sprite.name = "AnimSprite"
	anim_sprite.sprite_frames = sprite_frames
	anim_sprite.animation = "flutter"
	anim_sprite.play()
	add_child(anim_sprite)
	_update_palette()

	collision_shape = CollisionShape2D.new()
	var shape := CircleShape2D.new()
	shape.radius = 12
	collision_shape.shape = shape
	add_child(collision_shape)

func _physics_process(delta: float) -> void:
	phase += wave_speed * delta
	var horizontal_velocity := cos(phase) * amplitude * wave_speed
	var vertical_velocity := vertical_speed + sin(phase * 0.6) * amplitude * 0.4
	velocity = Vector2(horizontal_velocity, vertical_velocity)
	move_and_slide()

func configure_palette(palette_data: Dictionary) -> void:
	super.configure_palette(palette_data)
	_update_palette()

func _update_palette() -> void:
	if anim_sprite:
		anim_sprite.modulate = palette.get("primary", Color(0.7, 0.35, 0.6))

func set_spawn_params(params: Dictionary) -> void:
	amplitude = params.get("amp", amplitude)
	wave_speed = params.get("speed", wave_speed)
	vertical_speed = params.get("fall", vertical_speed)

func _create_frames() -> SpriteFrames:
	var frames := SpriteFrames.new()
	_add_animation(frames, "flutter", true, 12.0, _frame_paths("enemy_flying_flutter_", 4), Vector2i(28, 24))
	death_frames = _load_textures(_frame_paths("enemy_flying_death_", 4), Vector2i(26, 22))
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
	return GameConstants.make_circle_texture(palette.get("primary", Color(0.7, 0.35, 0.6)), int(size.x / 2.0))

func _load_textures(paths: Array[String], size: Vector2i) -> Array[Texture2D]:
	var textures: Array[Texture2D] = []
	for path in paths:
		if ResourceLoader.exists(path):
			textures.append(ResourceLoader.load(path))
		else:
			textures.append(GameConstants.make_rect_texture(palette.get("accent", Color(1, 0.9, 0.9)), size))
	return textures
