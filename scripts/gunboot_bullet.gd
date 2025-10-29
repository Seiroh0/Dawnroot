extends Area2D
class_name GunbootBullet

signal impact(target: Node, position: Vector2)

const SPEED := 820.0
const LIFE_TIME := 0.8

var direction: Vector2 = Vector2.DOWN
var damage: int = 1
var lifetime: float = LIFE_TIME

var sprite: Sprite2D = null
var collision_shape: CollisionShape2D = null

func _ready() -> void:
	collision_layer = GameConstants.LAYER_PLAYER_SHOT
	collision_mask = GameConstants.LAYER_ENEMY | GameConstants.LAYER_WORLD | GameConstants.LAYER_PICKUP | GameConstants.LAYER_SENSOR
	monitoring = true
	monitorable = true
	body_entered.connect(Callable(self, "_on_body_entered"))
	area_entered.connect(Callable(self, "_on_area_entered"))
	_build_visuals()

func setup(dir: Vector2, power: int) -> void:
	direction = dir.normalized()
	damage = max(1, power)

func _physics_process(delta: float) -> void:
	position += direction * SPEED * delta
	lifetime -= delta
	if lifetime <= 0.0:
		queue_free()

func _build_visuals() -> void:
	sprite = Sprite2D.new()
	sprite.texture = GameConstants.make_rect_texture(Color(1, 0.85, 0.2), Vector2i(6, 12))
	sprite.centered = true
	sprite.modulate = Color(1, 0.9, 0.6, 0.95)
	add_child(sprite)

	collision_shape = CollisionShape2D.new()
	var rect_shape: RectangleShape2D = RectangleShape2D.new()
	rect_shape.size = Vector2(6, 12)
	collision_shape.shape = rect_shape
	add_child(collision_shape)

	var glow: Sprite2D = Sprite2D.new()
	glow.texture = GameConstants.make_rect_texture(Color(1, 1, 1, 0.2), Vector2i(10, 16))
	glow.centered = true
	add_child(glow)

func _on_body_entered(body: Node) -> void:
	_process_hit(body)

func _on_area_entered(area: Node) -> void:
	_process_hit(area)

func _process_hit(target: Node) -> void:
	if target == null:
		return
	if target.has_method("hit_by_bullet"):
		target.call_deferred("hit_by_bullet", self, damage)
	elif target.has_method("apply_damage"):
		target.call_deferred("apply_damage", damage)
	elif target.has_method("collect_from_bullet"):
		target.collect_from_bullet()
	emit_signal("impact", target, global_position)
	queue_free()
