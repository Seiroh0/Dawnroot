extends EnemyBase
class_name TurretEnemy

const ProjectileScene := preload("res://scripts/enemies/turret_projectile.gd")

var fire_interval := 1.6
var fire_timer := 0.0
var projectile_speed := 420.0
var muzzle_offset := Vector2(0, -16)
var sprite_frames: SpriteFrames
var attack_reset_timer := 0.0

func _ready() -> void:
	max_health = 2
	contact_damage = 1
	stompable = true
	gem_drop_range = Vector2i(3, 5)
	stomp_bounce = -620.0
	score_reward = 130
	super()
	_build_visuals()
	fire_timer = fire_interval * randf_range(0.3, 0.9)

func _physics_process(delta: float) -> void:
	fire_timer -= delta
	if fire_timer <= 0.0:
		fire_timer = fire_interval + randf_range(-0.2, 0.2)
		_fire_projectile()
	if attack_reset_timer > 0.0:
		attack_reset_timer -= delta
		if attack_reset_timer <= 0.0 and anim_sprite:
			anim_sprite.play("idle")

func _build_visuals() -> void:
	sprite_frames = _create_frames()
	anim_sprite = AnimatedSprite2D.new()
	anim_sprite.name = "AnimSprite"
	anim_sprite.sprite_frames = sprite_frames
	anim_sprite.animation = "idle"
	anim_sprite.play()
	anim_sprite.position = Vector2(0, -12)
	add_child(anim_sprite)
	_update_palette()

	collision_shape = CollisionShape2D.new()
	var shape := RectangleShape2D.new()
	shape.size = Vector2(24, 24)
	collision_shape.shape = shape
	collision_shape.position = Vector2(0, -12)
	add_child(collision_shape)

func configure_palette(palette_data: Dictionary) -> void:
	super.configure_palette(palette_data)
	_update_palette()

func _update_palette() -> void:
	if anim_sprite:
		anim_sprite.modulate = palette.get("primary", Color(0.3, 0.3, 0.45))

func _fire_projectile() -> void:
	if not is_inside_tree():
		return
	var projectile: TurretProjectile = ProjectileScene.new()
	projectile.setup(Vector2.UP, projectile_speed)
	projectile.global_position = global_position + muzzle_offset
	var parent := get_parent()
	if parent:
		parent.add_child(projectile)
	else:
		get_tree().current_scene.add_child(projectile)
	if anim_sprite:
		anim_sprite.play("attack")
		var frames_count: int = max(1, anim_sprite.sprite_frames.get_frame_count("attack"))
		var speed: float = max(1.0, anim_sprite.sprite_frames.get_animation_speed("attack"))
		attack_reset_timer = frames_count / speed

func _handle_stomp(player: Player) -> void:
	if player.has_method("on_stomp_success"):
		player.on_stomp_success(self)
	player.velocity.y = stomp_bounce
	apply_damage(max_health, player)

func set_spawn_params(params: Dictionary) -> void:
	fire_interval = clamp(params.get("interval", fire_interval), 0.6, 2.5)
	projectile_speed = params.get("projectile_speed", projectile_speed)

func _create_frames() -> SpriteFrames:
	var frames := SpriteFrames.new()
	_add_animation(frames, "idle", true, 6.0, _frame_paths("enemy_turret_idle_", 2), Vector2i(28, 24))
	_add_animation(frames, "attack", false, 18.0, _frame_paths("enemy_turret_attack_", 2), Vector2i(28, 24))
	death_frames = _load_textures(_frame_paths("enemy_turret_death_", 3), Vector2i(28, 24))
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
	return GameConstants.make_rect_texture(palette.get("primary", Color(0.3, 0.3, 0.45)), size)

func _load_textures(paths: Array[String], size: Vector2i) -> Array[Texture2D]:
	var textures: Array[Texture2D] = []
	for path in paths:
		textures.append(_load_texture(path, size))
	return textures
