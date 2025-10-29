extends Node2D
class_name GameWorld

signal run_finished(result: Dictionary)
signal request_new_run()
signal shop_requested(shop_state: Dictionary)
signal meta_event(event_data: Dictionary)

const CAMERA_LIMIT_BOTTOM := 999999
const CAMERA_LIMIT_LEFT := -240
const CAMERA_LIMIT_RIGHT := 240
const CAMERA_LIMIT_TOP := -420
const CAMERA_SMOOTH_SPEED := 6.0
const SCREEN_SHAKE_FALLOFF := 12.0
const SCREEN_SHAKE_VERTICAL_LIMIT := 0.25

var player_script: Script = preload("res://scripts/player.gd")
var level_generator_script: Script = preload("res://scripts/level_generator.gd")
var hud_script: Script = preload("res://scripts/game_hud.gd")
var shop_ui_script: Script = preload("res://scripts/shop_ui.gd")
var gem_script: Script = preload("res://scripts/gem.gd")

var level_root: Node2D = null
var projectile_root: Node2D = null
var effect_root: Node2D = null
var player: Player = null
var camera: Camera2D = null
var level_generator: LevelGenerator = null
var hud: GameHud = null
var shop_ui: ShopUI = null
var background_layer: CanvasLayer = null
var background_rect: ColorRect = null

var run_score: int = 0
var gems: int = 0
var depth: float = 0.0
var run_time: float = 0.0
var current_combo: int = 0
var combo_multiplier: float = 1.0
var max_combo: int = 0
var combo_decay: float = 1.2
var combo_timer: float = 0.0
var run_active: bool = true
var shop_open: bool = false

var unlocked_palettes: Array[String] = ["DEFAULT"]
var current_palette: String = "DEFAULT"

var upgrade_state: Dictionary = {
	"gun_damage": 1,
	"combo_bonus": 0.0,
	"max_health": 4,
	"purchased": []
}

var palette_data: Dictionary = {
	"DEFAULT": {
		"background": Color(0.05, 0.05, 0.08),
		"platform": Color(0.25, 0.22, 0.3),
		"one_way": Color(0.32, 0.28, 0.38),
		"wall": Color(0.1, 0.08, 0.12),
		"block": Color(0.35, 0.32, 0.4),
		"hazard": Color(0.9, 0.25, 0.3),
		"gem": Color(0.6, 1.0, 0.85),
		"enemy": Color(0.85, 0.3, 0.35),
		"player": Color(0.85, 0.15, 0.25),
		"shop_text": Color(1, 0.95, 0.85)
	},
	"SUNSET": {
		"background": Color(0.12, 0.04, 0.08),
		"platform": Color(0.45, 0.2, 0.18),
		"one_way": Color(0.55, 0.28, 0.22),
		"wall": Color(0.2, 0.12, 0.1),
		"block": Color(0.55, 0.24, 0.2),
		"hazard": Color(1.0, 0.45, 0.3),
		"gem": Color(1.0, 0.8, 0.45),
		"enemy": Color(0.95, 0.4, 0.35),
		"player": Color(1.0, 0.5, 0.25),
		"shop_text": Color(1.0, 0.85, 0.7)
	},
	"MINT": {
		"background": Color(0.02, 0.09, 0.08),
		"platform": Color(0.1, 0.25, 0.24),
		"one_way": Color(0.14, 0.32, 0.3),
		"wall": Color(0.06, 0.18, 0.16),
		"block": Color(0.12, 0.3, 0.28),
		"hazard": Color(0.75, 0.25, 0.35),
		"gem": Color(0.7, 1.0, 0.8),
		"enemy": Color(0.7, 0.2, 0.3),
		"player": Color(0.9, 0.3, 0.4),
		"shop_text": Color(0.8, 1.0, 0.9)
	},
	"NEON": {
		"background": Color(0.02, 0.02, 0.08),
		"platform": Color(0.15, 0.08, 0.4),
		"one_way": Color(0.24, 0.16, 0.55),
		"wall": Color(0.08, 0.05, 0.25),
		"block": Color(0.2, 0.1, 0.45),
		"hazard": Color(1.0, 0.25, 0.7),
		"gem": Color(0.4, 1.0, 0.9),
		"enemy": Color(0.9, 0.2, 0.7),
		"player": Color(0.3, 0.9, 0.8),
		"shop_text": Color(0.7, 0.9, 1.0)
	}
}

var palette_unlocks: Dictionary = {
	"SUNSET": 6,
	"MINT": 10,
	"NEON": 16
}

var rng: RandomNumberGenerator = RandomNumberGenerator.new()
var shake_timer: float = 0.0
var shake_strength: float = 0.0
var freeze_timer: SceneTreeTimer = null
var audio_players: Dictionary = {}
var background_music_player: AudioStreamPlayer = null
var audio_profiles: Dictionary = {
	"stomp": 220.0,
	"damage": 140.0,
	"gem": 420.0,
	"block": 260.0,
	"shot": 520.0
}

func _ready() -> void:
	rng.randomize()
	_build_world()

func _build_world() -> void:
	Engine.time_scale = 1.0

	background_layer = CanvasLayer.new()
	background_layer.layer = -20
	add_child(background_layer)

	var palette: Dictionary = _current_palette_data()
	var background: ColorRect = ColorRect.new()
	background.name = "Background"
	background.color = palette.get("background", Color(0.05, 0.05, 0.08))
	background.set_anchors_preset(Control.PRESET_FULL_RECT)
	background_layer.add_child(background)
	background_rect = background

	level_root = Node2D.new()
	level_root.name = "LevelRoot"
	add_child(level_root)

	projectile_root = Node2D.new()
	projectile_root.name = "Projectiles"
	projectile_root.z_index = 10
	level_root.add_child(projectile_root)

	effect_root = Node2D.new()
	effect_root.name = "Effects"
	effect_root.z_index = 20
	level_root.add_child(effect_root)

	player = player_script.new() as Player
	if player == null:
		push_error("Player script failed to instantiate.")
		return
	player.name = "Player"
	player.position = Vector2(0.0, 120.0)
	level_root.add_child(player)
	player.setup(self, projectile_root)
	player.max_health = int(upgrade_state.get("max_health", player.max_health))
	player.health = player.max_health
	player.gun_damage = int(upgrade_state.get("gun_damage", player.gun_damage))
	player.stomp.connect(Callable(self, "_on_player_stomp"))
	player.bullet_fired.connect(Callable(self, "_on_player_bullet"))
	player.took_damage.connect(Callable(self, "_on_player_took_damage"))
	player.died.connect(Callable(self, "_on_player_died"))
	player.ammo_changed.connect(Callable(self, "_on_player_ammo_changed"))
	player.combo_progress.connect(Callable(self, "_on_player_combo_progress"))
	player.landed.connect(Callable(self, "_on_player_landed"))

	camera = Camera2D.new()
	camera.name = "GameCamera"
	camera.position = player.position
	camera.enabled = true
	camera.limit_left = CAMERA_LIMIT_LEFT
	camera.limit_right = CAMERA_LIMIT_RIGHT
	camera.limit_top = CAMERA_LIMIT_TOP
	camera.limit_bottom = CAMERA_LIMIT_BOTTOM
	camera.position_smoothing_enabled = true
	camera.position_smoothing_speed = CAMERA_SMOOTH_SPEED
	level_root.add_child(camera)

	# === DOWNROOT INTEGRATION ===
	var integration_script: Script = preload("res://scripts/downroot_integration.gd")
	if integration_script != null:
		var downroot_integration: DownrootIntegration = integration_script.new()
		if downroot_integration != null:
			downroot_integration.setup_downroot_experience(self, player, camera, level_root)
			DownrootIntegration.create_simple_sky_background(background_layer)
	# === END DOWNROOT INTEGRATION ===

	level_generator = level_generator_script.new() as LevelGenerator
	if level_generator == null:
		push_error("LevelGenerator script failed to instantiate.")
		return
	level_generator.name = "LevelGenerator"
	level_root.add_child(level_generator)
	level_generator.set_palette(palette)
	level_generator.setup(self, player)

	hud = hud_script.new() as GameHud
	if hud == null:
		push_error("HUD script failed to instantiate.")
		return
	add_child(hud)
	hud.set_palette(palette)
	combo_multiplier = _combo_multiplier_value(current_combo)
	hud.update_score(run_score, gems)
	hud.update_ammo(player.ammo, player.max_ammo)
	hud.update_combo(current_combo, combo_multiplier)
	hud.update_health(player.health)
	hud.update_depth(depth)

	shop_ui = shop_ui_script.new() as ShopUI
	if shop_ui == null:
		push_error("Shop UI script failed to instantiate.")
		return
	add_child(shop_ui)
	shop_ui.purchase_selected.connect(Callable(self, "_on_shop_option_selected"))
	shop_ui.closed.connect(Callable(self, "_on_shop_closed"))

	_setup_audio()
	_setup_background_music()
	_apply_palette()

func load_meta_progress(meta: Dictionary) -> void:
	unlocked_palettes = meta.get("unlocked_palettes", unlocked_palettes)
	if unlocked_palettes.is_empty():
		unlocked_palettes = ["DEFAULT"]
	if meta.has("preferred_palette"):
		var desired: String = meta["preferred_palette"]
		if unlocked_palettes.has(desired):
			current_palette = desired
	_apply_palette()

func _current_palette_data() -> Dictionary:
	return palette_data.get(current_palette, palette_data["DEFAULT"])

func _apply_palette() -> void:
	var palette: Dictionary = _current_palette_data()
	if background_rect != null:
		background_rect.color = palette.get("background", Color(0.05, 0.05, 0.08))
	if player != null:
		player.apply_palette(palette.get("player", Color(0.85, 0.15, 0.25)))
	if level_generator != null:
		level_generator.set_palette(palette)
	if hud != null:
		hud.set_palette(palette)

func _physics_process(delta: float) -> void:
	if not run_active:
		return
	run_time += delta
	if level_generator != null:
		level_generator.update_chunks()
	_update_camera_position(delta)
	_update_depth()
	_update_shake(delta)
	if current_combo > 0:
		combo_timer = max(0.0, combo_timer - delta)
		if combo_timer <= 0.0:
			_reset_combo()

func _update_camera_position(delta: float) -> void:
	if camera == null or player == null:
		return
	var target_position: Vector2 = player.position + Vector2(0.0, -120.0)
	camera.position = camera.position.lerp(target_position, clamp(delta * 5.0, 0.0, 1.0))

func _update_depth() -> void:
	if player == null:
		return
	depth = max(depth, player.global_position.y)
	if hud != null:
		hud.update_depth(depth)

func _combo_multiplier_value(count: int) -> float:
	var base: float = 1.0
	if count >= 20:
		base = 2.0
	elif count >= 10:
		base = 1.5
	var bonus: float = float(upgrade_state.get("combo_bonus", 0.0))
	return max(1.0, base + bonus)

func _on_player_stomp(_enemy: Node) -> void:
	if player == null:
		return
	var stomp_combo: int = player.combo_count
	current_combo = stomp_combo
	max_combo = max(max_combo, current_combo)
	combo_multiplier = _combo_multiplier_value(current_combo)
	if hud != null:
		hud.update_combo(current_combo, combo_multiplier)
	run_score += int(round(120.0 * combo_multiplier))
	if hud != null:
		hud.update_score(run_score, gems)
	combo_timer = combo_decay
	_start_screen_shake(12.0, 0.2)
	_trigger_time_freeze(0.05, 0.2)
	_play_sound("stomp")

func _on_player_combo_progress(combo: int, _airborne_time: float) -> void:
	current_combo = combo
	max_combo = max(max_combo, current_combo)
	combo_multiplier = _combo_multiplier_value(current_combo)
	if hud != null:
		hud.update_combo(current_combo, combo_multiplier)
	combo_timer = combo_decay

func _on_player_landed() -> void:
	_reset_combo()

func _reset_combo() -> void:
	current_combo = 0
	combo_multiplier = _combo_multiplier_value(0)
	if hud != null:
		hud.update_combo(current_combo, combo_multiplier)
	combo_timer = 0.0

func _on_player_took_damage(_amount: int, remaining_health: int) -> void:
	if hud != null:
		hud.update_health(max(remaining_health, 0))
		hud.show_message("Treffer!", 0.6)
	_start_screen_shake(14.0, 0.3)
	_play_sound("damage")

func _on_player_ammo_changed(current: int, maximum: int) -> void:
	if hud != null:
		hud.update_ammo(current, maximum)

func _on_player_bullet(_bullet: Node) -> void:
	_play_sound("shot")

func _on_player_died(_result: Dictionary) -> void:
	run_active = false
	var summary: Dictionary = {
		"score": run_score,
		"max_combo": max_combo,
		"depth": depth,
		"time": run_time,
		"gems": gems
	}
	if hud != null:
		hud.show_message("Run vorbei!", 2.0)
	if background_music_player != null and background_music_player.playing:
		background_music_player.stop()
	_check_palette_unlocks()
	emit_signal("run_finished", summary)
	var restart_timer: SceneTreeTimer = get_tree().create_timer(2.0)
	restart_timer.timeout.connect(Callable(self, "_emit_run_restart"))

func _emit_run_restart() -> void:
	emit_signal("request_new_run")

func _on_enemy_defeated(enemy: EnemyBase, pos: Vector2) -> void:
	var base_score: int = 90
	var base_gems: int = 4
	if enemy != null:
		base_score = enemy.score_reward
		if enemy.gem_drop_range is Vector2i:
			var gem_range: Vector2i = enemy.gem_drop_range
			base_gems = int(round((gem_range.x + gem_range.y) * 0.5))
	run_score += int(round(float(base_score) * combo_multiplier))
	if hud != null:
		hud.update_score(run_score, gems)
	_spawn_reward_gem(pos, base_gems)

func _on_block_broken(_block: DestructibleBlock, pos: Vector2) -> void:
	run_score += int(round(40.0 * combo_multiplier))
	if hud != null:
		hud.update_score(run_score, gems)
	_spawn_reward_gem(pos, 6)
	_play_sound("block")

func _spawn_reward_gem(pos: Vector2, value: int) -> void:
	if level_root == null:
		return
	var gem_instance: Gem = gem_script.new() as Gem
	if gem_instance == null:
		return
	var gem_data: Dictionary = {
		"value": max(1, int(round(value * combo_multiplier))),
		"color": _current_palette_data().get("gem", Color(0.6, 1.0, 0.85))
	}
	gem_instance.configure(gem_data)
	gem_instance.global_position = pos
	gem_instance.collected.connect(Callable(self, "_on_gem_collected"))
	level_root.add_child(gem_instance)

func _on_gem_collected(payload: Dictionary, _pos: Vector2) -> void:
	var gained: int = payload.get("gems", 0)
	if gained > 0:
		gems += gained
		run_score += int(round(float(gained) * combo_multiplier))
		_spawn_gem_collect_effect(player.global_position if player != null else Vector2.ZERO, gained)
	if payload.get("health", 0) > 0 and player != null:
		player.heal(payload["health"])
	if hud != null:
		hud.update_score(run_score, gems)
		hud.update_health(player.health)
	_play_sound("gem")

func _spawn_gem_collect_effect(world_position: Vector2, _amount: int) -> void:
	if effect_root == null:
		return
	var effect: AnimatedSprite2D = AnimatedSprite2D.new()
	var frames: SpriteFrames = SpriteFrames.new()
	var palette: Dictionary = _current_palette_data()
	var color: Color = palette.get("gem", Color(0.6, 1.0, 0.85))
	var paths: Array[String] = [
		"res://sprites/effects/gem_collect_0.png",
		"res://sprites/effects/gem_collect_1.png",
		"res://sprites/effects/gem_collect_2.png"
	]
	frames.add_animation("spark")
	frames.set_animation_loop("spark", false)
	frames.set_animation_speed("spark", 24.0)
	for path in paths:
		if ResourceLoader.exists(path):
			frames.add_frame("spark", ResourceLoader.load(path) as Texture2D)
		else:
			frames.add_frame("spark", GameConstants.make_rect_texture(color, Vector2i(16, 16)))
	effect.sprite_frames = frames
	effect.animation = "spark"
	effect.play()
	effect.global_position = world_position
	effect_root.add_child(effect)
	var tween: Tween = effect.create_tween()
	tween.tween_property(effect, "global_position", world_position + Vector2(0.0, -60.0), 0.35)
	tween.parallel().tween_property(effect, "modulate", Color(1, 1, 1, 0), 0.35)
	tween.tween_callback(effect.queue_free)
	_spawn_gem_particles(world_position, color)

func _spawn_gem_particles(pos: Vector2, color: Color) -> void:
	if effect_root == null:
		return
	var particles: CPUParticles2D = CPUParticles2D.new()
	particles.amount = 24
	particles.lifetime = 0.4
	particles.one_shot = true
	particles.gravity = Vector2(0.0, 220.0)
	particles.initial_velocity = 160.0
	particles.spread = 120.0
	particles.scale_amount = Vector2(0.6, 0.6)
	particles.color = color
	particles.position = pos
	effect_root.add_child(particles)
	particles.emitting = true
	particles.finished.connect(Callable(particles, "queue_free"))

func _start_screen_shake(strength: float, duration: float) -> void:
	shake_strength = strength
	shake_timer = duration

func _update_shake(delta: float) -> void:
	if camera == null:
		return
	if shake_timer > 0.0:
		shake_timer = max(0.0, shake_timer - delta)
		var decay: float = exp(-SCREEN_SHAKE_FALLOFF * delta)
		shake_strength *= decay
		var offset: Vector2 = Vector2(rng.randf_range(-1.0, 1.0), rng.randf_range(-SCREEN_SHAKE_VERTICAL_LIMIT, SCREEN_SHAKE_VERTICAL_LIMIT)) * shake_strength
		camera.offset = offset
	else:
		camera.offset = camera.offset.lerp(Vector2.ZERO, 0.2)

func _trigger_time_freeze(duration: float, time_scale: float) -> void:
	Engine.time_scale = clamp(time_scale, 0.05, 1.0)
	if freeze_timer != null and freeze_timer.is_inside_tree():
		freeze_timer.queue_free()
	freeze_timer = get_tree().create_timer(duration, false, false, true)
	freeze_timer.timeout.connect(Callable(self, "_on_freeze_timeout"))

func _on_freeze_timeout() -> void:
	Engine.time_scale = 1.0
	freeze_timer = null

func _play_sound(sound_name: String) -> void:
	if not audio_players.has(sound_name):
		return
	var player_node: AudioStreamPlayer = audio_players[sound_name]
	if player_node.stream is AudioStreamGenerator:
		if not player_node.playing:
			player_node.play()
		var playback: AudioStreamGeneratorPlayback = player_node.get_stream_playback()
		if playback != null:
			var base_frequency: float = float(audio_profiles.get(sound_name, 220.0))
			var frequency: float = base_frequency + rng.randi_range(-40, 40)
			_generate_beep(playback, frequency, 0.1)
	else:
		player_node.play()

func _setup_audio() -> void:
	var names: Array[String] = ["stomp", "damage", "gem", "block", "shot"]
	for sound_name in names:
		var stream_player: AudioStreamPlayer = AudioStreamPlayer.new()
		var generator: AudioStreamGenerator = AudioStreamGenerator.new()
		generator.mix_rate = 44100
		generator.buffer_length = 0.2
		stream_player.stream = generator
		stream_player.bus = "Master"
		add_child(stream_player)
		audio_players[sound_name] = stream_player
	for sound_name in audio_players.keys():
		var audio: AudioStreamPlayer = audio_players[sound_name]
		audio.play()
		audio.stop()

func _setup_background_music() -> void:
	var music_path: String = "res://assets/audio/GAME_SOUND.mp3"
	if not ResourceLoader.exists(music_path):
		push_warning("Background music file not found at: " + music_path)
		return

	var music_stream: AudioStream = ResourceLoader.load(music_path) as AudioStream
	if music_stream == null:
		push_error("Failed to load background music from: " + music_path)
		return

	background_music_player = AudioStreamPlayer.new()
	background_music_player.name = "BackgroundMusic"
	background_music_player.stream = music_stream
	background_music_player.bus = "Master"
	background_music_player.volume_db = -12.0
	background_music_player.autoplay = false

	if music_stream is AudioStreamMP3:
		music_stream.loop = true

	add_child(background_music_player)
	background_music_player.play()

func _generate_beep(playback: AudioStreamGeneratorPlayback, frequency: float, length: float) -> void:
	var mix_rate: float = 44100.0
	var increment: float = 1.0 / mix_rate
	var samples: int = int(mix_rate * length)
	if playback.get_frames_available() < samples:
		return
	var phase: float = 0.0
	var phase_inc: float = TAU * frequency * increment
	for _i in range(samples):
		var value: float = sin(phase) * 0.3
		playback.push_frame(Vector2(value, value))
		phase += phase_inc

func _on_shop_triggered(info: Dictionary) -> void:
	if shop_open or player == null:
		return
	shop_open = true
	run_active = false
	player.set_physics_process(false)
	player.velocity = Vector2.ZERO
	var options: Array[Dictionary] = _generate_shop_options()
	if shop_ui != null:
		shop_ui.open(options)
	emit_signal("shop_requested", info)
	if hud != null:
		hud.show_message("Shop erreicht!", 1.2)

func _generate_shop_options() -> Array[Dictionary]:
	var pool: Array[Dictionary] = [
		{"id": "ammo_up", "name": "Magazin Upgrade (+2 Ammo)", "cost": 12},
		{"id": "damage_up", "name": "Staerkere Gunboots (+1 Schaden)", "cost": 18},
		{"id": "heal", "name": "Erste Hilfe (+1 Gesundheit)", "cost": 10},
		{"id": "combo_boost", "name": "Combo Verstaerker (+0.1 Multiplier)", "cost": 14}
	]
	pool.shuffle()
	return pool.slice(0, 3)

func _on_shop_option_selected(option: Dictionary) -> void:
	var cost: int = option.get("cost", 0)
	if gems < cost:
		if hud != null:
			hud.show_message("Nicht genug Gems!", 1.0)
		return
	gems -= cost
	run_score += cost * 2
	if hud != null:
		hud.update_score(run_score, gems)
	var upgrade_id: String = option.get("id", "")
	_apply_upgrade(upgrade_id)
	if shop_ui != null:
		shop_ui.close()

func _apply_upgrade(upgrade_id: String) -> void:
	match upgrade_id:
		"ammo_up":
			if player != null:
				player.enhance_ammo(2)
		"damage_up":
			upgrade_state["gun_damage"] = int(upgrade_state.get("gun_damage", 1)) + 1
			if player != null:
				player.gun_damage = int(upgrade_state["gun_damage"])
		"heal":
			if player != null:
				player.heal(1)
		"combo_boost":
			upgrade_state["combo_bonus"] = float(upgrade_state.get("combo_bonus", 0.0)) + 0.1
	apply_upgrade_feedback(upgrade_id)
	upgrade_state["purchased"].append(upgrade_id)

func apply_upgrade_feedback(upgrade_id: String) -> void:
	var messages: Dictionary = {
		"ammo_up": "Max Ammo erhoeht!",
		"damage_up": "Gunboots staerker!",
		"heal": "Gesundheit +1",
		"combo_boost": "Combo waechst schneller!"
	}
	if hud != null and messages.has(upgrade_id):
		hud.show_message(messages[upgrade_id], 1.0)
	combo_multiplier = _combo_multiplier_value(current_combo)
	if hud != null and player != null:
		hud.update_combo(current_combo, combo_multiplier)
		hud.update_health(player.health)
		hud.update_ammo(player.ammo, player.max_ammo)

func _on_shop_closed() -> void:
	shop_open = false
	run_active = true
	if player != null:
		player.set_physics_process(true)

func _check_palette_unlocks() -> void:
	for palette_name in palette_unlocks.keys():
		if unlocked_palettes.has(palette_name):
			continue
		var requirement: int = palette_unlocks[palette_name]
		if max_combo >= requirement:
			unlocked_palettes.append(palette_name)
			emit_signal("meta_event", {"type": "unlock_palette", "palette": palette_name})
