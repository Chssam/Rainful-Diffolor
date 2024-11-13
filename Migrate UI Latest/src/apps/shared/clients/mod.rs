pub mod components;
pub mod events;
pub use components::*;
pub use events::*;

use bevy::prelude::*;
use lightyear::prelude::client::Predicted;

use super::SharingName;

pub(super) struct ClientWorldPlugin;
impl Plugin for ClientWorldPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<VerifyAction>()
			.add_event::<DisplayMsgEvent>()
			.observe(new_client);
	}
}

fn new_client(
	trigger: Trigger<OnAdd, UserId>,
	mut cmd: Commands,
	query_user: Query<&SharingName, Without<Predicted>>,
) {
	let ent_user = trigger.entity();
	let Ok(user_name) = query_user.get(ent_user) else {
		return;
	};
	let join_name = format!("[{}] entered world.", user_name.0);
	cmd.trigger(DisplayMsgEvent(join_name));
	cmd.entity(ent_user).insert(LocalUserBundle::default());
}
