pub mod image_edition;
mod processing;
pub mod proto;
pub mod pub_edition;
// pub mod rdio;
pub mod clients;
pub mod editor;
pub mod helperful_tool;
pub mod path_edition;
pub mod text_edition;
pub mod wolrd_view;

pub mod prelude {
	pub use super::image_edition::*;
	pub use super::proto::*;
	pub use super::pub_edition::*;
	// pub use super::rdio::*;
	pub use super::clients::*;
	pub use super::editor::lib::*;
	pub use super::path_edition::*;
	pub use super::text_edition::*;
}

use bevy::prelude::*;
use editor::*;
use lightyear::prelude::*;
use prelude::*;
use processing::*;
use proto::ProtocolPlugin;
use std::time::Duration;
use wolrd_view::LocalViewPlugin;

pub(super) struct AppSharedPlugin;
impl Plugin for AppSharedPlugin {
	fn build(&self, app: &mut App) {
		// app.configure_sets(Update, ProcessObject::Process.after(ProcessObject::Input));

		app.add_plugins((
			EditorPlugin,
			ProtocolPlugin,
			LocalViewPlugin,
			PathPlugin,
			ImageProcessPlugin,
			TextWorldPlugin,
			PublicPlugin,
			ClientWorldPlugin,
		));
	}
}

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub fn shared_config() -> SharedConfig {
	SharedConfig {
		tick: TickConfig {
			tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
		},
		mode: Mode::HostServer,
		..default()
	}
}

// #[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
// pub enum DeviceStatus {
// 	#[default]
// 	Offline,
// 	// HostAndDesign,
// 	ConnectServer,
// 	HostServer,
// 	HostAndView,
// }

// #[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// #[source(DeviceStatus = DeviceStatus::Offline || DeviceStatus = DeviceStatus::ConnectServer)]
// enum IsPaused {
// 	#[default]
// 	Running,
// 	Paused,
// }
