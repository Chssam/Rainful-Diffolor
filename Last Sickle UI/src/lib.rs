use bevy::{
	asset::{io::AssetSourceId, AssetPath},
	prelude::*,
	render::{
		settings::{RenderCreation, WgpuFeatures, WgpuSettings},
		RenderPlugin,
	},
	window::{PresentMode, WindowCreated, WindowMode, WindowResolution},
	winit::{UpdateMode, WinitSettings, WinitWindows},
};
use std::{
	fs,
	path::{Path, PathBuf},
};
use winit::window::Icon;

pub const APP_NAME: &str = "Rainful Diffolor";
pub const RDIO_IN_SAVE_DISK: &str = "Rdio";

pub struct StartupAppPlugin;
impl Plugin for StartupAppPlugin {
	fn build(&self, app: &mut App) {
		create_file_holder();
		let win_plugin = if cfg!(target_vendor = "pc") {
			WindowPlugin {
				primary_window: Some(Window {
					title: APP_NAME.to_owned(),
					resolution: WindowResolution::new(900.0, 700.0),
					resize_constraints: WindowResizeConstraints {
						min_width: 500.0,
						min_height: 400.0,
						..default()
					},
					transparent: true,
					present_mode: PresentMode::default(),
					..default()
				}),
				..default()
			}
		} else {
			WindowPlugin {
				primary_window: Some(Window {
					title: APP_NAME.to_owned(),
					mode: WindowMode::BorderlessFullscreen,
					..default()
				}),
				..default()
			}
		};

		app.insert_resource(Msaa::Off)
			.insert_resource(WinitSettings {
				focused_mode: UpdateMode::Continuous,
				unfocused_mode: UpdateMode::Continuous,
			})
			.add_plugins(
				DefaultPlugins
					.set(win_plugin)
					.set(RenderPlugin {
						render_creation: RenderCreation::Automatic(WgpuSettings {
							features: WgpuFeatures::POLYGON_MODE_LINE,
							..default()
						}),
						..default()
					})
					.build(),
			)
			.add_systems(Startup, int_sources);

		#[cfg(target_vendor = "pc")]
		app.add_systems(Last, setup_icon);
	}
}

fn setup_icon(
	mut created_window_event: EventReader<WindowCreated>,
	windows: NonSend<WinitWindows>,
	asset_img: Res<Assets<Image>>,
	icon_handler: Res<IconHandler>,
) {
	created_window_event.read().for_each(|ent_win| {
		let windowed = windows.get_window(ent_win.window).expect("Window Exist");
		let img_icon = asset_img
			.get(icon_handler.id())
			.expect("Included in Binary");

		let (i_rgba, i_width, i_height) =
			(img_icon.data.clone(), img_icon.width(), img_icon.height());
		let icon = Icon::from_rgba(i_rgba, i_width, i_height).unwrap();

		windowed.set_window_icon(Some(icon));
	});
}

#[derive(Resource, Deref)]
struct IconHandler(Handle<Image>);

fn int_sources(world: &mut World) {
	let asset_server = world.resource::<AssetServer>();
	let path_icon = Path::new("Icon").join("Marks.png");
	let icon_handle = asset_server.load(embed_path(&path_icon));

	world.insert_resource(IconHandler(icon_handle));
}

pub fn embed_path(path: &Path) -> String {
	let source = AssetSourceId::from("embedded");
	AssetPath::from_path(path)
		.with_source(source.clone())
		.to_string()
}

fn create_file_holder() {
	let Some(docs_dir) = source_to_docs() else {
		return;
	};
	let screen_file = docs_dir.join("ScreenShot");
	let temp = docs_dir.join("Temporary");
	let rdio_dir = docs_dir.join(RDIO_IN_SAVE_DISK);
	let rdio_img_dir = rdio_dir.join("Image");
	let _ = fs::create_dir_all(screen_file);
	let _ = fs::create_dir_all(rdio_dir);
	let _ = fs::create_dir_all(rdio_img_dir);
	let _ = fs::create_dir_all(temp);
}

pub fn source_to_docs() -> Option<PathBuf> {
	let Some(doc_dir) = dirs::document_dir() else {
		warn!("No Document file");
		return None;
	};
	let path = doc_dir.join(APP_NAME);
	Some(path)
}

// pub fn state_directory() -> PathBuf {
//     dirs::data_dir()
//         .map(|platform_data_dir| platform_data_dir.join(APP_NAME).join("state"))
//         .unwrap_or(Path::new("session").join("data").join("state"))
//         .join("setup")
// }
