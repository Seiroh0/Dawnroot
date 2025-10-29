extends Area2D
class_name Gem

signal collected(data: Dictionary, position: Vector2)

var value := 5
var heal_bonus := 0

var sprite: Sprite2D
var collision_shape: CollisionShape2D
var float_phase := 0.0
var base_position_y := 0.0
var base_color := Color(0.6, 1, 0.85)

func _ready() -> void:
	collision_layer = GameConstants.LAYER_PICKUP
	collision_mask = GameConstants.LAYER_PLAYER | GameConstants.LAYER_PLAYER_SHOT
	monitoring = true
	monitorable = true
	body_entered.connect(_on_body_entered)
	area_entered.connect(_on_area_entered)
	base_position_y = position.y
	_build_visuals()

func configure(data: Dictionary) -> void:
	value = data.get("value", value)
	heal_bonus = data.get("health", heal_bonus)
	base_color = data.get("color", base_color)
	if sprite:
		sprite.texture = GameConstants.make_rect_texture(base_color, Vector2i(10, 10))

func _build_visuals() -> void:
	sprite = Sprite2D.new()
	sprite.name = "Sprite"
	sprite.texture = GameConstants.make_rect_texture(base_color, Vector2i(10, 10))
	sprite.centered = true
	add_child(sprite)

	collision_shape = CollisionShape2D.new()
	var shape := CircleShape2D.new()
	shape.radius = 6
	collision_shape.shape = shape
	add_child(collision_shape)

func _physics_process(delta: float) -> void:
	float_phase += delta * 3.0
	position.y = base_position_y + sin(float_phase) * 0.6

func _on_body_entered(body: Node) -> void:
	_process_collect(body)

func _on_area_entered(area: Node) -> void:
	if area is GunbootBullet:
		collect_from_bullet()

func _process_collect(body: Node) -> void:
	if not body or not body.has_method("on_pickup"):
		return
	var payload := {
		"gems": value,
		"health": heal_bonus
	}
	body.on_pickup(payload)
	emit_signal("collected", payload, global_position)
	queue_free()

func collect_from_bullet() -> void:
	emit_signal("collected", {
		"gems": value,
		"health": heal_bonus
	}, global_position)
	queue_free()
