use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use super::*;

pub fn instant_action<T: Actionlike + Copy>(
	target: Entity,
	action: T,
) -> OwnObserve<RunEffect, ()> {
	OwnObserve::new(
		move |_trigger: Trigger<RunEffect>, mut query_target: Query<&mut ActionState<T>>| {
			let Ok(mut action_state) = query_target.get_mut(target) else {
				error!("Targeting Non Exist Entity");
				return;
			};
			action_state.release(&action);
			action_state.press(&action);
		},
	)
}
