#![allow(unused)]
use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};

use super::*;

/// Removed Once Done
#[derive(Clone)]
pub struct DropDownAt(Entity, DisplayPos);

impl DropDownAt {
	fn bundle() -> impl Bundle {
		(EffectUIBundle::column().auto_size(), SubMenu)
	}
	pub fn bottom(ent_ui: Entity) -> impl Bundle {
		(DropDownAt::bundle(), DropDownAt(ent_ui, DisplayPos::Bottom))
	}
	pub fn right(ent_ui: Entity) -> impl Bundle {
		(DropDownAt::bundle(), DropDownAt(ent_ui, DisplayPos::Right))
	}
}

#[derive(Clone)]
enum DisplayPos {
	Bottom,
	Right,
}

impl Component for DropDownAt {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let DropDownAt(near, pos) = world.entity(entity).get::<DropDownAt>().unwrap().clone();
			let near = world.entity(near);
			let node = near.get::<Node>();
			let global_form = near.get::<GlobalTransform>();

			if let Some(rect) = node
				.zip(global_form)
				.map(|(node, global_form)| node.logical_rect(global_form))
			{
				if let Some(mut style) = world.entity_mut(entity).get_mut::<Style>() {
					let (x, y) = match pos {
						DisplayPos::Bottom => (rect.min.x, rect.max.y),
						DisplayPos::Right => (rect.max.x, rect.min.y),
					};
					style.top = Val::Px(y);
					style.left = Val::Px(x);
				};
			}
			world.commands().entity(entity).remove::<DropDownAt>();
		});
	}
}
