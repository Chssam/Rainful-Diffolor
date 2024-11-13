use bevy::prelude::*;
use lightyear::prelude::ClientId;
use serde::{Deserialize, Serialize};

use crate::apps::shared::{prelude::NetObjectBundle, ObjectPosition, ObjectZLayer};

#[derive(Bundle, Default)]
pub struct WorldTextBundle {
	object: NetObjectBundle,
	value: TextValue,
	position: ObjectPosition,
	pos_z: ObjectZLayer,
}

#[derive(Component, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextValue(pub String);

impl WorldTextBundle {
	pub fn new(name: &str, value: String, position: Vec2, owner: ClientId) -> Self {
		Self {
			object: NetObjectBundle::new(name, owner),
			value: TextValue(value),
			position: ObjectPosition(position),
			..default()
		}
	}
}
