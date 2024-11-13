use std::{
	collections::{HashMap, HashSet},
	sync::{Arc, RwLock},
};

use bevy::prelude::*;
use leafwing_input_manager::Actionlike;
use lightyear::prelude::ClientId;

#[derive(Resource, Default)]
pub struct ObjectIncrementCount(pub u64);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Users(HashMap<ClientId, Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ObjectOrderZ(Vec<Entity>);

#[derive(Resource, Deref, DerefMut)]
pub struct BackendTaskServer(pub Arc<RwLock<BackendItem>>);

pub struct BackendItem {
	pub clients: HashSet<u64>,
	pub inactive: bool,
	pub password: Option<String>,
	pub password_admin: String,
}

impl Default for BackendItem {
	fn default() -> Self {
		Self {
			clients: HashSet::default(),
			inactive: false,
			password: None,
			password_admin: "Admin".to_owned(),
		}
	}
}

#[derive(Actionlike, Component, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ServerAction {
	Disconnect,
	Admin,
}

#[derive(Component)]
pub struct UserAdmin;

#[derive(Event)]
pub struct DisconnectClient(pub ClientId);

// #[derive(Component, Serialize, Deserialize, Debug, Clone)]
// struct UniqueUserKey([char; 16]);

// struct DataSaveUser {
//     unique_key: UniqueUserKey,
// }
