class_name GameConstants

const LAYER_WORLD := 1
const LAYER_PLAYER := 2
const LAYER_ENEMY := 4
const LAYER_PLAYER_SHOT := 8
const LAYER_ENEMY_SHOT := 16
const LAYER_PICKUP := 32
const LAYER_SHOP := 64
const LAYER_SENSOR := 128
const LAYER_HAZARD := 256
const LAYER_TERRAIN := 512

static func make_rect_texture(color: Color, size: Vector2i) -> Texture2D:
	var image := Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	image.fill(color)
	return ImageTexture.create_from_image(image)

static func make_outline_texture(size: Vector2i, fill: Color, outline: Color, thickness: int = 2) -> Texture2D:
	var image := Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
	image.fill(fill)
	for x in size.x:
		for y in size.y:
			if x < thickness or y < thickness or x >= size.x - thickness or y >= size.y - thickness:
				image.set_pixel(x, y, outline)
	return ImageTexture.create_from_image(image)

static func move_toward(current: float, target: float, step: float) -> float:
	if current < target:
		return min(target, current + step)
	return max(target, current - step)

static func make_circle_texture(color: Color, radius: int) -> Texture2D:
	var size := radius * 2
	var image := Image.create(size, size, false, Image.FORMAT_RGBA8)
	for x in size:
		for y in size:
			var dx := x - radius + 0.5
			var dy := y - radius + 0.5
			if dx * dx + dy * dy <= radius * radius:
				image.set_pixel(x, y, color)
			else:
				image.set_pixel(x, y, Color(0, 0, 0, 0))
	return ImageTexture.create_from_image(image)
