use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use bevy_mod_picking::prelude::Pickable;

use crate::trait_bevy::DirectImage;

pub struct IconBuild(f32, Option<DirectImage>);

impl IconBuild {
	pub fn new(data: Vec<u8>, size: UVec2) -> Self {
		Self(20., Some(DirectImage::Image { data, size }))
	}
	pub fn medium(path: String) -> Self {
		Self(20., Some(DirectImage::Path(path)))
	}
	pub fn large(mut self) -> Self {
		self.0 = 28.;
		self
	}
	pub fn extra_large(mut self) -> Self {
		self.0 = 36.;
		self
	}
}

impl Component for IconBuild {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut ent = world.entity_mut(entity);

			let mut icon = ent.get_mut::<IconBuild>().unwrap();
			let size = Val::Px(icon.0);
			let image = icon.1.take().unwrap();
			let all_side = UiRect::all(Val::Px(2.0));
			let image_bundle = (
				ImageBundle {
					style: Style {
						width: size,
						height: size,
						border: all_side,
						margin: all_side,
						..default()
					},
					..default()
				},
				image,
			);
			if !ent.contains::<Pickable>() {
				world.commands().entity(entity).insert(Pickable::IGNORE);
			}
			world
				.commands()
				.entity(entity)
				.insert(image_bundle)
				.remove::<IconBuild>();
		});
	}
}
