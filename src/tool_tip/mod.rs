pub mod lib;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use i_cant_believe_its_not_bsn::WithChild;
use lib::*;

use crate::{
	camera_control::lib::GlobalScreen2D,
	trait_bevy::{BevyColorTheme, FontTypeSize},
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
	cmd.spawn((
		NodeBundle {
			style: Style {
				padding: UiRect::all(Val::Px(2.0)),
				border: UiRect::all(Val::Px(1.0)),
				position_type: PositionType::Absolute,
				..default()
			},
			border_color: Srgba::BEVY_BLACK.into(),
			background_color: Srgba::BEVY_DARK_GRAY.into(),
			z_index: ZIndex::Global(i32::MAX),
			..default()
		},
		Pickable::IGNORE,
		ContentDisplayerUI,
		WithChild((
			TextBundle::from_sections([
				TextSection::new(
					"\n",
					TextStyle {
						font_size: FontTypeSize::NAME,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
				TextSection::new(
					"",
					TextStyle {
						font_size: FontTypeSize::DESCRIPTION,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
			]),
			Pickable::IGNORE,
			ContentDisplayerUI,
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
	query_has_tip: Query<
		(
			Entity,
			&PickingInteraction,
			Option<&Interaction>,
			&ToolTipContent,
		),
		Changed<PickingInteraction>,
	>,
) {
	let (Ok((mut style, mut visibility, node)), Ok(mut texted)) = (
		query_content_displayer.get_single_mut(),
		query_text_display.get_single_mut(),
	) else {
		return;
	};
	*visibility = if active_content.0.is_some_and(|ent| {
		query_has_tip
			.get(ent)
			.is_ok_and(|(_, pick_inter, op_inter, _)| {
				pick_inter != &PickingInteraction::None
					|| op_inter.is_some_and(|inter| inter != &Interaction::None)
			})
	}) {
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
		Visibility::Hidden
	};
	query_has_tip
		.iter()
		.for_each(|(ent, pick_inter, op_inter, content)| {
			if (pick_inter != &PickingInteraction::None
				|| op_inter.is_some_and(|inter| inter != &Interaction::None))
				&& !active_content.0.is_some_and(|ent_active| ent == ent_active)
			{
				let mut modified_name = content.tool_name();
				if !modified_name.is_empty() {
					modified_name.push('\n');
				}
				texted.sections[0].value = modified_name;
				texted.sections[1].value = content.tool_tip().to_string();
				active_content.0 = Some(ent);
			}
		});
}
