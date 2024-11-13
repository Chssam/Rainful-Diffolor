#[cfg(target_vendor = "pc")]
mod pc_only;
use bevy::prelude::*;

pub(super) struct PlatformSpecificPlugin;
impl Plugin for PlatformSpecificPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			#[cfg(target_vendor = "pc")]
			pc_only::GlobalUsagePlugin,
		));
	}
}
