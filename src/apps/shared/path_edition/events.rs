use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Serialize, Deserialize)]
pub struct RequestingPointRelation(pub Entity);

impl MapEntities for RequestingPointRelation {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.0 = entity_mapper.map_entity(self.0);
	}
}
