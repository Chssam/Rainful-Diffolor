use std::{collections::HashMap, ops::Not};

use crate::{apps::shared::*, trait_bevy::AyuColorTheme};
use aery::prelude::*;
use bevy::{
	color::palettes::css::{BROWN, GREEN, RED},
	prelude::*,
	render::primitives::Aabb,
};
use bevy_vector_shapes::prelude::*;
use prelude::*;

use super::DottedGizmo;

pub(super) fn draw_point(
	query_point: Query<(Entity, &ObjectPosition), (With<ObjectPoint>, With<ObjectWorld>)>,
	query_user: Query<(&SelectedObject, &PaintInk), With<UserId>>,
	mut gizmos: Gizmos,
) {
	let mut sorted_point: HashMap<Entity, &ObjectPosition> = query_point.iter().collect();
	query_user.iter().for_each(|(selected_obj, paint)| {
		selected_obj.group.iter().for_each(|point| {
			let Ok((point, position)) = query_point.get(*point) else {
				return;
			};
			let color = if selected_obj.single.is_some_and(|sin| sin == point) {
				paint.0
			} else {
				paint.1
			}
			.with_alpha(1.0);
			gizmos.circle_2d(position.xy(), PICK_RANGE, color);
			sorted_point.remove(&point);
		});
	});

	sorted_point.iter().for_each(|(_, position)| {
		gizmos.circle_2d(position.xy(), PICK_RANGE, BROWN);
	});
}

pub(super) fn draw_point_line(
	mut gizmos: Gizmos,
	query_point: Query<((&ObjectPosition, &PointType), Relations<PointToPoint>), With<ObjectPoint>>,
	items: Query<Entity, Root<PointToPoint>>,
) {
	query_point
		.traverse::<PointToPoint>(items.iter())
		.track_self()
		.for_each(|(parent, op_path_parent), _, (child, op_path_child), _| {
			use PointType::*;
			match (op_path_parent, op_path_child) {
				(LineTo, LineTo) => {
					gizmos
						.arrow_2d(parent.xy(), child.xy(), Color::WHITE)
						.with_tip_length(4.0);
				},
				_ => {
					gizmos
						.arrow_2d(parent.xy(), child.xy(), Color::WHITE)
						.with_tip_length(4.0);
				},
				// (None, Some(path_child)) => match path_child {
				// 	PathType::QuadraticBezier { to } => todo!(),
				// 	PathType::CubricBezier { ctrl1, ctrl2, to } => todo!(),
				// 	PathType::Arc {
				// 		radii,
				// 		sweep_angle,
				// 		x_rotation,
				// 	} => todo!(),
				// },
				// (Some(path_parent), None) => match path_child {
				// 	PathType::QuadraticBezier { to } => todo!(),
				// 	PathType::CubricBezier { ctrl1, ctrl2, to } => todo!(),
				// 	PathType::Arc {
				// 		radii,
				// 		sweep_angle,
				// 		x_rotation,
				// 	} => todo!(),
				// },
				// (Some(path_parent), Some(path_child)) => match (path_parent, path_child) {
				// 	(PathType::QuadraticBezier { to }, PathType::QuadraticBezier { to }) => todo!(),
				// 	(
				// 		PathType::QuadraticBezier { to },
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 	) => todo!(),
				// 	(
				// 		PathType::QuadraticBezier { to },
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 	) => todo!(),
				// 	(
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 		PathType::QuadraticBezier { to },
				// 	) => todo!(),
				// 	(
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 	) => todo!(),
				// 	(
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 	) => gizmos.arc_2d(*center, *sweep_angle, *x_rotation, radii.to_angle(), fore),
				// 	(
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 		PathType::QuadraticBezier { to },
				// 	) => gizmos.arc_2d(*center, *sweep_angle, *x_rotation, radii.to_angle(), fore),
				// 	(
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 		PathType::CubricBezier { ctrl1, ctrl2, to },
				// 	) => todo!(),
				// 	(
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 		PathType::Arc {
				// 			radii,
				// 			sweep_angle,
				// 			x_rotation,
				// 		},
				// 	) => todo!(),
				// },
			}
		});
}

pub(super) fn draw_user_cursor(
	mut gizmos: Gizmos,
	query_user: Query<(&CursorPos, &PaintInk)>,
	physics_cam: Query<&OrthographicProjection, With<Camera2d>>,
) {
	let scale_cam = physics_cam.single();
	query_user.iter().for_each(|(cur_pos, paint)| {
		let pos = cur_pos.xy();
		gizmos.circle_2d(pos, 8.0 * scale_cam.scale, paint.0.with_alpha(1.0));
		gizmos.circle_2d(pos, 10.0 * scale_cam.scale, paint.1.with_alpha(1.0));
	});
}

pub(super) fn draw_xy_line(mut dot_gizmos: Gizmos<DottedGizmo>) {
	let max = 1e+9;
	dot_gizmos.line_2d(Vec2::X * max, Vec2::NEG_X * max, RED);
	dot_gizmos.line_2d(Vec2::Y * max, Vec2::NEG_Y * max, GREEN);
}

pub(super) fn selected_obj_outline(
	query_user: Query<(&SelectedObject, &PaintInk), With<UserId>>,
	query_object: Query<AnyOf<((&GlobalTransform, &Aabb), &ObjectPosition)>, With<ObjectWorld>>,
	mut dot_gizmos: Gizmos<DottedGizmo>,
) {
	query_user.iter().for_each(|(selected_obj, paint)| {
		let mut cover_all: Option<Rect> = None;
		selected_obj.group.iter().for_each(|ent_obj| {
			let Ok((op_obj, op_pos)) = query_object.get(*ent_obj) else {
				return;
			};
			if let Some((obj_transform, aabb)) = op_obj {
				let position = obj_transform.translation().truncate() + aabb.center.xy();
				let size = aabb.half_extents.truncate() * 2.0;
				let rected = Rect::from_center_size(position, size);
				if let Some(recty) = &mut cover_all {
					*recty = recty.union(rected);
				} else {
					cover_all = Some(rected);
				}
				let true_paint = if &selected_obj.single.unwrap() == ent_obj {
					paint.0
				} else {
					paint.1
				}
				.with_alpha(1.0);
				dot_gizmos.rect_2d(position, 0.0, size, true_paint);
			}
			if let Some(point_pos) = op_pos {
				if let Some(recty) = &mut cover_all {
					*recty = recty.union_point(point_pos.xy());
				} else {
					cover_all = Some(Rect::from_center_size(point_pos.xy(), Vec2::ZERO));
				}
			}
		});
		if let Some(recty) = cover_all {
			let center_pos = recty.center();
			let size = recty.size();
			dot_gizmos.rect_2d(center_pos, 0.0, size, paint.1.darker(0.5));
		}
	});
}

pub(super) fn draw_selecting_box(
	query_user: Query<(&CursorPos, &BeginSelectPoint, &PaintInk), With<UserId>>,
	mut gizmos: Gizmos,
) {
	query_user
		.iter()
		.for_each(|(user_pos, begin_select, paint)| {
			let Some(pin_point) = begin_select.0 else {
				return;
			};
			let color = paint.0.with_alpha(1.0);
			let real_world_ray = user_pos.xy();
			let rect = Rect::from_corners(real_world_ray, pin_point);
			gizmos.rect_2d(rect.center(), 0.0, rect.size(), color);
		});
}

pub(super) fn draw_cropper(
	mut dot_gizmos: Gizmos<DottedGizmo>,
	query_user: Query<(&Selection, &PaintInk), With<UserId>>,
) {
	query_user.iter().for_each(|(selection, paint)| {
		let (Some(pin_begin), Some(pin_end)) = (selection.0, selection.1) else {
			return;
		};
		let rect_box = Rect::from_corners(pin_begin, pin_end);
		dot_gizmos.rect_2d(rect_box.center(), 0.0, rect_box.size(), paint.0);
	});
}

pub(super) fn shape_drawer(
	mut painter: ShapePainter,
	mut mark_source: ResMut<MarkerDisplay>,
	time: Res<Time>,
) {
	painter.cap = Cap::Round;
	painter.thickness = 4.0;
	let far = 1000.0;
	mark_source.0 = mark_source
		.drain(..)
		.filter_map(|mut marker| {
			let ticked_time = marker.timer.tick(time.delta());
			let remain_time = ticked_time.remaining().as_secs_f32();
			let ori_time = ticked_time.duration().as_secs_f32();
			let fade_away = remain_time / ori_time;
			let color = Color::AYU_LIME.with_alpha(fade_away);
			painter.set_color(color);

			match marker.mark_type {
				MarkerType::Circle(pos) => {
					let expand = 4.0 + 4.0 * remain_time.sin();
					painter.set_translation(pos.extend(far));
					painter.circle(expand);
				},
				MarkerType::Line(from_to) => {
					painter.set_translation(Vec2::ZERO.extend(far));
					let CursorFromTo { from, to } = from_to;
					let start = from.extend(far);
					let end = to.extend(far);
					painter.line(start, end);
				},
			};
			ticked_time.finished().not().then_some(marker)
		})
		.collect::<Vec<_>>();
}

pub(super) fn receive_marker_draw(
	mut events_client: EventReader<ClientMessageEvent<MarkerType>>,
	mut events_server: EventReader<ServerMessageEvent<MarkerType>>,
	mut mark_source: ResMut<MarkerDisplay>,
) {
	events_client
		.read()
		.map(|a| a.message())
		.chain(events_server.read().map(|a| a.message()))
		.for_each(|mark_type| {
			mark_source.push_stroke(*mark_type);
		});
}
