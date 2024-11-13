// use bevy::{prelude::*, sprite::Anchor};
// use vleue_kinetoscope::*;

// use super::NetObjectBundle;

// #[derive(Bundle)]
// pub struct RdioAnimatedImageBundle {
// 	object: NetObjectBundle,
// 	animated_bundle: AnimatedImageBundle,
// }

// impl RdioAnimatedImageBundle {
// 	pub fn new(name: &str, a_handle_image: Handle<AnimatedImage>, position: Vec2) -> Self {
// 		Self {
// 			object: NetObjectBundle::new(name),
// 			animated_bundle: AnimatedImageBundle {
// 				transform: Transform::from_translation(position),
// 				animated_image: a_handle_image,
// 				sprite: Sprite {
// 					anchor: Anchor::TopLeft,
// 					..default()
// 				},
// 				..default()
// 			},
// 		}
// 	}
// }
