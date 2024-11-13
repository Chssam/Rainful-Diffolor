// #![windows_subsystem = "windows"] // Uncomment when releasing
#![allow(clippy::type_complexity)]
#![deny(clippy::deprecated_semver)]
use bevy::prelude::*;
use rainful_diffolor::*;
mod apps;
mod camera_control;
mod tool_tip;
mod trait_bevy;
mod ui;
use bevy_cosmic_edit::*;
use bevy_mod_picking::prelude::*;

#[bevy_main]
fn main() -> AppExit {
	let font_config = CosmicFontConfig {
		fonts_dir_path: dirs::font_dir(),
		load_system_fonts: true,
		..default()
	};
	App::new()
		.add_plugins((
			bevy_embedded_assets::EmbeddedAssetPlugin::default(),
			StartupAppPlugin,
			DefaultPickingPlugins,
			apps::ApplicationPlugin,
			camera_control::CameraPlugin,
			tool_tip::ToolInfoPlugin,
			ui::EffectUIPlugin,
			vleue_kinetoscope::AnimatedImagePlugin,
			bevy_prototype_lyon::plugin::ShapePlugin,
			bevy_vector_shapes::Shape2dPlugin::default(),
			CosmicEditPlugin { font_config },
		))
		.insert_resource(DebugPickingMode::Noisy)
		.run()
}
