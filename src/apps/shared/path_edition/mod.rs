use crate::trait_bevy::BuildCommonImage;

use super::{connect_relations, prelude::*};
use aery::prelude::*;
use backend::{HitData, PointerHits};
use bevy::{
	ecs::system::SystemParam,
	prelude::*,
	render::{primitives::Aabb, render_resource::Extent3d},
	sprite::Anchor,
	window::PrimaryWindow,
};
use bevy_mod_picking::{picking_core::PickSet, prelude::*};
use bevy_prototype_lyon::prelude::*;
use std::collections::HashSet;
use svg::node::element::SVG;
use zeno::Placement;

pub mod components;
pub mod events;
mod lyon_to;

pub use components::*;
pub use events::*;
use lightyear::prelude::client::Predicted;
use lyon_to::*;

pub const PICK_RANGE: f32 = 5.0;

pub(super) struct PathPlugin;
impl Plugin for PathPlugin {
	fn build(&self, app: &mut App) {
		app.observe(connect_relations::<PointToPoint>)
			.add_systems(PreUpdate, point_picking.in_set(PickSet::Backend))
			.add_systems(Update, path_render_as)
			.add_systems(PostUpdate, (update_svg, update_aabb_path).chain());
	}
}

fn point_picking(
	pointers: Query<(&PointerId, &PointerLocation)>,
	cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
	primary_window: Query<Entity, With<PrimaryWindow>>,
	point_query: Query<
		(Entity, &ObjectPosition, &ObjectZLayer, Option<&Pickable>),
		With<ObjectPoint>,
	>,
	mut output: EventWriter<PointerHits>,
) {
	for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
		pointer_location.location().map(|loc| (pointer, loc))
	}) {
		let mut blocked = false;
		let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras
			.iter()
			.filter(|(_, camera, ..)| camera.is_active)
			.find(|(_, camera, ..)| {
				camera
					.target
					.normalize(Some(match primary_window.get_single() {
						Ok(w) => w,
						Err(_) => return false,
					}))
					.unwrap() == location.target
			})
		else {
			continue;
		};

		let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position)
		else {
			continue;
		};

		let picks: Vec<(Entity, HitData)> = point_query
			.iter()
			.sort::<&ObjectZLayer>()
			.filter_map(|(entity, point_position, z_position, pickable)| {
				if blocked {
					return None;
				}

				let distance = point_position.xy().distance(cursor_pos_world);
				let is_less_5 = distance < PICK_RANGE;
				blocked = is_less_5 && pickable.map(|p| p.should_block_lower) != Some(false);

				let depth = -cam_ortho.near - z_position.0 as f32;

				is_less_5.then_some((entity, HitData::new(cam_entity, depth, None, None)))
			})
			.collect();

		let order = camera.order as f32;
		output.send(PointerHits::new(*pointer, picks, order));
	}
}

fn update_svg(
	mut image_assets: ResMut<Assets<Image>>,
	mut query_path: Query<
		(
			Entity,
			&mut Transform,
			Option<&mut Path>,
			Option<&mut Handle<Image>>,
		),
		With<ObjectPath>,
	>,
	path_changed: Query<
		Entity,
		(
			Or<(
				Changed<PathClose>,
				Changed<Children>,
				Changed<RenderPathAs>,
				Changed<StrokeNet>,
				Changed<FillNet>,
			)>,
			With<ObjectPath>,
		),
	>,
	query_point_changes: Query<
		&Parent,
		(
			Or<(Changed<ObjectPosition>, Changed<PointType>)>,
			With<ObjectPoint>,
		),
	>,
	path_param: PathParam,
) {
	let mut par_change = query_point_changes
		.iter()
		.map(|parent| parent.get())
		.collect::<HashSet<_>>();
	path_changed.into_iter().for_each(|ent| {
		par_change.insert(ent);
	});
	let mut filted_par = query_path.iter_many_mut(par_change.iter());

	while let Some((ent_holder, mut transform, op_path, op_img)) = filted_par.fetch_next() {
		if let Some(mut path) = op_path {
			if let Some(pathed) = path_param.to_render_path(ent_holder) {
				*path = pathed;
			}
		} else if let Some(mut img_handle) = op_img {
			let Some((data, placement)) = path_param.to_image(ent_holder) else {
				warn!("Path Not Exist");
				return;
			};

			let img = image_assets.get_mut(img_handle.id()).unwrap();
			img.resize(Extent3d {
				width: placement.width,
				height: placement.height,
				..default()
			});

			img.data = data;
			transform.translation.x = placement.left as f32;
			transform.translation.y = placement.top as f32 * -1.0;
			img_handle.set_changed();
		}
	}
}

fn update_aabb_path(
	mut query_object: Query<(&mut Aabb, &mut Transform, &Path), (Changed<Path>, With<ObjectPath>)>,
) {
	query_object
		.iter_mut()
		.for_each(|(mut aabb, mut transform, path)| {
			transform.translation.x = 0.0;
			transform.translation.y = 0.0;
			let mut boxed_rect: Option<Rect> = None;

			for point in path.0.iter() {
				let a_pos = point.from().to_f32();
				let xy_pos = point.to().to_f32();
				let new_box = Rect::new(a_pos.x, a_pos.y, xy_pos.x, xy_pos.y);

				if let Some(boxy) = &mut boxed_rect {
					*boxy = boxy.union(new_box);
				} else {
					boxed_rect = Some(new_box);
				}
			}

			if let Some(boxy) = boxed_rect {
				let center = boxy.center();
				let half_size = boxy.half_size();
				aabb.center.x = center.x;
				aabb.center.y = center.y;
				aabb.half_extents.x = half_size.x;
				aabb.half_extents.y = half_size.y;
			}
		});
}

fn path_render_as(
	mut cmd: Commands,
	mut image_assets: ResMut<Assets<Image>>,
	query_path: Query<
		(Entity, &RenderPathAs, &StrokeNet, &FillNet),
		(Changed<RenderPathAs>, With<ObjectWorld>),
	>,
) {
	query_path
		.iter()
		.for_each(|(ent_path, render_as, stroke, fill)| match render_as {
			RenderPathAs::Image => {
				cmd.entity(ent_path)
					.remove::<PathAsSvgBundle>()
					.insert(PathAsImageBundle {
						sprite: Sprite {
							anchor: Anchor::TopLeft,
							..default()
						},
						texture: image_assets.rgba8_image(vec![0; 4], UVec2::ONE),
					});
			},
			RenderPathAs::Path => {
				cmd.entity(ent_path)
					.remove::<PathAsImageBundle>()
					.insert(PathAsSvgBundle::new(
						stroke.clone().into(),
						fill.clone().into(),
					));
			},
		});
}

#[derive(SystemParam)]
pub struct PathParam<'w, 's> {
	pub query_path: Query<
		'w,
		's,
		(
			&'static Children,
			&'static StrokeNet,
			&'static FillNet,
			&'static PathClose,
		),
		With<ObjectWorld>,
	>,
	pub query_point: Query<
		'w,
		's,
		(
			(&'static ObjectPosition, &'static PointType),
			Relations<PointToPoint>,
		),
		With<ObjectPoint>,
	>,
	pub root_point: Query<'w, 's, (Entity, &'static Parent), Root<PointToPoint>>,
}

impl<'w, 's> PathParam<'w, 's> {
	fn to_render_path(&self, path_ent: Entity) -> Option<Path> {
		let (first_point, _) = self
			.root_point
			.iter()
			.find(|(_, holder)| holder.get() == path_ent)?;
		let (.., close) = self.query_path.get(path_ent).ok()?;

		let mut path_builder = PathBuilder::new();

		let mut is_first = true;
		self.query_point
			.traverse::<PointToPoint>([first_point])
			.track_self()
			.for_each(|a, _, b, _| {
				let a_pos = a.0.xy();
				let xy_pos = b.0.xy();

				if is_first {
					is_first = false;
					path_builder.move_to(a_pos);
				}

				match *b.1 {
					PointType::LineTo => {
						path_builder.line_to(xy_pos);
					},
					PointType::QuadraticBezier { to } => {
						path_builder.quadratic_bezier_to(xy_pos, to);
					},
					PointType::CubricBezier { ctrl1, ctrl2, to } => {
						path_builder.cubic_bezier_to(ctrl1, ctrl2, to);
					},
					PointType::Arc {
						radii,
						sweep_angle,
						x_rotation,
					} => path_builder.arc(xy_pos, radii, sweep_angle, x_rotation),
				}
			});

		close.0.then(|| path_builder.close());

		Some(path_builder.build())
	}
	pub fn to_image(&self, path_ent: Entity) -> Option<(Vec<u8>, Placement)> {
		// use raqote::*;
		use zeno::*;
		let (first_point, _) = self
			.root_point
			.iter()
			.find(|(_, holder)| holder.get() == path_ent)?;
		let (children, stroke, fill, close) = self.query_path.get(path_ent).ok()?;
		let mut path_builder: Vec<Command> = Vec::with_capacity(children.len());
		let mut pb = raqote::PathBuilder::new();
		let mut is_first = true;

		self.query_point
			.traverse::<PointToPoint>([first_point])
			.track_self()
			.for_each(|a, _, b, _| {
				let a_pos = a.0.xy() * Vec2::new(1.0, -1.0);
				let first_arr = a_pos.to_array();
				let xy_pos = b.0.xy() * Vec2::new(1.0, -1.0);
				let to_arr = xy_pos.to_array();

				if is_first {
					is_first = false;
					path_builder.move_to(first_arr);
					pb.move_to(a_pos.x, a_pos.y);
				}

				match *b.1 {
					PointType::LineTo => {
						path_builder.line_to(to_arr);
						pb.line_to(xy_pos.x, xy_pos.y);
					},
					PointType::QuadraticBezier { to } => {
						path_builder.quad_to(to_arr, to.to_array());
						pb.quad_to(xy_pos.x, xy_pos.y, to.x, to.y);
					},
					PointType::CubricBezier { ctrl1, ctrl2, to } => {
						path_builder.curve_to(ctrl1.to_array(), ctrl2.to_array(), to.to_array());
						pb.cubic_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y);
					},
					PointType::Arc {
						radii,
						sweep_angle,
						x_rotation: _,
					} => {
						pb.arc(xy_pos.x, xy_pos.y, radii.x, 0.0, sweep_angle);
						// path_builder.arc_to(rx, ry, angle, size, sweep, to);
					},
				}
			});

		close.0.then(|| {
			path_builder.close();
			pb.close();
		});

		let StrokeOptions {
			start_cap,
			end_cap,
			line_join,
			line_width,
			miter_limit,
			..
		} = stroke.options;

		let placement = {
			let Placement {
				left,
				top,
				width,
				height,
			} = zeno::Mask::new(&path_builder)
				.style(
					Stroke::new(line_width)
						.join(line_join.to_zeno())
						.miter_limit(miter_limit)
						.caps(start_cap.to_zeno(), end_cap.to_zeno()),
				)
				.render()
				.1;
			let upped_miter = miter_limit.ceil() as i32 * 2;
			let upped_miter_double = upped_miter as u32 * 2;
			Placement {
				left: left - upped_miter,
				top: top - upped_miter,
				width: width + upped_miter_double,
				height: height + upped_miter_double,
			}
		};

		use raqote::{DrawOptions, DrawTarget, SolidSource, Source, StrokeStyle};
		let mut dt = DrawTarget::new(placement.width as i32, placement.height as i32);
		dt.set_transform(&raqote::Transform::translation(
			-placement.left as f32,
			-placement.top as f32,
		));
		let path_finish = pb.finish();

		let draw_op = DrawOptions::new();
		let [r, g, b, a] = fill.color.to_srgba().to_u8_array();
		dt.fill(
			&path_finish,
			&Source::Solid(SolidSource::from_unpremultiplied_argb(a, b, g, r)),
			&draw_op,
		);

		let [r, g, b, a] = stroke.color.to_srgba().to_u8_array();
		dt.stroke(
			&path_finish,
			&Source::Solid(SolidSource::from_unpremultiplied_argb(a, b, g, r)),
			&StrokeStyle {
				cap: start_cap.to_raqote(),
				width: line_width,
				join: line_join.to_raqote(),
				miter_limit,
				// dash_array: todo!(),
				// dash_offset: todo!(),
				..default()
			},
			&draw_op,
		);

		Some((dt.get_data_u8().to_vec(), placement))
	}
	pub fn to_svg(&self, path_ent: Entity) -> Option<SVG> {
		use svg::{
			node::element::{path::Data, Path},
			Document,
		};
		use zeno::*;

		let (first_point, _) = self
			.root_point
			.iter()
			.find(|(_, holder)| holder.get() == path_ent)?;

		let (children, stroke, fill, close) = self.query_path.get(path_ent).ok()?;
		let mut path_builder: Vec<Command> = Vec::with_capacity(children.len());
		let mut data = Data::new();
		let mut is_first = true;

		self.query_point
			.traverse::<PointToPoint>([first_point])
			.track_self()
			.for_each(|a, _, b, _| {
				let Vec2 { x, y } = a.0.xy() * Vec2::new(1.0, -1.0);
				let first_arr = (x, y);
				let Vec2 { x, y } = b.0.xy() * Vec2::new(1.0, -1.0);
				let to_arr = (x, y);

				if is_first {
					is_first = false;
					path_builder.move_to(first_arr);
					data = data.clone().move_to(first_arr);
				}

				match *b.1 {
					PointType::LineTo => {
						path_builder.line_to(to_arr);
						data = data.clone().line_to(to_arr);
					},
					PointType::QuadraticBezier { to: Vec2 { x, y } } => {
						path_builder.quad_to(to_arr, [x, y]);
						data = data.clone().quadratic_curve_to((x, y));
					},
					PointType::CubricBezier { ctrl1, ctrl2, to } => {
						path_builder.curve_to(ctrl1.to_array(), ctrl2.to_array(), to.to_array());
						// data = data.clone().cubic_curve_to(ctrl1.to_array(), ctrl2.to_array(), to.to_array());
					},
					PointType::Arc {
						radii: _,
						sweep_angle: _,
						x_rotation: _,
					} => {
						// data = data.arc_to(rx, ry, angle, size, sweep, to);
					},
				}
			});

		close.0.then(|| data = data.clone().close());

		let StrokeOptions {
			start_cap,
			end_cap,
			line_join,
			line_width,
			miter_limit,
			..
		} = stroke.options;
		let FillOptions { fill_rule, .. } = fill.options;
		let placement = {
			Mask::new(&path_builder)
				.style(
					Stroke::new(line_width)
						.join(line_join.to_zeno())
						.miter_limit(miter_limit)
						.cap(Cap::Round)
						.caps(start_cap.to_zeno(), end_cap.to_zeno()),
				)
				.render()
				.1
		};
		let fill_rule = match fill_rule {
			FillRule::EvenOdd => "evenodd",
			FillRule::NonZero => "nonzero",
		};
		let line_join = match line_join {
			LineJoin::Miter => "miter",
			LineJoin::MiterClip => "miter-clip",
			LineJoin::Round => "round",
			LineJoin::Bevel => "bevel",
		};
		let path = Path::new()
			.set("fill", fill.color.to_srgba().to_hex())
			.set("fill-rule", fill_rule)
			.set("stroke", stroke.color.to_srgba().to_hex())
			.set("stroke-width", line_width)
			.set("stroke-miterlimit", miter_limit)
			.set("stroke-linejoin", line_join)
			.set("d", data);
		let document = Document::new()
			.set(
				"viewBox",
				(
					placement.left - 5,
					placement.top - 5,
					placement.left + placement.width as i32 + 5,
					placement.top + placement.height as i32 + 5,
				),
			)
			.add(path);
		Some(document)
	}
}
