extends Node
class_name GameManager

const SAVE_PATH := "user://progress.save"

var world_scene: Script = preload("res://scripts/game_world.gd")
var world: GameWorld = null
var meta_progress_data: Dictionary = {
	"high_score": 0,
	"max_combo": 0,
	"unlocked_palettes": ["DEFAULT"]
}

func _ready() -> void:
	_load_progress()
	_start_run()

func _start_run() -> void:
	if world:
		world.queue_free()
	world = world_scene.new()
	add_child(world)
	world.request_new_run.connect(_on_request_new_run)
	world.run_finished.connect(_on_run_finished)
	world.shop_requested.connect(_on_shop_requested)
	world.meta_event.connect(_on_meta_event)
	world.load_meta_progress(meta_progress_data)

func _on_request_new_run() -> void:
	_start_run()

func _on_run_finished(result: Dictionary) -> void:
	meta_progress_data["high_score"] = max(meta_progress_data.get("high_score", 0), result.get("score", 0))
	meta_progress_data["max_combo"] = max(meta_progress_data.get("max_combo", 0), result.get("max_combo", 0))
	_save_progress()

func _on_shop_requested(_shop_state: Dictionary) -> void:
	# Shops are handled entirely inside the world. This hook exists for future meta-features.
	pass

func _on_meta_event(event: Dictionary) -> void:
	if event.get("type", "") == "unlock_palette":
		var palette_name: String = event.get("palette", "")
		if palette_name != "" and palette_name not in meta_progress_data.get("unlocked_palettes", []):
			meta_progress_data["unlocked_palettes"].append(palette_name)
			_save_progress()

func _load_progress() -> void:
	if not FileAccess.file_exists(SAVE_PATH):
		return
	var file: FileAccess = FileAccess.open(SAVE_PATH, FileAccess.READ)
	if file != null:
		var data: Variant = file.get_var()
		if typeof(data) == TYPE_DICTIONARY:
			meta_progress_data = data

func _save_progress() -> void:
	var file: FileAccess = FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if file != null:
		file.store_var(meta_progress_data)
