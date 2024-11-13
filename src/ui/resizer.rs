use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use bevy_mod_picking::prelude::*;

use super::UILayer;

pub(super) fn resize_handler(
	mut on_over: EventReader<Pointer<Over>>,
	mut on_out: EventReader<Pointer<Out>>,
	mut query_resizer: Query<&mut Style, With<ResizeHandler>>,
) {
	on_over.read().for_each(|pointed| {
		let Ok(mut style) = query_resizer.get_mut(pointed.target()) else {
			return;
		};
		if let Val::Px(v) = &mut style.border.left {
			*v = 2.;
		}
		if let Val::Px(v) = &mut style.border.right {
			*v = 2.;
		}
		if let Val::Px(v) = &mut style.border.top {
			*v = 2.;
		}
		if let Val::Px(v) = &mut style.border.bottom {
			*v = 2.;
		}
	});
	on_out.read().for_each(|pointed| {
		let Ok(mut style) = query_resizer.get_mut(pointed.target()) else {
			return;
		};
		if let Val::Px(v) = &mut style.border.left {
			*v = 1.;
		}
		if let Val::Px(v) = &mut style.border.right {
			*v = 1.;
		}
		if let Val::Px(v) = &mut style.border.top {
			*v = 1.;
		}
		if let Val::Px(v) = &mut style.border.bottom {
			*v = 1.;
		}
	});
}

pub(super) struct ResizeHandler;

impl Component for ResizeHandler {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut ent_mut = world.entity_mut(entity);
			if let Some(mut z_index) = ent_mut.get_mut::<ZIndex>() {
				*z_index = UILayer::RESIZE_BAR;
			}
		});
	}
}
