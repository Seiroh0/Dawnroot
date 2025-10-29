extends CanvasLayer
class_name TutorialManager

signal tutorial_completed()
signal step_completed(step_id: String)

enum TutorialStep {
	NONE,
	MOVEMENT,
	JUMP,
	SHOOT,
	STOMP,
	COMPLETED
}

const PROMPT_FADE_DURATION := 0.3
const STEP_COMPLETION_DELAY := 1.2
const TEXT_APPEAR_DURATION := 0.4

var current_step: TutorialStep = TutorialStep.MOVEMENT
var player: Player = null
var tutorial_active: bool = false
var step_completed_flags: Dictionary = {}

# UI Elements
var prompt_container: Control = null
var prompt_label: Label = null
var prompt_background: ColorRect = null
var prompt_icon: Label = null

# Tutorial text content
var tutorial_prompts: Dictionary = {
	TutorialStep.MOVEMENT: {
		"text": "Use WASD or Arrow Keys to move left and right",
		"icon": "←→",
		"check": "_check_movement"
	},
	TutorialStep.JUMP: {
		"text": "Press SPACE while on the ground to jump",
		"icon": "↑",
		"check": "_check_jump"
	},
	TutorialStep.SHOOT: {
		"text": "Press SPACE in the air to shoot downward",
		"icon": "✦",
		"check": "_check_shoot"
	},
	TutorialStep.STOMP: {
		"text": "Land on enemies to stomp them and refill ammo",
		"icon": "⚡",
		"check": "_check_stomp"
	}
}

# Progress tracking
var movement_distance: float = 0.0
var last_player_x: float = 0.0
var has_jumped: bool = false
var has_shot: bool = false
var has_stomped: bool = false

const MOVEMENT_THRESHOLD := 120.0

func _ready() -> void:
	layer = 50
	_build_ui()

func setup(player_ref: Player) -> void:
	player = player_ref

	# Connect to player signals
	if player != null:
		if player.has_signal("bullet_fired"):
			player.bullet_fired.connect(_on_player_shot)
		if player.has_signal("stomp"):
			player.stomp.connect(_on_player_stomp)
		if player.has_signal("landed"):
			player.landed.connect(_on_player_landed)

		last_player_x = player.global_position.x

func start_tutorial() -> void:
	if tutorial_active:
		return

	tutorial_active = true
	current_step = TutorialStep.MOVEMENT
	_reset_progress()
	_show_current_step()

func stop_tutorial() -> void:
	tutorial_active = false
	_hide_prompt()

func _build_ui() -> void:
	# Create container for tutorial prompts
	prompt_container = Control.new()
	prompt_container.name = "TutorialPrompt"
	prompt_container.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	prompt_container.offset_top = -180
	prompt_container.offset_bottom = -120
	prompt_container.offset_left = -250
	prompt_container.offset_right = 250
	add_child(prompt_container)

	# Background panel
	prompt_background = ColorRect.new()
	prompt_background.color = Color(0.1, 0.1, 0.15, 0.92)
	prompt_background.set_anchors_preset(Control.PRESET_FULL_RECT)
	prompt_background.grow_horizontal = Control.GROW_DIRECTION_BOTH
	prompt_background.grow_vertical = Control.GROW_DIRECTION_BOTH
	prompt_container.add_child(prompt_background)

	# Icon label
	prompt_icon = Label.new()
	prompt_icon.name = "Icon"
	prompt_icon.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	prompt_icon.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	prompt_icon.add_theme_font_size_override("font_size", 36)
	prompt_icon.position = Vector2(10, 10)
	prompt_icon.size = Vector2(50, 50)
	prompt_container.add_child(prompt_icon)

	# Text label
	prompt_label = Label.new()
	prompt_label.name = "PromptText"
	prompt_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	prompt_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	prompt_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	prompt_label.add_theme_font_size_override("font_size", 18)
	prompt_label.position = Vector2(70, 10)
	prompt_label.size = Vector2(410, 50)
	prompt_container.add_child(prompt_label)

	# Initially hidden
	prompt_container.modulate = Color(1, 1, 1, 0)
	prompt_container.visible = false

func _physics_process(_delta: float) -> void:
	if not tutorial_active or player == null:
		return

	# Check current step completion
	match current_step:
		TutorialStep.MOVEMENT:
			_check_movement()
		TutorialStep.JUMP:
			_check_jump()
		TutorialStep.SHOOT:
			_check_shoot()
		TutorialStep.STOMP:
			_check_stomp()

func _show_current_step() -> void:
	if current_step == TutorialStep.COMPLETED or current_step == TutorialStep.NONE:
		_hide_prompt()
		return

	var step_data: Dictionary = tutorial_prompts.get(current_step, {})
	if step_data.is_empty():
		return

	prompt_label.text = step_data.get("text", "")
	prompt_icon.text = step_data.get("icon", "")

	# Show with fade-in animation
	prompt_container.visible = true
	var tween: Tween = create_tween()
	tween.tween_property(prompt_container, "modulate", Color(1, 1, 1, 1), TEXT_APPEAR_DURATION)

func _hide_prompt() -> void:
	if prompt_container == null:
		return

	var tween: Tween = create_tween()
	tween.tween_property(prompt_container, "modulate", Color(1, 1, 1, 0), PROMPT_FADE_DURATION)
	tween.tween_callback(func() -> void:
		prompt_container.visible = false
	)

func _advance_to_next_step() -> void:
	emit_signal("step_completed", _step_to_string(current_step))

	match current_step:
		TutorialStep.MOVEMENT:
			current_step = TutorialStep.JUMP
		TutorialStep.JUMP:
			current_step = TutorialStep.SHOOT
		TutorialStep.SHOOT:
			current_step = TutorialStep.STOMP
		TutorialStep.STOMP:
			current_step = TutorialStep.COMPLETED
			_complete_tutorial()
			return
		_:
			_complete_tutorial()
			return

	# Wait before showing next step
	var timer: SceneTreeTimer = get_tree().create_timer(STEP_COMPLETION_DELAY)
	timer.timeout.connect(_show_current_step)
	_hide_prompt()

func _complete_tutorial() -> void:
	tutorial_active = false
	_hide_prompt()
	emit_signal("tutorial_completed")

func _check_movement() -> void:
	if player == null:
		return

	var current_x: float = player.global_position.x
	movement_distance += abs(current_x - last_player_x)
	last_player_x = current_x

	if movement_distance >= MOVEMENT_THRESHOLD:
		_advance_to_next_step()

func _check_jump() -> void:
	if has_jumped:
		_advance_to_next_step()

func _check_shoot() -> void:
	if has_shot:
		_advance_to_next_step()

func _check_stomp() -> void:
	if has_stomped:
		_advance_to_next_step()

func _on_player_shot(_bullet: Node) -> void:
	has_shot = true

func _on_player_stomp(_enemy: Node) -> void:
	has_stomped = true

func _on_player_landed() -> void:
	if player == null:
		return

	# Check if player jumped (was airborne and landed)
	if player.airborne_time > 0.15:
		has_jumped = true

func _reset_progress() -> void:
	movement_distance = 0.0
	has_jumped = false
	has_shot = false
	has_stomped = false
	step_completed_flags.clear()

	if player != null:
		last_player_x = player.global_position.x

func _step_to_string(step: TutorialStep) -> String:
	match step:
		TutorialStep.MOVEMENT:
			return "movement"
		TutorialStep.JUMP:
			return "jump"
		TutorialStep.SHOOT:
			return "shoot"
		TutorialStep.STOMP:
			return "stomp"
		TutorialStep.COMPLETED:
			return "completed"
		_:
			return "none"

func skip_tutorial() -> void:
	if not tutorial_active:
		return

	current_step = TutorialStep.COMPLETED
	_complete_tutorial()

func is_tutorial_active() -> bool:
	return tutorial_active
