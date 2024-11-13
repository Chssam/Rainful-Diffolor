use bevy::prelude::*;
use editor::ClientEditorPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use platform_specific::PlatformSpecificPlugin;

use super::shared::proto::MainChannel;
use crate::apps::shared::prelude::*;

mod connections;
mod control_room;
mod editor;
// mod experimental;
mod performing;
mod platform_specific;
mod sources;
mod world_view;

use connections::*;
use control_room::MegaEditorPlugin;
use performing::*;
use sources::*;

pub(super) struct AppClientPlugin;
impl Plugin for AppClientPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<EditorTools>()
			.register_type::<VisualGrid>()
			.add_plugins((
				PlatformSpecificPlugin,
				MegaEditorPlugin,
				build_client_plugin(),
				ClientEditorPlugin,
			))
			.add_systems(
				Update,
				(receive_message, fetch_connect_token).run_if(in_state(RdioClientState::Online)),
			)
			.add_systems(OnEnter(RdioClientState::Online), activate_connect_token)
			.add_systems(OnExit(RdioClientState::Offline), disconnect_from_server);
	}
}

fn receive_message(mut events: EventReader<MessageEvent<MessageCtx>>, mut cmd: Commands) {
	events.read().for_each(|event| {
		cmd.trigger(DisplayMsgEvent(event.message().0.clone()));
	});
}
