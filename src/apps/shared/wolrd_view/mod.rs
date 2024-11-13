mod view_render;
use super::MarkerDisplay;
use bevy::prelude::*;
use view_render::*;

pub(super) struct LocalViewPlugin;
impl Plugin for LocalViewPlugin {
	fn build(&self, app: &mut App) {
		app.init_gizmo_group::<DottedGizmo>()
			.init_resource::<MarkerDisplay>()
			.add_systems(Startup, setup_gizmo)
			.add_systems(
				Update,
				(
					draw_user_cursor,
					draw_point_line,
					draw_point,
					draw_cropper,
					draw_selecting_box,
					selected_obj_outline,
					shape_drawer,
					receive_marker_draw,
					// change_obj_wire_color,
					// change_selected_wire,
					// mesh2d_image
					// draw_grid_box,
				),
			)
			.add_systems(FixedUpdate, draw_xy_line);
	}
}

fn setup_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
	let (dot_config, _) = config_store.config_mut::<DottedGizmo>();
	dot_config.line_style = GizmoLineStyle::Dotted;
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct DottedGizmo;
