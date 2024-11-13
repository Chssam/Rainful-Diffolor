use aery::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::{Up as PickUp, *};

use super::*;

#[derive(Relation, Clone, Copy)]
#[aery(Recursive)]
pub struct ObjectRelationUI;

#[derive(Component)]
pub struct IconVisiblity;

#[derive(Component)]
pub struct IconMoveLock;

#[derive(Component)]
pub struct BrushCollector;

#[derive(Event)]
pub struct NewBrush(pub BrushRef, pub UVec2, pub Vec<u8>);

#[derive(Reflect)]
pub(super) enum AttactObject {
	Pick,
	LockPick,
	#[reflect(@ToolPath("gimp-visible.png"))]
	Visibility,
}

impl AttactObject {
	pub fn target_up(&self, ent_obj: Entity) -> On<Pointer<PickUp>> {
		match self {
			AttactObject::Pick => On::<Pointer<PickUp>>::run(
				move |mut query_user: Query<&mut SelectedObject, With<MainUser>>| {
					let Ok(mut selected_obj) = query_user.get_single_mut() else {
						return;
					};
					if selected_obj.group.contains(&ent_obj) {
						selected_obj.deselect_single(ent_obj);
						return;
					}
					selected_obj.select_single(ent_obj);
				},
			),
			AttactObject::LockPick => {
				On::<Pointer<PickUp>>::run(move |mut query_object: Query<&mut Pickable>| {
					let Ok(mut pick_lock) = query_object.get_mut(ent_obj) else {
						return;
					};
					pick_lock.is_hoverable = !pick_lock.is_hoverable;
				})
			},
			AttactObject::Visibility => {
				On::<Pointer<PickUp>>::run(move |mut query_object: Query<&mut Visibility>| {
					let Ok(mut visibility) = query_object.get_mut(ent_obj) else {
						return;
					};
					*visibility = match *visibility {
						Visibility::Hidden => Visibility::Inherited,
						_ => Visibility::Hidden,
					}
				})
			},
		}
	}
}

#[derive(Component)]
pub(super) struct HexColorText;

#[derive(Component, Clone)]
pub(super) struct ColorChoice;

#[derive(Component)]
pub(super) struct DisplayObjectDirectory;

#[derive(Component)]
pub enum DisplayColor {
	Foreground,
	Background,
}

impl ObjectActionNet {
	pub fn target_up(self, obj_ent: Entity) -> On<Pointer<PickUp>> {
		On::<Pointer<PickUp>>::run(move |mut client: ResMut<ClientConnectionManager>| {
			client
				.send_message_to_target::<MainChannel, ObjectActionToServer>(
					&mut ObjectActionToServer {
						obj_ent,
						action: self,
					},
					NetworkTarget::All,
				)
				.unwrap_or_else(|e| {
					error!("Fail to send message: {:?}", e);
				});
		})
	}
}
