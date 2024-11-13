use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

use crate::apps::shared::prelude::DataHold;

use super::CursorFromTo;

#[derive(Event, Clone, Copy, Serialize, Deserialize)]
pub struct PenDraw(pub CursorFromTo, pub DrawingWay);

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum DrawingWay {
	Color,
	Erase,
}
// #[derive(Event, Clone, Copy, Serialize, Deserialize)]
// pub struct MarkerDraw(pub PathType);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MarkerDisplay(pub Vec<OnceStroke>);

impl MarkerDisplay {
	pub fn push_stroke(&mut self, mark_type: MarkerType) {
		self.push(OnceStroke::new(mark_type));
	}
}

pub struct OnceStroke {
	pub timer: Timer,
	pub mark_type: MarkerType,
}

impl OnceStroke {
	fn new(mark_type: MarkerType) -> Self {
		Self {
			timer: Timer::from_seconds(4.0, TimerMode::Once),
			mark_type,
		}
	}
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum MarkerType {
	Circle(Vec2),
	Line(CursorFromTo),
}

#[derive(Event, Deref, DerefMut)]
pub struct DisplayMsgEvent(pub String);

#[derive(Clone, Serialize, Deserialize)]
pub struct RequestImageData(pub Entity);

impl MapEntities for RequestImageData {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.0 = entity_mapper.map_entity(self.0);
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveImageData {
	pub ent: Entity,
	data: DataHold,
	pub size: UVec2,
}

impl ReceiveImageData {
	pub fn new(ent: Entity, data: &[u8], size: UVec2) -> Self {
		Self {
			ent,
			data: DataHold::to_compress(data),
			size,
		}
	}
	pub fn data(&self) -> &DataHold {
		&self.data
	}
}

impl MapEntities for ReceiveImageData {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.ent = entity_mapper.map_entity(self.ent);
	}
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyChange(pub bool, pub Entity);

impl MapEntities for ApplyChange {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.1 = entity_mapper.map_entity(self.1);
	}
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageCtx(pub String);

#[derive(Resource, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovedPoint {
	pub world: Vec2,
	pub pixel: IVec2,
}
