extends CharacterBody2D
class_name Player

signal stomp(enemy: Node)
signal took_damage(amount: int, remaining: int)
signal died(result: Dictionary)
signal ammo_changed(current: int, maximum: int)
signal combo_progress(combo: int, airborne_time: float)
signal bullet_fired(bullet: Node)
signal landed()

const GRAVITY := 1600.0
const TERMINAL_VELOCITY := 1400.0
const MOVE_SPEED := 240.0
const AIR_SPEED := 220.0
const ACCEL_GROUND := 1800.0
const ACCEL_AIR := 900.0
const FRICTION := 1400.0
const JUMP_SPEED := -520.0
const GUNBOOT_IMPULSE := -280.0
const GUNBOOT_PUSH := -120.0
const STOMP_BOUNCE := -420.0
const INVULN_TIME := 1.0
const SLOWFALL_TIME := 0.15
const SLOWFALL_FACTOR := 0.4
const GUNBOOT_LIFT := -90.0
const MAX_JUMP_HOLD := 0.2
const JUMP_HOLD_BOOST := 360.0

var bullet_script: Script = preload("res://scripts/gunboot_bullet.gd")

var max_ammo: int = 8
var ammo: int = max_ammo
var gun_damage: int = 1
var gun_cooldown: float = 0.12
var shoot_cooldown: float = 0.0
var max_health: int = 4
var health: int = max_health
var combo_count: int = 0
var airborne_time: float = 0.0
var invulnerable: float = 0.0
var slowfall_timer: float = 0.0
var is_jumping: bool = false
var jump_hold_time: float = 0.0

var projectile_root: Node = null
var game_world: Node = null

var anim_sprite: AnimatedSprite2D = null
var shadow: Sprite2D = null
var collision_shape: CollisionShape2D = null
var base_color: Color = Color(0.85, 0.15, 0.25)
var shoot_timer: float = 0.0
var stomp_timer: float = 0.0
var hit_timer: float = 0.0
var current_animation: String = "idle"
var trail_root: Node2D = null
var trail_timer: float = 0.0

const TRAIL_INTERVAL := 0.045
const TRAIL_LIFETIME := 0.25

func _ready() -> void:
	collision_layer = GameConstants.LAYER_PLAYER
	collision_mask = GameConstants.LAYER_WORLD | GameConstants.LAYER_ENEMY | GameConstants.LAYER_PICKUP | GameConstants.LAYER_SHOP
	_build_visuals()
	add_to_group("player")
	emit_signal("ammo_changed", ammo, max_ammo)

func setup(world: Node, projectile_holder: Node) -> void:
	game_world = world
	projectile_root = projectile_holder

func _build_visuals() -> void:
	anim_sprite = AnimatedSprite2D.new()
	anim_sprite.name = "AnimSprite"
	anim_sprite.sprite_frames = _create_player_frames()
	anim_sprite.animation = "idle"
	anim_sprite.play()
	anim_sprite.position = Vector2(0, -12)
	add_child(anim_sprite)

	trail_root = Node2D.new()
	trail_root.z_index = -1
	add_child(trail_root)

	shadow = Sprite2D.new()
	shadow.name = "Shadow"
	shadow.texture = GameConstants.make_outline_texture(Vector2i(20, 8), Color(0, 0, 0, 0.2), Color(0, 0, 0, 0.45))
	shadow.centered = true
	shadow.position = Vector2(0, 12)
	shadow.modulate = Color(1, 1, 1, 0.7)
	add_child(shadow)

	collision_shape = CollisionShape2D.new()
	var shape: CapsuleShape2D = CapsuleShape2D.new()
	shape.radius = 8
	shape.height = 28
	collision_shape.shape = shape
	collision_shape.position = Vector2(0, -10)
	add_child(collision_shape)

func _create_player_frames() -> SpriteFrames:
	var frames: SpriteFrames = SpriteFrames.new()

	# Load the spritesheet
	var spritesheet_path: String = "res://assets/sprites/satiro-Sheet v1.1.png"
	var spritesheet: Texture2D = null
	if ResourceLoader.exists(spritesheet_path):
		spritesheet = ResourceLoader.load(spritesheet_path) as Texture2D

	if spritesheet != null:
		# Build animations from spritesheet atlas
		# Frame size is 24x24 pixels
		const FRAME_SIZE: Vector2i = Vector2i(24, 24)

		# Row 2: Idle animation (frames 8-15)
		_build_atlas_animation(frames, "idle", true, 8.0, spritesheet, FRAME_SIZE, 8, 8)

		# Row 4: Jump animation (frames 24-26 for upward motion)
		_build_atlas_animation(frames, "jump", false, 12.0, spritesheet, FRAME_SIZE, 24, 3)

		# Row 4: Fall animation (frames 29-31 for downward motion)
		_build_atlas_animation(frames, "fall", true, 10.0, spritesheet, FRAME_SIZE, 29, 3)

		# Row 6: Shoot animation (frames 40-43)
		_build_atlas_animation(frames, "shoot", false, 16.0, spritesheet, FRAME_SIZE, 40, 4)

		# Row 4: Stomp uses fall frames with shoot (frames 29-30)
		_build_atlas_animation(frames, "stomp", false, 14.0, spritesheet, FRAME_SIZE, 29, 2)

		# Row 5: Hit/Death animation (frames 32-35)
		_build_atlas_animation(frames, "hit", false, 12.0, spritesheet, FRAME_SIZE, 32, 4)
	else:
		# Fallback to placeholder animations
		_build_animation(frames, "idle", true, 10.0, _frame_paths("player_idle_", 4), Vector2i(24, 28))
		_build_animation(frames, "fall", true, 8.0, _frame_paths("player_fall_", 3), Vector2i(24, 28))
		_build_animation(frames, "jump", true, 10.0, _frame_paths("player_jump_", 2), Vector2i(24, 28))
		_build_animation(frames, "shoot", false, 14.0, _frame_paths("player_shoot_", 2), Vector2i(26, 32))
		_build_animation(frames, "stomp", false, 12.0, _frame_paths("player_stomp_", 2), Vector2i(26, 32))
		_build_animation(frames, "hit", false, 18.0, _frame_paths("player_hit_", 1), Vector2i(24, 28))

	return frames

func _frame_paths(prefix: String, count: int) -> Array[String]:
	var paths: Array[String] = []
	for i in range(count):
		paths.append("res://sprites/%s%d.png" % [prefix, i])
	return paths

func _build_atlas_animation(frames: SpriteFrames, anim_name: String, loop: bool, fps: float, spritesheet: Texture2D, frame_size: Vector2i, start_index: int, frame_count: int) -> void:
	"""
	Build animation from spritesheet atlas.
	Spritesheet is 8 frames wide, frames are indexed left-to-right, top-to-bottom.
	"""
	frames.add_animation(anim_name)
	frames.set_animation_loop(anim_name, loop)
	frames.set_animation_speed(anim_name, fps)

	const COLUMNS: int = 8
	for i in range(frame_count):
		var frame_index: int = start_index + i
		var col: int = frame_index % COLUMNS
		var row: int = int(frame_index / float(COLUMNS))

		var atlas: AtlasTexture = AtlasTexture.new()
		atlas.atlas = spritesheet
		atlas.region = Rect2(col * frame_size.x, row * frame_size.y, frame_size.x, frame_size.y)
		frames.add_frame(anim_name, atlas)

func _build_animation(frames: SpriteFrames, anim_name: String, loop: bool, fps: float, paths: Array[String], fallback_size: Vector2i, fallback_color: Color = base_color) -> void:
	frames.add_animation(anim_name)
	frames.set_animation_loop(anim_name, loop)
	frames.set_animation_speed(anim_name, fps)
	for path in paths:
		frames.add_frame(anim_name, _load_or_placeholder(path, fallback_size, fallback_color))

func _load_or_placeholder(path: String, size: Vector2i, fallback_color: Color) -> Texture2D:
	if ResourceLoader.exists(path):
		return ResourceLoader.load(path) as Texture2D
	return GameConstants.make_rect_texture(fallback_color, size)

func _physics_process(delta: float) -> void:
	invulnerable = max(0.0, invulnerable - delta)
	shoot_cooldown = max(0.0, shoot_cooldown - delta)
	slowfall_timer = max(0.0, slowfall_timer - delta)
	if shoot_timer > 0.0:
		shoot_timer = max(0.0, shoot_timer - delta)
	if stomp_timer > 0.0:
		stomp_timer = max(0.0, stomp_timer - delta)
	if hit_timer > 0.0:
		hit_timer = max(0.0, hit_timer - delta)
	_spawn_trail(delta)

	_apply_gravity(delta)
	_process_horizontal(delta)
	_handle_actions(delta)
	_apply_jump_hold(delta)

	var was_on_floor: bool = is_on_floor()
	move_and_slide()
	shadow.position.y = max(12, 12 + (global_position.y - floor(global_position.y)))
	_handle_collisions()

	if is_on_floor():
		if not was_on_floor:
			_on_landed()
		airborne_time = 0.0
	else:
		airborne_time += delta
	_update_animation_state()

	emit_signal("combo_progress", combo_count, airborne_time)

func _update_animation_state() -> void:
	if anim_sprite == null:
		return
	if hit_timer > 0.0:
		_set_animation("hit")
		return
	if stomp_timer > 0.0:
		_set_animation("stomp")
		return
	if shoot_timer > 0.0:
		_set_animation("shoot")
		return
	if not is_on_floor():
		if velocity.y < 0.0:
			_set_animation("jump")
		else:
			_set_animation("fall")
	else:
		_set_animation("idle")

func _set_animation(anim_name: String) -> void:
	if current_animation == anim_name or anim_sprite == null:
		return
	if not anim_sprite.sprite_frames.has_animation(anim_name):
		return
	current_animation = anim_name
	anim_sprite.play(anim_name)

func _apply_gravity(delta: float) -> void:
	var gravity_scale: float = 1.0
	if slowfall_timer > 0.0:
		gravity_scale = SLOWFALL_FACTOR
	if velocity.y < TERMINAL_VELOCITY:
		velocity.y = min(TERMINAL_VELOCITY, velocity.y + GRAVITY * gravity_scale * delta)

func _process_horizontal(delta: float) -> void:
	var input_dir: float = Input.get_action_strength("move_right") - Input.get_action_strength("move_left")
	var target_speed: float = input_dir * (MOVE_SPEED if is_on_floor() else AIR_SPEED)
	var accel: float = ACCEL_GROUND if is_on_floor() else ACCEL_AIR
	if input_dir == 0 and is_on_floor():
		velocity.x = move_toward(velocity.x, 0.0, FRICTION * delta)
	else:
		velocity.x = move_toward(velocity.x, target_speed, accel * delta)
	if anim_sprite != null and abs(velocity.x) > 6:
		anim_sprite.flip_h = velocity.x < 0.0

func _handle_actions(_delta: float) -> void:
	if Input.is_action_just_pressed("shoot"):
		if is_on_floor():
			_jump()
		else:
			_shoot()
	if Input.is_action_just_released("shoot"):
		is_jumping = false

func _jump() -> void:
	velocity.y = JUMP_SPEED
	is_jumping = true
	jump_hold_time = MAX_JUMP_HOLD
	combo_count = 0
	emit_signal("combo_progress", combo_count, airborne_time)
	_set_animation("jump")

func _shoot() -> void:
	if ammo <= 0 or shoot_cooldown > 0.0 or projectile_root == null:
		return
	ammo -= 1
	shoot_cooldown = gun_cooldown
	slowfall_timer = SLOWFALL_TIME
	var slowed: float = velocity.y * SLOWFALL_FACTOR
	velocity.y = min(slowed + GUNBOOT_LIFT, GUNBOOT_PUSH)
	shoot_timer = 0.18
	_set_animation("shoot")
	var bullet_instance: GunbootBullet = bullet_script.new() as GunbootBullet
	if bullet_instance != null:
		bullet_instance.global_position = global_position + Vector2(0, 14)
		bullet_instance.setup(Vector2.DOWN, gun_damage)
		projectile_root.add_child(bullet_instance)
		emit_signal("bullet_fired", bullet_instance)
	_spawn_muzzle_flash()
	emit_signal("ammo_changed", ammo, max_ammo)

func _handle_collisions() -> void:
	for collision_index in range(get_slide_collision_count()):
		var collision: KinematicCollision2D = get_slide_collision(collision_index)
		var collider: Object = collision.get_collider()
		if collider != null and collider.has_method("on_player_collision"):
			collider.on_player_collision(self, collision)

func _on_landed() -> void:
	emit_signal("landed")
	is_jumping = false
	stomp_timer = 0.0
	if combo_count > 0:
		combo_count = 0
		emit_signal("combo_progress", combo_count, airborne_time)
	_set_animation("idle")

func on_stomp_success(enemy: Node) -> void:
	combo_count += 1
	ammo = max_ammo
	emit_signal("ammo_changed", ammo, max_ammo)
	velocity.y = STOMP_BOUNCE
	stomp_timer = 0.25
	_spawn_stomp_effect()
	emit_signal("stomp", enemy)

func refill_ammo(amount: int) -> void:
	ammo = clamp(ammo + amount, 0, max_ammo)
	emit_signal("ammo_changed", ammo, max_ammo)

func take_damage(amount: int, source: Node = null) -> void:
	if invulnerable > 0.0 or amount <= 0:
		return
	health -= amount
	invulnerable = INVULN_TIME
	hit_timer = 0.25
	if anim_sprite != null:
		anim_sprite.modulate = Color(1, 0.6, 0.6)
		var tween: Tween = create_tween()
		tween.tween_property(anim_sprite, "modulate", Color(1, 1, 1), 0.25)
	_spawn_hit_particles()
	emit_signal("took_damage", amount, health)
	if health <= 0:
		_die(source)

func _die(source: Node) -> void:
	emit_signal("died", {"cause": source, "height": global_position.y})

func heal(amount: int) -> void:
	health = clamp(health + amount, 0, max_health)

func enhance_ammo(delta_ammo: int) -> void:
	max_ammo = max(1, max_ammo + delta_ammo)
	ammo = max_ammo
	emit_signal("ammo_changed", ammo, max_ammo)

func on_pickup(collected: Dictionary) -> void:
	if collected.has("health"):
		health += collected["health"]

func apply_palette(color: Color) -> void:
	base_color = color
	if anim_sprite:
		# Use modulation to tint the sprite with the palette color
		# This preserves the original sprite details while applying color
		anim_sprite.modulate = color
	_set_animation(current_animation)

func _apply_jump_hold(delta: float) -> void:
	if not is_jumping:
		return
	if Input.is_action_pressed("shoot") and jump_hold_time > 0.0:
		velocity.y -= JUMP_HOLD_BOOST * delta
		jump_hold_time = max(0.0, jump_hold_time - delta)
	else:
		is_jumping = false

func _spawn_muzzle_flash() -> void:
	if not is_inside_tree():
		return
	var flash: AnimatedSprite2D = AnimatedSprite2D.new()
	var frames: SpriteFrames = SpriteFrames.new()
	var paths: Array[String] = [
		"res://sprites/effects/muzzle_0.png",
		"res://sprites/effects/muzzle_1.png"
	]
	_build_animation(frames, "flash", false, 30.0, paths, Vector2i(20, 20), Color(1, 0.9, 0.6))
	flash.sprite_frames = frames
	flash.animation = "flash"
	flash.play()
	flash.position = Vector2(0, 6)
	add_child(flash)
	flash.animation_finished.connect(func(_anim: StringName) -> void:
		if is_instance_valid(flash):
			flash.queue_free()
	)
	var light: PointLight2D = PointLight2D.new()
	light.texture = GameConstants.make_circle_texture(Color(1, 0.9, 0.6, 0.5), 16)
	light.energy = 0.7
	light.texture_scale = 1.4
	light.position = Vector2(0, 10)
	add_child(light)
	var tween: Tween = light.create_tween()
	tween.tween_property(light, "energy", 0.0, 0.12)
	tween.tween_callback(light.queue_free)

	# Add muzzle flash particles
	_spawn_muzzle_particles()

func _spawn_stomp_effect() -> void:
	if not is_inside_tree():
		return
	var effect: AnimatedSprite2D = AnimatedSprite2D.new()
	var frames: SpriteFrames = SpriteFrames.new()
	var paths: Array[String] = [
		"res://sprites/effects/stomp_0.png",
		"res://sprites/effects/stomp_1.png"
	]
	_build_animation(frames, "impact", false, 18.0, paths, Vector2i(28, 16), Color(1, 0.85, 0.4))
	effect.sprite_frames = frames
	effect.animation = "impact"
	effect.play()
	effect.position = Vector2(0, 12)
	add_child(effect)
	effect.animation_finished.connect(func(_anim: StringName) -> void:
		if is_instance_valid(effect):
			effect.queue_free()
	)

	var light: PointLight2D = PointLight2D.new()
	light.texture = GameConstants.make_circle_texture(Color(1, 0.8, 0.4, 0.5), 20)
	light.energy = 0.6
	light.texture_scale = 1.6
	light.position = Vector2(0, 12)
	add_child(light)
	var ltween: Tween = light.create_tween()
	ltween.tween_property(light, "energy", 0.0, 0.18)
	ltween.tween_callback(light.queue_free)

func _spawn_trail(delta: float) -> void:
	if trail_root == null:
		return
	trail_timer -= delta
	var moving: bool = velocity.length() > 120.0
	if not moving or is_on_floor():
		return
	if trail_timer > 0.0:
		return
	trail_timer = TRAIL_INTERVAL
	var ghost: Sprite2D = Sprite2D.new()
	if anim_sprite != null and anim_sprite.sprite_frames != null:
		var frame_tex: Texture2D = anim_sprite.sprite_frames.get_frame_texture(anim_sprite.animation, anim_sprite.frame)
		if frame_tex != null:
			ghost.texture = frame_tex
		else:
			ghost.texture = GameConstants.make_rect_texture(base_color, Vector2i(18, 24))
	else:
		ghost.texture = GameConstants.make_rect_texture(base_color, Vector2i(18, 24))
	ghost.modulate = Color(1, 1, 1, 0.5)
	if anim_sprite != null:
		ghost.position = anim_sprite.position
		ghost.flip_h = anim_sprite.flip_h
	else:
		ghost.position = Vector2.ZERO
		ghost.flip_h = false
	trail_root.add_child(ghost)
	var tween: Tween = ghost.create_tween()
	tween.tween_property(ghost, "modulate", Color(1, 1, 1, 0), TRAIL_LIFETIME)
	tween.tween_property(ghost, "scale", ghost.scale * 1.2, TRAIL_LIFETIME)
	tween.tween_callback(ghost.queue_free)

func _spawn_muzzle_particles() -> void:
	var particles := CPUParticles2D.new()
	particles.amount = 12
	particles.lifetime = 0.3
	particles.one_shot = true
	particles.explosiveness = 0.9
	particles.gravity = Vector2(0, 200)
	particles.initial_velocity_min = 100.0
	particles.initial_velocity_max = 180.0
	particles.direction = Vector2(0, 1)
	particles.spread = 45.0
	particles.scale_amount_min = 0.6
	particles.scale_amount_max = 1.2

	# Try to load smoke sprite texture
	var smoke_tex_path := "res://assets/effects/Free Smoke Fx  Pixel 06.png"
	if ResourceLoader.exists(smoke_tex_path):
		var smoke_sheet: Texture2D = ResourceLoader.load(smoke_tex_path)
		if smoke_sheet:
			var atlas := AtlasTexture.new()
			atlas.atlas = smoke_sheet
			atlas.region = Rect2(0, 0, 88, 80)
			particles.texture = atlas
	else:
		particles.texture = GameConstants.make_circle_texture(Color(1, 0.9, 0.6, 0.8), 8)

	particles.color = Color(1, 0.95, 0.7, 1)
	var gradient := Gradient.new()
	gradient.set_color(0, Color(1, 1, 1, 1))
	gradient.set_color(1, Color(1, 1, 1, 0))
	particles.color_ramp = gradient
	particles.position = Vector2(0, 12)
	add_child(particles)
	particles.emitting = true
	particles.finished.connect(particles.queue_free)

func _spawn_hit_particles() -> void:
	var particles := CPUParticles2D.new()
	particles.amount = 20
	particles.lifetime = 0.5
	particles.one_shot = true
	particles.explosiveness = 0.85
	particles.gravity = Vector2(0, 300)
	particles.initial_velocity_min = 120.0
	particles.initial_velocity_max = 200.0
	particles.spread = 180.0
	particles.scale_amount_min = 0.5
	particles.scale_amount_max = 1.0

	# Try to load smoke sprite texture
	var smoke_tex_path := "res://assets/effects/Free Smoke Fx  Pixel 07.png"
	if ResourceLoader.exists(smoke_tex_path):
		var smoke_sheet: Texture2D = ResourceLoader.load(smoke_tex_path)
		if smoke_sheet:
			var atlas := AtlasTexture.new()
			atlas.atlas = smoke_sheet
			atlas.region = Rect2(0, 0, 88, 80)
			particles.texture = atlas
	else:
		particles.texture = GameConstants.make_circle_texture(Color(1, 0.3, 0.3, 0.8), 8)

	particles.color = Color(1, 0.4, 0.4, 1)
	var gradient := Gradient.new()
	gradient.set_color(0, Color(1, 1, 1, 1))
	gradient.set_color(1, Color(1, 1, 1, 0))
	particles.color_ramp = gradient
	particles.position = Vector2(0, -10)
	add_child(particles)
	particles.emitting = true
	particles.finished.connect(particles.queue_free)
