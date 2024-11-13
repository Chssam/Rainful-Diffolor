use aery::prelude::*;
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::prelude::{ClientId, NetworkTarget};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use super::ObjectActionNet;

#[derive(Event, Clone, Serialize, Deserialize)]
pub struct ConnectRelations<T: Relation + Clone> {
	pub ent1: Entity,
	pub ent2: Entity,
	_phantom: PhantomData<T>,
}

impl<T: Relation + Clone> ConnectRelations<T> {
	pub fn new(ent1: Entity, ent2: Entity) -> Self {
		Self {
			ent1,
			ent2,
			_phantom: default(),
		}
	}
}

impl<T: Relation + Clone> MapEntities for ConnectRelations<T> {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.ent1 = entity_mapper.map_entity(self.ent1);
		self.ent2 = entity_mapper.map_entity(self.ent2);
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToClientEntDataEvent<T: Clone> {
	pub ent: Entity,
	pub data: T,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectActionToServer {
	pub obj_ent: Entity,
	pub action: ObjectActionNet,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PerActionNet {
	pub obj_ent: Entity,
	pub action: PerAction,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PerAction {
	Add(ClientId),
	Remove(ClientId),
	Change(NetworkTarget),
}

impl<T: Clone> ToClientEntDataEvent<T> {
	pub fn new(data: T, ent: Entity) -> Self {
		Self { ent, data }
	}
}

impl<T: Clone> MapEntities for ToClientEntDataEvent<T> {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.ent = entity_mapper.map_entity(self.ent);
	}
}

impl MapEntities for ObjectActionToServer {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.obj_ent = entity_mapper.map_entity(self.obj_ent);
	}
}

impl MapEntities for PerActionNet {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.obj_ent = entity_mapper.map_entity(self.obj_ent);
	}
}
