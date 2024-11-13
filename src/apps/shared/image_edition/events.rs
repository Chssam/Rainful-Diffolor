use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::DataHold;

#[derive(Event, Serialize, Deserialize)]
pub struct ImageNetwork {
	pub name: String,
	data: DataHold,
	pub size: UVec2,
}

impl ImageNetwork {
	pub fn new(name: String, data: &[u8], size: UVec2) -> Self {
		Self {
			name,
			data: DataHold::to_compress(data),
			size,
		}
	}
	pub fn data(&self) -> &DataHold {
		&self.data
	}
}
