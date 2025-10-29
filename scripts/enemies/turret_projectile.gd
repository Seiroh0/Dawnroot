extends Area2D
class_name TurretProjectile

var speed := 420.0
var direction := Vector2.UP
var lifetime := 2.5

func _ready() -> void:
	collision_layer = GameConstants.LAYER_ENEMY_SHOT
	collision_mask = GameConstants.LAYER_PLAYER | GameConstants.LAYER_TERRAIN
	monitoring = true
	monitorable = true
	body_entered.connect(_on_body_entered)
	area_entered.connect(_on_area_entered)
	_build_visuals()

func setup(dir: Vector2, projectile_speed: float) -> void:
	direction = dir.normalized()
	speed = projectile_speed

func _physics_process(delta: float) -> void:
	position += direction * speed * delta
	lifetime -= delta
	if lifetime <= 0.0:
		queue_free()

func _on_body_entered(body: Node) -> void:
	if body and body.has_method("take_damage"):
		body.take_damage(1, self)
	queue_free()

func _on_area_entered(_area: Area2D) -> void:
	queue_free()

func _build_visuals() -> void:
	var sprite := Sprite2D.new()
	sprite.texture = GameConstants.make_rect_texture(Color(1, 0.5, 0.2), Vector2i(6, 16))
	sprite.centered = true
	add_child(sprite)

	var glow := Sprite2D.new()
	glow.texture = GameConstants.make_rect_texture(Color(1, 0.8, 0.3, 0.4), Vector2i(10, 20))
	glow.centered = true
	add_child(glow)

	var shape := CollisionShape2D.new()
	var rect := RectangleShape2D.new()
	rect.size = Vector2(6, 16)
	shape.shape = rect
	add_child(shape)
