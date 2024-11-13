pub mod components;
pub mod events;

use client::Predicted;
pub use components::*;
pub use events::*;

use bevy::prelude::*;
use lightyear::prelude::MainSet;

use super::*;

pub(super) struct PublicPlugin;
impl Plugin for PublicPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ObjectActionNet>()
			.add_systems(
				Update,
				(opacity_obj, insert_view_object.after(MainSet::Receive)),
			)
			.add_systems(
				PostUpdate,
				(object_z_to_transform, object_position_to_transform),
			);
		app.world_mut()
			.register_component_hooks::<Transform>()
			.on_add(|mut world, entity, _component_id| {
				let mut ent_mut = world.entity_mut(entity);
				let op_pos = ent_mut.get::<ObjectPosition>().map(|obj_pos| obj_pos.0);
				let op_z = ent_mut.get::<ObjectZLayer>().map(|obj_z| obj_z.0);
				let mut transform = ent_mut.get_mut::<Transform>().unwrap();
				if let Some(z) = op_z {
					transform.translation.z = z as f32;
				}
				if let Some(Vec2 { x, y }) = op_pos {
					transform.translation.x = x;
					transform.translation.y = y;
				}
			});
	}
}

fn insert_view_object(
	mut cmd: Commands,
	query_object: Query<
		(Entity, Has<ObjectPoint>),
		(
			Without<Predicted>,
			Or<(
				Added<TextValue>,
				Added<ProcessImage>,
				Added<ObjectPoint>,
				Added<ObjectPath>,
			)>,
		),
	>,
) {
	query_object.iter().for_each(|(ent_obj, is_point)| {
		let mut ent_cmd = cmd.entity(ent_obj);
		ent_cmd.insert(ObjectWorld);
		if !is_point {
			ent_cmd.insert(SpatialBundle::default());
		}
	});
}

fn object_z_to_transform(
	mut query_object: Query<(&mut Transform, &ObjectZLayer), Changed<ObjectZLayer>>,
) {
	query_object
		.iter_mut()
		.for_each(|(mut transform, z_layer)| {
			transform.translation.z = z_layer.0 as f32;
		});
}

fn object_position_to_transform(
	mut query_object: Query<(&mut Transform, &ObjectPosition), Changed<ObjectPosition>>,
) {
	query_object
		.iter_mut()
		.for_each(|(mut transform, position)| {
			transform.translation.x = position.x;
			transform.translation.y = position.y;
		});
}

fn opacity_obj(
	mut query_obj: Query<
		(&ObjectOpacity, Option<&mut Sprite>),
		(With<ObjectWorld>, Changed<ObjectOpacity>),
	>,
) {
	query_obj.iter_mut().for_each(|(opacity, op_sprite)| {
		if let Some(mut sprite) = op_sprite {
			sprite.color.set_alpha(opacity.0 as f32 / 100.0);
		}
	});
}
