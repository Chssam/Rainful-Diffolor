mod lib;

use std::time::{SystemTime, UNIX_EPOCH};

use self::lib::*;
use crate::apps::client::*;
use rainful_diffolor::{source_to_docs, APP_NAME};

use bevy::{
	render::view::screenshot::ScreenshotManager,
	window::{PrimaryWindow, WindowMode},
};
use leafwing_input_manager::prelude::*;

pub(super) struct GlobalUsagePlugin;
impl Plugin for GlobalUsagePlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(InputManagerPlugin::<StaticBind>::default())
			.init_resource::<ActionState<StaticBind>>()
			.insert_resource(StaticBind::bind_default())
			.add_systems(Update, (screen_toggle, screenshot));
	}
}

fn screen_toggle(
	mut windows: Query<&mut Window, With<PrimaryWindow>>,
	kb_i: Res<ActionState<StaticBind>>,
) {
	if !kb_i.just_pressed(&StaticBind::ScreenMode) {
		return;
	}
	let mut window = windows.single_mut();
	window.mode = match window.mode {
		WindowMode::Windowed => WindowMode::BorderlessFullscreen,
		WindowMode::BorderlessFullscreen => WindowMode::Fullscreen,
		_ => WindowMode::Windowed,
	};
}

fn screenshot(
	mut screenshot_manager: ResMut<ScreenshotManager>,
	kb_i: Res<ActionState<StaticBind>>,
	main_window: Query<Entity, With<PrimaryWindow>>,
) {
	if !kb_i.just_pressed(&StaticBind::ScreenShot) {
		return;
	}
	let time_now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_nanos();
	let file_name = format!("{} {}.png", APP_NAME, time_now);
	let Some(doc_dir) = source_to_docs() else {
		return;
	};
	let path = doc_dir.join("ScreenShot").join(file_name);
	screenshot_manager
		.save_screenshot_to_disk(main_window.single(), path)
		.unwrap();
}
