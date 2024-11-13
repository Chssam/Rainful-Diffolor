use std::{collections::HashSet, path::PathBuf};

use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use bevy_mod_picking::prelude::*;
use i_cant_believe_its_not_bsn::Maybe;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::trait_bevy::ToolPath;

#[derive(Component, Deref, DerefMut)]
pub struct SaveLocation(pub PathBuf);

// pub struct UserUniqueID(pub u32);

// pub struct UserLogin {
// 	name: &'static str,
// 	password: &'static str,
// }

// pub struct UserIdentityLogin(pub HashMap<UserLogin, UserUniqueID>);

// pub enum UserLogAs {
// 	New,
// 	Login(UserLogin),
// }

// Or Reverted
// pub trait ClientIdToUniId {

// }

#[derive(Component, Clone, Deref, Serialize, Deserialize, PartialEq)]
pub struct ObjectOwner(pub ClientId);

#[derive(Component, Clone, Deref, DerefMut, Serialize, Deserialize, PartialEq)]
pub struct ObjectAccess(pub NetworkTarget);

pub trait EditPermission {
	fn add_id(&mut self, ref_client_id: ClientId);
	fn remove_id(&mut self, ref_client_id: &ClientId);
}

impl EditPermission for NetworkTarget {
	fn add_id(&mut self, ref_client_id: ClientId) {
		match self {
			NetworkTarget::None => {},
			NetworkTarget::AllExceptSingle(client_id) => *client_id = ref_client_id,
			NetworkTarget::AllExcept(vec) => {
				vec.push(ref_client_id);
				vec.dedup();
			},
			NetworkTarget::All => {},
			NetworkTarget::Only(vec) => {
				vec.push(ref_client_id);
				vec.dedup();
			},
			NetworkTarget::Single(client_id) => *client_id = ref_client_id,
		}
	}
	fn remove_id(&mut self, ref_client_id: &ClientId) {
		match self {
			NetworkTarget::None => {},
			NetworkTarget::AllExceptSingle(_client_id) => *self = NetworkTarget::None,
			NetworkTarget::AllExcept(vec) => {
				if let Some(pos) = vec.iter().position(|client_id| client_id == ref_client_id) {
					vec.remove(pos);
				}
			},
			NetworkTarget::All => {},
			NetworkTarget::Only(vec) => {
				if let Some(pos) = vec.iter().position(|client_id| client_id == ref_client_id) {
					vec.remove(pos);
				}
			},
			NetworkTarget::Single(_client_id) => *self = NetworkTarget::None,
		}
	}
}

#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct ObjectZLayer(pub i16);

#[derive(
	Component, Reflect, Clone, Default, Deref, DerefMut, Serialize, Deserialize, PartialEq,
)]
pub struct ObjectPosition(pub Vec2);

#[derive(Clone, Default)]
pub struct ObjectWorld;

impl Component for ObjectWorld {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			world.commands().entity(entity).insert(PickableBundle {
				pickable: Pickable {
					should_block_lower: false,
					..default()
				},
				..default()
			});
		});
	}
}

#[derive(Component, Clone, Deref, Serialize, Deserialize, PartialEq)]
pub struct SharingName(pub String);

impl Default for SharingName {
	fn default() -> Self {
		Self("Nameless".to_owned())
	}
}

// #[derive(Component, Default)]
// pub struct PendingObjectWorld;

#[derive(Component, Clone, Default, Deref, DerefMut, Serialize, Deserialize, PartialEq)]
pub struct MoveLock(pub HashSet<ClientId>);

#[derive(Bundle, Default)]
pub struct NetObjectBundle {
	object: ObjectWorld,
	name: SharingName,
	move_lock: MoveLock,
	opacity: ObjectOpacity,
	owner: Maybe<(ObjectOwner, ObjectAccess)>,
}

impl NetObjectBundle {
	pub fn new(name: &str, owner: ClientId) -> Self {
		Self {
			name: SharingName(name.to_owned()),
			owner: Maybe::new((ObjectOwner(owner), ObjectAccess(NetworkTarget::All))),
			..default()
		}
	}
}

#[derive(Component, Clone, PartialEq, Deserialize, Serialize)]
pub struct ObjectOpacity(pub i8, pub i8);

impl Default for ObjectOpacity {
	fn default() -> Self {
		Self(100, 100)
	}
}

#[allow(clippy::enum_variant_names)]
#[derive(Event, Reflect, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObjectActionNet {
	#[reflect(@ToolPath("gimp-tool-move.png"))]
	LockMove,
	LockPixel,
	LockAlpha,
	#[reflect(@ToolPath("go-up.png"))]
	LayerUp,
	#[reflect(@ToolPath("go-down.png"))]
	LayerDown,
}

#[derive(Event, Reflect, Clone, Copy, Serialize, Deserialize)]
pub enum ObjectBirNet {
	Undo,
	Redo,
}
