#![allow(unused)]

use std::path::Path;

use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use rainful_diffolor::ToEmbedPath;

use crate::trait_bevy::{BevyColorTheme, FontTypeSize};

#[derive(Clone)]
pub struct TextBuild {
	text: TextTotal,
	font_size: f32,
	font: TextStylish,
	color: Color,
}

#[derive(Clone)]
enum TextTotal {
	Single(String),
	Multiple(Vec<String>),
}

#[derive(Clone)]
enum TextStylish {
	Medium,
	Bold,
}

impl Default for TextBuild {
	fn default() -> Self {
		Self {
			text: TextTotal::Single("Default Build".to_owned()),
			font_size: FontTypeSize::DESCRIPTION,
			font: TextStylish::Medium,
			color: Color::BEVY_WHITE,
		}
	}
}

impl TextBuild {
	pub fn single(value: &str) -> Self {
		Self {
			text: TextTotal::Single(value.to_owned()),
			..default()
		}
	}
	pub fn multiple(value: Vec<&str>) -> Self {
		let value = value.iter().map(|v| v.to_string()).collect::<Vec<_>>();
		Self {
			text: TextTotal::Multiple(value),
			..default()
		}
	}
	pub fn name(mut self) -> Self {
		self.font_size = FontTypeSize::NAME;
		self
	}
	pub fn bold(mut self) -> Self {
		self.font = TextStylish::Bold;
		self
	}
	pub fn custom_color(mut self, color: Color) -> Self {
		self.color = color;
		self
	}
	pub fn dark(mut self) -> Self {
		self.color = Color::BEVY_BLACK;
		self
	}
}

impl Component for TextBuild {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let menu_text = world.entity(entity).get::<TextBuild>().unwrap().clone();
			let assets = world.resource::<AssetServer>();
			let font_path = Path::new("Font");
			let font_ref_path = match menu_text.font {
				TextStylish::Medium => font_path.join("FiraMono-Medium.ttf").embed(),
				TextStylish::Bold => font_path.join("FiraSans-Bold.ttf").embed(),
			};
			let font = assets.load(font_ref_path);
			let font_size = menu_text.font_size;
			let color = menu_text.color;
			let text_bundle = match menu_text.text {
				TextTotal::Single(value) => TextBundle::from_section(
					value,
					TextStyle {
						font_size,
						color,
						font,
					},
				)
				.with_no_wrap(),
				TextTotal::Multiple(vec_value) => {
					let text_sectioned = vec_value
						.iter()
						.map(|value| {
							TextSection::new(
								value,
								TextStyle {
									font_size,
									color,
									..default()
								},
							)
						})
						.collect::<Vec<_>>();
					TextBundle::from_sections(text_sectioned).with_no_wrap()
				},
			};
			world
				.commands()
				.entity(entity)
				.insert(text_bundle)
				.remove::<TextBuild>();
		});
	}
}
