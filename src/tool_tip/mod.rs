pub mod lib;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use focus::HoverMap;
use i_cant_believe_its_not_bsn::WithChild;
use lib::*;

use crate::{
	camera_control::lib::GlobalScreen2D,
	trait_bevy::{BevyColorTheme, FontTypeSize},
	ui::EffectItem,
};

pub(super) struct ToolInfoPlugin;
impl Plugin for ToolInfoPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<ActiveContent>()
			.add_systems(Startup, spawn_ui_content_displayer)
			.add_systems(PostUpdate, content_ui);
	}
}

fn spawn_ui_content_displayer(mut cmd: Commands) {
	let text_in = |value: &str, font_size: f32| {
		TextSection::new(
			value,
			TextStyle {
				font_size,
				color: Srgba::BEVY_WHITE,
				..default()
			},
		)
	};
	cmd.spawn((
		NodeBundle {
			style: Style {
				padding: UiRect::all(Val::Px(3.0)),
				border: UiRect::all(Val::Px(1.0)),
				margin: UiRect::all(Val::Px(1.0)),
				position_type: PositionType::Absolute,
				..default()
			},
			z_index: ZIndex::Local(50),
			border_radius: BorderRadius::all(Val::Px(3.0)),
			..default()
		},
		ContentDisplayerUI,
		Pickable::IGNORE,
		EffectItem::Regular,
		WithChild((
			TextBundle::from_sections([
				text_in("\n", FontTypeSize::NAME),
				text_in("", FontTypeSize::DESCRIPTION),
			]),
			ContentDisplayerUI,
			EffectItem::Regular,
			Pickable::IGNORE,
		)),
	));
}

fn content_ui(
	mut screen2d: GlobalScreen2D,
	mut query_text_display: Query<&mut Text, With<ContentDisplayerUI>>,
	mut query_content_displayer: Query<
		(&mut Style, &mut Visibility, &Node),
		(With<ContentDisplayerUI>, Without<Text>),
	>,
	mut active_content: ResMut<ActiveContent>,
	query_has_tip: Query<&ToolTipContent>,
	hover_map: Res<HoverMap>,
) {
	let (Ok((mut style, mut visibility, node)), Ok(mut texted)) = (
		query_content_displayer.get_single_mut(),
		query_text_display.get_single_mut(),
	) else {
		return;
	};
	for (_, hover_mapped) in hover_map.iter() {
		let mapped = hover_mapped
			.keys()
			.find_map(|ent_first| query_has_tip.get(*ent_first).ok().zip(Some(*ent_first)));

		*visibility = if active_content
			.0
			.zip(mapped)
			.is_some_and(|(ent, (_, active_ent))| ent == active_ent)
		{
			let distance = Vec2::splat(25.0);
			let window_size = screen2d.single().size();
			let mut pos = screen2d
				.cursor_ui()
				.map(|cur| cur + distance)
				.unwrap_or_default();

			let node_size = node.size();
			let calculated_corner = pos + node_size;

			if calculated_corner.x > window_size.x {
				pos.x -= node_size.x;
			}

			if calculated_corner.y > window_size.y {
				pos.y -= node_size.y * 2.0 + distance.y;
			}

			style.left = Val::Px(pos.x.max(0.0));
			style.top = Val::Px(pos.y.max(node_size.y));

			Visibility::Inherited
		} else {
			active_content.0.take();
			texted.sections[0].value.clear();
			texted.sections[1].value.clear();

			if let Some((content, ent)) = mapped {
				let mut modified_name = content.tool_name();
				if !modified_name.is_empty() {
					modified_name.push('\n');
				}
				texted.sections[0].value = modified_name;
				texted.sections[1].value = content.tool_tip().to_string();
				active_content.0 = Some(ent);
				return;
			}

			Visibility::Hidden
		};
	}
}
