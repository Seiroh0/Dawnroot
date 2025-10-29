extends Node2D
## Main entry point scene for Gunboot Descent
## The actual game logic is handled by the GameRoot autoload singleton

func _ready() -> void:
	print("Main scene initialized")
	print("GameRoot autoload is handling game initialization")

	# The GameRoot autoload (game_root.gd) automatically creates and manages
	# the GameWorld instance. This main scene just serves as the entry point.
	# All game logic happens in GameRoot -> GameWorld -> Player/Enemies/etc.
