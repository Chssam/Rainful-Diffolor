use bevy::{
	asset::{io::AssetSourceId, AssetPath},
	prelude::*,
	render::{
		settings::{RenderCreation, WgpuFeatures, WgpuSettings},
		RenderPlugin,
	},
	window::{WindowCreated, WindowResolution},
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
		app.insert_resource(Msaa::Off)
			.insert_resource(WinitSettings {
				focused_mode: UpdateMode::Continuous,
				unfocused_mode: UpdateMode::Continuous,
			})
			.add_plugins(
				DefaultPlugins
					.set(WindowPlugin {
						primary_window: Some(Window {
							title: APP_NAME.to_owned(),
							resolution: WindowResolution::new(900.0, 700.0),
							resize_constraints: WindowResizeConstraints {
								min_width: 500.0,
								min_height: 400.0,
								..default()
							},
							transparent: true,
							..default()
						}),
						..default()
					})
					.set(RenderPlugin {
						render_creation: RenderCreation::Automatic(WgpuSettings {
							features: WgpuFeatures::POLYGON_MODE_LINE,
							..default()
						}),
						..default()
					})
					.build(),
			)
			.add_systems(Startup, int_sources)
			.add_systems(Last, setup_icon);
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
		let img_icon = asset_img.get(icon_handler.id()).unwrap();

		let (i_rgba, i_width, i_height) =
			(img_icon.data.clone(), img_icon.width(), img_icon.height());
		let icon = Icon::from_rgba(i_rgba, i_width, i_height).unwrap();

		windowed.set_window_icon(Some(icon));
	});
}

#[derive(Resource, Deref)]
struct IconHandler(Handle<Image>);

fn int_sources(world: &mut World) {
	let path_icon = Path::new("Icon").join("Marks.png").embed();
	let asset_server = world.resource::<AssetServer>();
	let icon_handle = asset_server.load(path_icon);

	world.insert_resource(IconHandler(icon_handle));
}

pub trait ToEmbedPath {
	fn embed(&self) -> String;
}

impl ToEmbedPath for PathBuf {
	fn embed(&self) -> String {
		let source = AssetSourceId::from("embedded");
		AssetPath::from_path(self)
			.with_source(source.clone())
			.to_string()
	}
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
	dirs::document_dir().map(|doc_dir| doc_dir.join(APP_NAME))
}
