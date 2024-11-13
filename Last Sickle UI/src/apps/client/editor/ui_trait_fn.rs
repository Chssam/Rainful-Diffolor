use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn instant_action<T: Actionlike + Copy>(target: Entity, action: T) -> impl Bundle {
	(
		On::<Pointer<Click>>::run(move |mut query_target: Query<&mut ActionState<T>>| {
			let Ok(mut action_state) = query_target.get_mut(target) else {
				error!("Targeting Non Exist Entity");
				return;
			};
			action_state.reset(&action);
			action_state.press(&action);
		}),
		On::<Pointer<Out>>::run(move |mut query_target: Query<&mut ActionState<T>>| {
			let Ok(mut action_state) = query_target.get_mut(target) else {
				error!("Targeting Non Exist Entity");
				return;
			};
			action_state.release(&action);
		}),
	)
}

pub fn toggle_action<T: Actionlike + Copy>(target: Entity, action: T) -> On<Pointer<Click>> {
	On::<Pointer<Click>>::run(move |mut query_target: Query<&mut ActionState<T>>| {
		let Ok(mut action_state) = query_target.get_mut(target) else {
			error!("Targeting Non Exist Entity");
			return;
		};
		if action_state.pressed(&action) {
			action_state.release(&action);
		} else {
			action_state.press(&action);
		}
	})
}
