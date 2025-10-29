extends Node2D
class_name IntroCutscene

signal cutscene_finished()

const FALL_START_Y := -800.0
const FALL_END_Y := 0.0
const FALL_DURATION := 2.5
const CAMERA_FOLLOW_SMOOTH := 8.0
const LANDING_SHAKE_STRENGTH := 20.0
const LANDING_SHAKE_DURATION := 0.4
const FADE_IN_DURATION := 0.5

var player: Player = null
var camera: Camera2D = null
var world: Node = null
var background_layer: CanvasLayer = null
var is_playing: bool = false
var cutscene_timer: float = 0.0
var fall_timer: float = 0.0
var landing_triggered: bool = false

func setup(player_ref: Player, camera_ref: Camera2D, world_ref: Node) -> void:
	player = player_ref
	camera = camera_ref
	world = world_ref

func play_cutscene() -> void:
	if is_playing or player == null or camera == null:
		return

	is_playing = true
	cutscene_timer = 0.0
	fall_timer = 0.0
	landing_triggered = false

	# Disable player controls
	if player.has_method("set_physics_process"):
		player.set_physics_process(false)

	# Position player at the top
	player.position = Vector2(0.0, FALL_START_Y)
	player.velocity = Vector2.ZERO

	# Create fade-in overlay
	_create_fade_overlay()

	# Start the falling sequence
	_begin_fall_sequence()

func _physics_process(delta: float) -> void:
	if not is_playing:
		return

	cutscene_timer += delta
	fall_timer += delta

	# Animate the fall
	if fall_timer < FALL_DURATION and not landing_triggered:
		_update_fall_animation(delta)
	elif not landing_triggered:
		_trigger_landing()

	# Camera follows player smoothly
	if camera != null and player != null:
		var target_pos: Vector2 = player.position + Vector2(0.0, -120.0)
		camera.position = camera.position.lerp(target_pos, delta * CAMERA_FOLLOW_SMOOTH)

func _begin_fall_sequence() -> void:
	if player == null:
		return

	# Apply initial downward velocity
	var _fall_acceleration: float = (FALL_END_Y - FALL_START_Y) / (FALL_DURATION * FALL_DURATION) * 2.0
	player.velocity.y = 0.0

func _update_fall_animation(_delta: float) -> void:
	if player == null:
		return

	# Smooth fall using easing
	var t: float = fall_timer / FALL_DURATION
	var eased_t: float = _ease_in_out_cubic(t)

	# Calculate position along fall path
	var target_y: float = lerp(FALL_START_Y, FALL_END_Y, eased_t)
	player.position.y = target_y

	# Add falling velocity for physics
	var fall_speed: float = (FALL_END_Y - FALL_START_Y) / FALL_DURATION
	player.velocity.y = fall_speed

func _trigger_landing() -> void:
	if landing_triggered:
		return

	landing_triggered = true

	# Create landing impact effect
	_spawn_landing_effect()

	# Trigger screen shake
	if world != null and world.has_method("_start_screen_shake"):
		world._start_screen_shake(LANDING_SHAKE_STRENGTH, LANDING_SHAKE_DURATION)

	# Reset player physics
	if player != null:
		player.velocity = Vector2.ZERO
		player.position.y = FALL_END_Y

	# Wait a moment, then finish cutscene
	var finish_timer: SceneTreeTimer = get_tree().create_timer(0.6)
	finish_timer.timeout.connect(_finish_cutscene)

func _finish_cutscene() -> void:
	is_playing = false

	# Re-enable player controls
	if player != null and player.has_method("set_physics_process"):
		player.set_physics_process(true)

	emit_signal("cutscene_finished")

func _spawn_landing_effect() -> void:
	if player == null or not is_inside_tree():
		return

	var effect_root: Node = world if world != null else get_parent()
	if effect_root == null:
		return

	# Create dust cloud effect
	var dust: CPUParticles2D = CPUParticles2D.new()
	dust.amount = 40
	dust.lifetime = 0.8
	dust.one_shot = true
	dust.gravity = Vector2(0.0, 180.0)
	dust.initial_velocity = 220.0
	dust.initial_velocity_random = 0.5
	dust.angle = 270.0
	dust.spread = 60.0
	dust.scale_amount = Vector2(2.0, 2.0)
	dust.color = Color(0.9, 0.8, 0.7, 0.7)
	dust.position = player.global_position + Vector2(0, 16)

	if effect_root.has_node("Effects"):
		effect_root.get_node("Effects").add_child(dust)
	else:
		effect_root.add_child(dust)

	dust.emitting = true
	dust.finished.connect(dust.queue_free)

	# Create radial impact wave
	_spawn_impact_wave(player.global_position + Vector2(0, 16))

func _spawn_impact_wave(pos: Vector2) -> void:
	if not is_inside_tree():
		return

	var effect_root: Node = world if world != null else get_parent()
	if effect_root == null:
		return

	var wave: Sprite2D = Sprite2D.new()
	wave.texture = GameConstants.make_circle_texture(Color(1, 0.9, 0.7, 0.5), 32)
	wave.position = pos
	wave.scale = Vector2.ZERO

	if effect_root.has_node("Effects"):
		effect_root.get_node("Effects").add_child(wave)
	else:
		effect_root.add_child(wave)

	var tween: Tween = wave.create_tween()
	tween.set_parallel(true)
	tween.tween_property(wave, "scale", Vector2(3.0, 3.0), 0.5)
	tween.tween_property(wave, "modulate", Color(1, 1, 1, 0), 0.5)
	tween.chain().tween_callback(wave.queue_free)

func _create_fade_overlay() -> void:
	if not is_inside_tree():
		return

	var fade_layer: CanvasLayer = CanvasLayer.new()
	fade_layer.layer = 100
	add_child(fade_layer)

	var fade_rect: ColorRect = ColorRect.new()
	fade_rect.color = Color.BLACK
	fade_rect.set_anchors_preset(Control.PRESET_FULL_RECT)
	fade_layer.add_child(fade_rect)

	# Fade from black to transparent
	var tween: Tween = fade_rect.create_tween()
	tween.tween_property(fade_rect, "modulate", Color(1, 1, 1, 0), FADE_IN_DURATION)
	tween.tween_callback(fade_layer.queue_free)

func _ease_in_out_cubic(t: float) -> float:
	if t < 0.5:
		return 4.0 * t * t * t
	else:
		var f: float = 2.0 * t - 2.0
		return 0.5 * f * f * f + 1.0

func skip_cutscene() -> void:
	if not is_playing:
		return

	landing_triggered = true
	fall_timer = FALL_DURATION

	if player != null:
		player.position.y = FALL_END_Y
		player.velocity = Vector2.ZERO

	_finish_cutscene()
