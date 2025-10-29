extends StaticBody2D
class_name DestructibleBlock

signal broken(block: DestructibleBlock, position: Vector2)

var health := 2
var reward := {"gems": 5}
var block_color := Color(0.35, 0.32, 0.4)

var sprite: Sprite2D
var collision_shape: CollisionShape2D

func _ready() -> void:
	collision_layer = GameConstants.LAYER_WORLD
	collision_mask = GameConstants.LAYER_PLAYER | GameConstants.LAYER_PLAYER_SHOT
	_build_visuals()

func configure(size: Vector2, hit_points: int, reward_data: Dictionary) -> void:
	health = max(1, hit_points)
	reward = reward_data
	block_color = reward_data.get("color", block_color)
	if sprite:
		var texture := GameConstants.make_rect_texture(block_color, Vector2i(int(size.x), int(size.y)))
		sprite.texture = texture
		sprite.centered = false
		sprite.offset = Vector2(0, -size.y)
	if collision_shape:
		var shape := RectangleShape2D.new()
		shape.size = size
		collision_shape.shape = shape
		collision_shape.position = Vector2(size.x * 0.5, -size.y * 0.5)

func _build_visuals() -> void:
	sprite = Sprite2D.new()
	sprite.texture = GameConstants.make_rect_texture(block_color, Vector2i(32, 20))
	sprite.centered = false
	sprite.offset = Vector2(0, -20)
	add_child(sprite)

	collision_shape = CollisionShape2D.new()
	var shape := RectangleShape2D.new()
	shape.size = Vector2(32, 20)
	collision_shape.shape = shape
	collision_shape.position = Vector2(16, -10)
	add_child(collision_shape)

func hit_by_bullet(_bullet: Node, damage: int) -> void:
	apply_damage(damage)

func apply_damage(damage: int) -> void:
	health -= max(1, damage)
	var tween := create_tween()
	tween.tween_property(sprite, "modulate", Color(0.9, 0.7, 0.9), 0.05)
	tween.tween_property(sprite, "modulate", Color(1, 1, 1), 0.1)
	if health <= 0:
		emit_signal("broken", self, global_position)
		queue_free()
