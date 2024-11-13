use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use image::RgbaImage;
use lightyear::prelude::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::EnumIter;

use super::*;

#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ObjectImage;

#[derive(Component)]
pub struct PreviousImage {
	pub pos: Vec2,
	pub img: Handle<Image>,
}

impl PreviousImage {
	pub fn new(pos: Vec2, img: Handle<Image>) -> Self {
		Self { pos, img }
	}
}

#[derive(Bundle, Default)]
pub struct RdioImageBundle {
	mark: ObjectImage,
	object: NetObjectBundle,
	pixel_lock: PixelLock,
	alpha_lock: AlphaLock,
	process_image: ProcessImage,
	in_use: InUse,
	position: ObjectPosition,
	pos_z: ObjectZLayer,
}

impl RdioImageBundle {
	pub fn new(name: &str, size: UVec2, buf: Vec<u8>, position: Vec2, owner: ClientId) -> Self {
		let img = RgbaImage::from_vec(size.x, size.y, buf).unwrap();
		Self {
			object: NetObjectBundle::new(name, owner),
			process_image: ProcessImage(img),
			position: ObjectPosition(position),
			..default()
		}
	}
}

#[derive(Component, Clone, Default, Deref, DerefMut, Serialize, Deserialize, PartialEq)]
pub struct PixelLock(pub HashSet<ClientId>);

#[derive(Component, Clone, Default, Deref, DerefMut, Serialize, Deserialize, PartialEq)]
pub struct AlphaLock(pub HashSet<ClientId>);

#[derive(Component, Reflect, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct BrushScale(i8);

impl BrushScale {
	pub fn get(&self) -> i8 {
		self.0
	}
	pub fn add(&mut self, value: i8) {
		self.0 = self.0.checked_add(value).unwrap_or(self.0).clamp(1, 100);
	}
}

impl Default for BrushScale {
	fn default() -> Self {
		Self(1)
	}
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub enum BrushRef {
	/// Alpha bit
	CustomAlpha {
		brush: DataHold,
		size: UVec2,
	},
	CustomColor {
		brush: DataHold,
		size: UVec2,
	},
	Circle,
}

#[derive(Component, Reflect, Default, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct HardEdgeDraw(pub bool);

impl Default for BrushRef {
	fn default() -> Self {
		Self::CustomAlpha {
			brush: DataHold::Uncompress(vec![255]),
			size: UVec2::ONE,
		}
	}
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct DrawPiled(pub RgbaImage);

#[derive(Component, Reflect, Default, Clone, Copy, EnumIter, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub enum DrawType {
	#[default]
	Normal,
	Replace,
	// Overlay,
	Behind,
}

#[derive(Default, Deref, DerefMut)]
pub struct ProcessImage(pub RgbaImage);

impl Component for ProcessImage {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let process_img = world.entity(entity).get::<ProcessImage>().unwrap();
			let size = process_img.dimensions().into();
			let data = process_img.0.clone().into_vec();
			let mut image_assets = world.resource_mut::<Assets<Image>>();
			let handle_img = image_assets.rgba8_image(data, size);
			world.commands().entity(entity).insert((
				Sprite {
					anchor: Anchor::TopLeft,
					..default()
				},
				handle_img,
			));
		});
	}
}

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct InUse(pub HashSet<ClientId>);

#[derive(Component, Reflect, Default, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct BlurScale(pub f32);
