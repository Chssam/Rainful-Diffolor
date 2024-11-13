pub mod client;
pub mod server;
pub mod shared;

use bevy::prelude::*;
use client::AppClientPlugin;
use server::AppServerPlugin;
use shared::AppSharedPlugin;

pub struct ApplicationPlugin;
impl Plugin for ApplicationPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((AppClientPlugin, AppServerPlugin, AppSharedPlugin));
	}
}
