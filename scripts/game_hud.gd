extends CanvasLayer
class_name GameHud

const MAX_HEARTS := 4
const MAX_AMMO := 8

var heart_container: HBoxContainer = null
var ammo_container: HBoxContainer = null
var gem_icon: TextureRect = null
var gem_label: Label = null
var score_label: Label = null
var combo_label: Label = null
var message_label: Label = null

var heart_nodes: Array[TextureRect] = []
var ammo_nodes: Array[Control] = []
var combo_tween: Tween = null
var message_tween: Tween = null
var ammo_capacity: int = MAX_AMMO
var label_sizes: Dictionary = {}

var palette: Dictionary = {
	"bg": Color(0.05, 0.05, 0.08, 0.8),
	"ui": Color(1, 1, 1),
	"accent": Color(0.9, 0.15, 0.2),
	"muted": Color(0.2, 0.2, 0.25)
}

func set_palette(colors: Dictionary) -> void:
	for key in colors.keys():
		palette[key] = colors[key]
	_reapply_palette()

func _ready() -> void:
	layer = 30
	_build_ui()

func _build_ui() -> void:
	var root: Control = Control.new()
	root.name = "HUDRoot"
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(root)

	var top_left: MarginContainer = MarginContainer.new()
	top_left.set_anchors_preset(Control.PRESET_TOP_LEFT)
	top_left.offset_right = 160
	top_left.offset_bottom = 120
	top_left.add_theme_constant_override("margin_left", 8)
	top_left.add_theme_constant_override("margin_top", 8)
	root.add_child(top_left)

	var tl_box: VBoxContainer = VBoxContainer.new()
	tl_box.alignment = BoxContainer.ALIGNMENT_BEGIN
	tl_box.add_theme_constant_override("separation", 4)
	top_left.add_child(tl_box)

	heart_container = HBoxContainer.new()
	heart_container.add_theme_constant_override("separation", 4)
	tl_box.add_child(heart_container)
	_build_hearts()

	ammo_container = HBoxContainer.new()
	ammo_container.add_theme_constant_override("separation", 2)
	tl_box.add_child(ammo_container)
	_build_ammo()

	var top_right: MarginContainer = MarginContainer.new()
	top_right.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	top_right.offset_left = -160
	top_right.offset_bottom = 64
	top_right.add_theme_constant_override("margin_right", 8)
	top_right.add_theme_constant_override("margin_top", 8)
	root.add_child(top_right)

	var tr_box: HBoxContainer = HBoxContainer.new()
	tr_box.alignment = BoxContainer.ALIGNMENT_END
	tr_box.add_theme_constant_override("separation", 6)
	top_right.add_child(tr_box)

	gem_icon = TextureRect.new()
	gem_icon.texture = _make_rect_texture(palette.get("accent", Color(0.9, 0.15, 0.2)), Vector2i(14, 14))
	gem_icon.stretch_mode = TextureRect.STRETCH_KEEP
	gem_icon.custom_minimum_size = Vector2(18, 18)
	tr_box.add_child(gem_icon)

	gem_label = _make_label("000", 16)
	tr_box.add_child(gem_label)

	var top_center: MarginContainer = MarginContainer.new()
	top_center.set_anchors_preset(Control.PRESET_TOP_WIDE)
	top_center.offset_top = 6
	top_center.offset_bottom = 34
	root.add_child(top_center)

	score_label = _make_label("000000", 18)
	score_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	top_center.add_child(score_label)

	combo_label = _make_label("", 48)
	combo_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	combo_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	combo_label.modulate = Color(1, 1, 1, 0)
	combo_label.set_anchors_preset(Control.PRESET_CENTER)
	combo_label.offset_left = -80
	combo_label.offset_right = 80
	combo_label.offset_top = -24
	combo_label.offset_bottom = 24
	root.add_child(combo_label)

	message_label = _make_label("", 18)
	message_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	message_label.modulate = Color(1, 1, 1, 0)
	message_label.set_anchors_preset(Control.PRESET_TOP_WIDE)
	message_label.offset_top = 44
	message_label.offset_bottom = 72
	root.add_child(message_label)

func _build_hearts() -> void:
	_clear_container(heart_container)
	heart_nodes.clear()
	for i in range(MAX_HEARTS):
		var heart_rect: TextureRect = TextureRect.new()
		heart_rect.texture = _make_heart_texture(i < MAX_HEARTS)
		heart_rect.custom_minimum_size = Vector2(18, 16)
		heart_rect.stretch_mode = TextureRect.STRETCH_KEEP
		heart_container.add_child(heart_rect)
		heart_nodes.append(heart_rect)

func _build_ammo() -> void:
	_clear_container(ammo_container)
	ammo_nodes.clear()
	for i in range(ammo_capacity):
		var slot: ColorRect = ColorRect.new()
		slot.color = palette.get("muted", Color(0.2, 0.2, 0.25))
		slot.custom_minimum_size = Vector2(10, 6)
		slot.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		ammo_container.add_child(slot)
		ammo_nodes.append(slot)

func update_health(value: int) -> void:
	var hearts: int = clamp(value, 0, MAX_HEARTS)
	for i in range(heart_nodes.size()):
		var heart_rect: TextureRect = heart_nodes[i]
		if heart_rect != null:
			heart_rect.texture = _make_heart_texture(i < hearts)

func update_ammo(current: int, maximum: int) -> void:
	if maximum != ammo_capacity:
		ammo_capacity = clamp(maximum, 1, 16)
		_build_ammo()
	var filled: int = clamp(current, 0, ammo_capacity)
	for i in range(ammo_nodes.size()):
		var ammo_slot: Control = ammo_nodes[i]
		if ammo_slot != null:
			ammo_slot.color = palette.get("accent", Color(0.9, 0.15, 0.2)) if i < filled else palette.get("muted", Color(0.2, 0.2, 0.25))
	if current < maximum and maximum > 0:
		_flash_node(ammo_container)

func update_score(score: int, gems: int) -> void:
	score_label.text = "%06d" % score
	gem_label.text = "%03d" % gems

func update_depth(_value: float) -> void:
	pass

func update_combo(combo: int, _multiplier: float) -> void:
	if combo <= 0:
		_hide_combo()
		return
	combo_label.text = "%d" % combo
	combo_label.modulate = Color(1, 1, 1, 1)
	if combo_tween:
		combo_tween.kill()
	combo_tween = create_tween()
	combo_tween.tween_property(combo_label, "scale", Vector2.ONE * 1.4, 0.1).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	combo_tween.tween_property(combo_label, "scale", Vector2.ONE, 0.15).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)

func _hide_combo() -> void:
	if combo_tween:
		combo_tween.kill()
	combo_label.text = ""
	combo_label.scale = Vector2.ONE
	combo_label.modulate = Color(1, 1, 1, 0)

func show_message(text: String, duration: float = 1.5) -> void:
	message_label.text = text
	if message_tween:
		message_tween.kill()
	message_tween = create_tween()
	message_tween.tween_property(message_label, "modulate", Color(1, 1, 1, 1), 0.2)
	message_tween.tween_interval(duration)
	message_tween.tween_property(message_label, "modulate", Color(1, 1, 1, 0), 0.3)

func _make_label(text: String, size: int) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.label_settings = _label_settings(size)
	label.custom_minimum_size = Vector2(120, size + 6)
	label_sizes[label] = size
	return label

func _label_settings(size: int) -> LabelSettings:
	var settings: LabelSettings = LabelSettings.new()
	settings.font_size = size
	settings.font_color = palette.get("ui", Color(1, 1, 1))
	settings.outline_size = 1
	settings.outline_color = Color(0, 0, 0, 0.85)
	return settings

func _make_rect_texture(color: Color, size: Vector2i) -> Texture2D:
	var image: Image = Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	image.fill(color)
	return ImageTexture.create_from_image(image)

func _make_heart_texture(filled: bool) -> Texture2D:
	var size: Vector2i = Vector2i(16, 14)
	var image: Image = Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	var color: Color = palette.get("accent", Color(0.9, 0.15, 0.2)) if filled else palette.get("muted", Color(0.2, 0.2, 0.25))
	for x in range(size.x):
		for y in range(size.y):
			var dx: float = abs(x - size.x / 2.0)
			var _dy: float = float(y)
			if (y < 4 and dx < size.x / 2.5) or ((y >= 4) and (dx + y * 0.6) < size.x / 2.4):
				image.set_pixel(x, y, color)
			else:
				image.set_pixel(x, y, Color(0, 0, 0, 0))
	return ImageTexture.create_from_image(image)

func _flash_node(node: CanvasItem) -> void:
	if node == null:
		return
	var tween: Tween = node.create_tween()
	tween.tween_property(node, "modulate", Color(1, 1, 1, 0.2), 0.05)
	tween.tween_property(node, "modulate", Color(1, 1, 1, 1), 0.1)

func _clear_container(container: Container) -> void:
	for child in container.get_children():
		child.queue_free()

func _reapply_palette() -> void:
	if gem_icon:
		gem_icon.texture = _make_rect_texture(palette.get("accent", Color(0.9, 0.15, 0.2)), Vector2i(14, 14))
	_build_hearts()
	_build_ammo()
	for label in label_sizes.keys():
		var size: int = int(label_sizes[label])
		label.label_settings = _label_settings(size)








