extends CanvasLayer
class_name ShopUI

signal purchase_selected(option: Dictionary)
signal closed()

var option_buttons: Array[Button] = []
var options: Array[Dictionary] = []
var selected_index := 0
var is_open := false

func _ready() -> void:
	layer = 20
	visible = false
	_build_ui()

func _build_ui() -> void:
	var panel := Panel.new()
	panel.name = "Panel"
	panel.set_anchors_preset(Control.PRESET_CENTER)
	panel.offset_left = -220
	panel.offset_right = 220
	panel.offset_top = -160
	panel.offset_bottom = 160
	add_child(panel)

	var vbox := VBoxContainer.new()
	vbox.name = "Container"
	vbox.set_anchors_preset(Control.PRESET_CENTER)
	vbox.offset_left = -180
	vbox.offset_right = 180
	vbox.offset_top = -120
	vbox.offset_bottom = 120
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	vbox.add_theme_constant_override("separation", 16)
	panel.add_child(vbox)

	var title := Label.new()
	title.text = "Untergrund-Shop"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.label_settings = _label_settings()
	vbox.add_child(title)

	for i in range(3):
		var button := Button.new()
		button.text = "Option"
		button.visible = false
		button.focus_mode = Control.FOCUS_ALL
		button.toggle_mode = false
		button.pressed.connect(func() -> void:
			_select_by_button(button)
		)
		option_buttons.append(button)
		vbox.add_child(button)

	var close_button := Button.new()
	close_button.text = "Weiter"
	close_button.pressed.connect(_on_close_pressed)
	vbox.add_child(close_button)

func _label_settings() -> LabelSettings:
	var settings := LabelSettings.new()
	settings.font_size = 24
	settings.outline_size = 1
	settings.outline_color = Color(0, 0, 0, 0.8)
	return settings

func open(option_data: Array[Dictionary]) -> void:
	options = option_data
	selected_index = 0
	is_open = true
	visible = true
	_refresh_buttons()
	set_process_unhandled_input(true)
	_grab_focus()

func _refresh_buttons() -> void:
	for i in option_buttons.size():
		var button := option_buttons[i]
		if i < options.size():
			var data := options[i]
			button.text = "%s\nKosten: %d" % [data.get("name", "Upgrade"), data.get("cost", 0)]
			button.disabled = false
			button.visible = true
		else:
			button.visible = false
	if options.is_empty():
		return
	highlight_selected()

func highlight_selected() -> void:
	for i in option_buttons.size():
		var button := option_buttons[i]
		if not button.visible:
			continue
		button.modulate = Color(1, 1, 1) if i == selected_index else Color(0.8, 0.8, 0.8)

func _select_by_button(button: Button) -> void:
	var index := option_buttons.find(button)
	if index == -1:
		return
	selected_index = index
	highlight_selected()
	confirm_selection()

func confirm_selection() -> void:
	if selected_index < 0 or selected_index >= options.size():
		return
	var choice := options[selected_index]
	emit_signal("purchase_selected", choice)

func close() -> void:
	is_open = false
	visible = false
	set_process_unhandled_input(false)
	emit_signal("closed")

func _on_close_pressed() -> void:
	close()

func _unhandled_input(event: InputEvent) -> void:
	if not is_open:
		return
	if event.is_action_pressed("move_left"):
		selected_index = max(0, selected_index - 1)
		highlight_selected()
	elif event.is_action_pressed("move_right"):
		selected_index = min(options.size() - 1, selected_index + 1)
		highlight_selected()
	elif event.is_action_pressed("shoot"):
		confirm_selection()
	elif event.is_action_pressed("ui_cancel"):
		close()

func _grab_focus() -> void:
	if selected_index < option_buttons.size():
		option_buttons[selected_index].grab_focus()
