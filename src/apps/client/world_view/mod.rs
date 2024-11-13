use bevy::prelude::*;

use crate::apps::shared::helperful_tool::{hide_ent, unhide_ent};

use super::{DisplayBrush, EditorTools, RdioClientState};
use crate::apps::client::*;
use bevy::render::{primitives::Aabb, render_resource::Extent3d};

pub(super) struct LocalViewPlugin;
impl Plugin for LocalViewPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				draw_grid_box,
				// change_obj_wire_color,
				// change_selected_wire,
				// mesh2d_image
				// draw_grid_box,
			)
				.run_if(in_state(RdioClientState::Online)),
		)
		.add_systems(
			Last,
			display_brush_position.run_if(in_state(EditorTools::Pencel)),
		)
		.add_systems(OnEnter(EditorTools::Pencel), unhide_ent::<DisplayBrush>)
		.add_systems(OnExit(EditorTools::Pencel), hide_ent::<DisplayBrush>);
	}
}

fn draw_grid_box(
	mut gizmos: Gizmos,
	query_user: Query<(&SelectedObject, &VisualGrid, &PaintInk), With<MainUser>>,
	query_object: Query<(&ObjectPosition, &Aabb), (With<ObjectWorld>, With<Handle<Image>>)>,
) {
	let Ok((selected_obj, visual_grid, paint)) = query_user.get_single() else {
		return;
	};

	let Some(Ok((obj_pos, aabb))) = selected_obj.single.map(|ent_obj| query_object.get(ent_obj))
	else {
		return;
	};
	if !visual_grid.0 {
		return;
	}
	let position = obj_pos.0 + aabb.center.xy();
	let size = aabb.half_extents.truncate().as_uvec2() * 2;
	gizmos
		.grid_2d(
			position,
			0.0,
			size,
			Vec2::ONE,
			paint.1.with_alpha(1.0).darker(0.1),
		)
		.outer_edges();
}

fn display_brush_position(
	mut image_assets: ResMut<Assets<Image>>,
	mut query_display_brush: Query<(&mut Transform, &mut Aabb, &Handle<Image>), With<DisplayBrush>>,
	query_user: Query<(Ref<CursorPos>, Ref<DrawPiled>), With<MainUser>>,
) {
	let Ok((mut display_transform, mut aabb, handle_img)) = query_display_brush.get_single_mut()
	else {
		return;
	};
	let Ok((cur_pos, draw_piled)) = query_user.get_single() else {
		return;
	};

	let (width, height) = draw_piled.dimensions();
	if draw_piled.is_changed() {
		let imged = image_assets.get_mut(handle_img).unwrap();
		imged.resize(Extent3d {
			width,
			height,
			..default()
		});
		imged.data = draw_piled.to_vec();

		let Vec2 { x, y } = imged.size_f32() / 2.0;
		aabb.half_extents.x = x;
		aabb.half_extents.y = y;
		aabb.center.x = x;
		aabb.center.y = -y;
	}

	if cur_pos.is_changed() {
		let uvec: UVec2 = (width, height).into();
		let Vec2 { x, y } = (uvec / 2).as_vec2();

		display_transform.translation.x = cur_pos.x.floor() - x;
		display_transform.translation.y = cur_pos.y.ceil() + y;
	}
}
