pub mod components;
pub mod events;

pub use components::*;
// pub use events::*;

use bevy::{prelude::*, sprite::Anchor};
use lightyear::prelude::client::Predicted;

use super::ObjectWorld;

pub(super) struct TextWorldPlugin;
impl Plugin for TextWorldPlugin {
	fn build(&self, app: &mut App) {
		app.observe(new_text).add_systems(Update, update_text);
	}
}

fn new_text(
	trigger: Trigger<OnAdd, TextValue>,
	mut cmd: Commands,
	query_object: Query<&TextValue, Without<Predicted>>,
) {
	let ent_obj = trigger.entity();
	let Ok(value) = query_object.get(ent_obj) else {
		return;
	};
	cmd.entity(ent_obj).insert(Text2dBundle {
		text: Text::from_section(
			value.0.clone(),
			TextStyle {
				font_size: 30.0,
				color: Color::BLACK,
				..default()
			},
		),
		text_anchor: Anchor::TopLeft,
		..default()
	});
}

fn update_text(
	mut query_text: Query<(&mut Text, &TextValue), (With<ObjectWorld>, Changed<TextValue>)>,
) {
	query_text.iter_mut().for_each(|(mut text, value_text)| {
		text.sections[0].value = value_text.0.clone();
	});
}
