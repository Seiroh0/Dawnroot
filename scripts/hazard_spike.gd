extends Area2D
class_name HazardSpike

signal body_damaged(body: Node, hazard: Node)

var damage: int = 1
var palette: Dictionary = {
	"hazard": Color(0.8, 0.2, 0.25)
}

var sprite: Sprite2D = null

func _ready() -> void:
	collision_layer = GameConstants.LAYER_HAZARD
	collision_mask = GameConstants.LAYER_PLAYER
	monitoring = true
	monitorable = true
	body_entered.connect(Callable(self, "_on_body_entered"))
	_build_visuals()

func set_palette(palette_data: Dictionary) -> void:
	for key in palette_data.keys():
		palette[key] = palette_data[key]
	_update_visuals()

func _build_visuals() -> void:
	sprite = Sprite2D.new()
	sprite.name = "Sprite"
	sprite.centered = true
	add_child(sprite)
	_update_visuals()

	var glow: Sprite2D = Sprite2D.new()
	glow.texture = GameConstants.make_circle_texture(palette.get("hazard", Color(0.8, 0.2, 0.25)), 10)
	glow.centered = true
	glow.modulate = Color(1, 1, 1, 0.25)
	glow.position = Vector2(0, -12)
	add_child(glow)

	var shape: CollisionShape2D = CollisionShape2D.new()
	var triangle: ConvexPolygonShape2D = ConvexPolygonShape2D.new()
	triangle.points = PackedVector2Array([
		Vector2(-12, 8),
		Vector2(12, 8),
		Vector2(0, -12)
	])
	shape.shape = triangle
	add_child(shape)

func _update_visuals() -> void:
	if sprite == null:
		return
	sprite.texture = GameConstants.make_rect_texture(palette.get("hazard", Color(0.8, 0.2, 0.25)), Vector2i(24, 14))
	sprite.position = Vector2(0, -6)

func _on_body_entered(body: Node) -> void:
	if body == null:
		return
	body_damaged.emit(body, self)
