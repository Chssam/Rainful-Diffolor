use crate::trait_bevy::BuildCommonImage;

use super::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::render_resource::Extent3d;
use bevy::sprite::Anchor;
use bevy::{math::I64Vec2, prelude::*};
use image::*;
use imageops::{blur, overlay, replace, resize, FilterType};
use imageproc::drawing::{draw_filled_circle_mut, BresenhamLineIter};
use lightyear::prelude::ClientMessageEvent;
use lz4_flex::block::{compress_prepend_size, decompress_size_prepended, DecompressError};
use serde::{Deserialize, Serialize};

pub mod components;
pub mod events;
// pub mod gif_edition;

pub use components::*;
pub use events::*;
// pub use gif_edition::*;

pub(super) struct ImageProcessPlugin;
impl Plugin for ImageProcessPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<DataHold>()
			.register_type::<BrushRef>()
			.observe(pencel_draw)
			.add_systems(
				Update,
				(
					update_img_aabb,
					update_draw_ref,
					(resize_img_fr, apply_change_img).chain(),
				),
			)
			.add_systems(PostUpdate, process_image_changed);
	}
}

#[derive(Reflect, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataHold {
	Uncompress(Vec<u8>),
	Compressed(Vec<u8>),
}

impl DataHold {
	pub fn to_compress(data: &[u8]) -> Self {
		let compressed = compress_prepend_size(data);
		Self::Compressed(compressed)
	}
	pub fn uncompress(&self) -> Result<Vec<u8>, DecompressError> {
		match self {
			DataHold::Uncompress(v) => Ok(v.clone()),
			DataHold::Compressed(v) => decompress_size_prepended(v.as_slice()),
		}
	}
	pub fn len(&self) -> usize {
		match self {
			DataHold::Uncompress(v) => v.len(),
			DataHold::Compressed(v) => v.len(),
		}
	}
}

fn update_draw_ref(
	mut query_user: Query<
		(
			&mut DrawPiled,
			&BrushRef,
			&BrushScale,
			&PaintInk,
			&HardEdgeDraw,
			&BlurScale,
		),
		(
			With<UserId>,
			Or<(
				Changed<BrushRef>,
				Changed<BrushScale>,
				Changed<PaintInk>,
				Changed<HardEdgeDraw>,
				Changed<BlurScale>,
			)>,
		),
	>,
) {
	query_user.iter_mut().for_each(
		|(mut draw_pile, brush_ref, brush_scale, paint, hard_edge, blur_scale)| {
			let rgba = Rgba(paint.0.to_u8_array());
			let a_alpha = paint.0.alpha();
			let mut ref_draw = match brush_ref {
				BrushRef::CustomAlpha { brush, size } => {
					let Ok(valid_data) = brush.uncompress() else {
						return;
					};
					let mut ref_draw = RgbaImage::new(size.x, size.y);
					ref_draw.pixels_mut().enumerate().for_each(|(n, v)| {
						let alpha_bit = valid_data[n];
						*v = if alpha_bit != 0 {
							if hard_edge.0 {
								rgba
							} else {
								let calculate_alpha = alpha_bit as f32 / 255.0 * a_alpha;
								paint.0.with_alpha(calculate_alpha).to_u8_array().into()
							}
						} else {
							Rgba([0; 4])
						};
					});
					let brush_sized = brush_scale.get().clamp(1, 100) as u32;
					resize(
						&ref_draw,
						ref_draw.width() * brush_sized,
						ref_draw.height() * brush_sized,
						FilterType::Nearest,
					)
				},
				BrushRef::CustomColor { brush, size } => {
					let Ok(valid_data) = brush.uncompress() else {
						return;
					};

					let Some(new_ref_draw) = RgbaImage::from_vec(size.x, size.y, valid_data) else {
						return;
					};
					let brush_sized = brush_scale.get().clamp(1, 100) as u32;
					resize(
						&new_ref_draw,
						new_ref_draw.width() * brush_sized,
						new_ref_draw.height() * brush_sized,
						FilterType::Nearest,
					)
				},
				BrushRef::Circle => {
					let size = brush_scale.get();
					let img_size = 1 + size as u32 * 2;
					let mut new_ref_draw = RgbaImage::new(img_size, img_size);
					let center = (img_size / 2) as i32;
					draw_filled_circle_mut(&mut new_ref_draw, (center, center), size as i32, rgba);
					new_ref_draw
				},
			};
			if blur_scale.0 != 0.0 {
				let new_dimension = ref_draw.dimensions();
				let add_pix = 4 + (blur_scale.0 * 10.0) as u32;
				let real_add = add_pix + add_pix % 2;
				let mut new_size =
					RgbaImage::new(new_dimension.0 + real_add, new_dimension.1 + real_add);
				overlay(
					&mut new_size,
					&ref_draw,
					(real_add / 2) as i64,
					(real_add / 2) as i64,
				);
				ref_draw = blur(&new_size, blur_scale.0);
			}

			draw_pile.0 = ref_draw;
		},
	);
}

/// Drawing at Left side sometime shift to right 1 pixel
fn pencel_draw(
	trigger: Trigger<PenDraw>,
	query_user: Query<
		(
			&SelectedObject,
			&UserId,
			&DrawingSpacing,
			&DrawPiled,
			&DrawType,
		),
		With<UserId>,
	>,
	mut query_object: Query<
		(
			&mut ProcessImage,
			&ObjectPosition,
			&PixelLock,
			&AlphaLock,
			&ObjectAccess,
		),
		With<ObjectWorld>,
	>,
) {
	let CursorFromTo { from, to } = trigger.event().0;
	let (selected_obj, user_id, draw_spacing, draw_pile, draw_type) =
		query_user.get(trigger.entity()).unwrap();

	selected_obj.group.iter().for_each(|ent_obj| {
		let Ok((mut process_img, obj_pos, pix_lock, alpha_lock, access)) =
			query_object.get_mut(*ent_obj)
		else {
			return;
		};
		if pix_lock.contains(&user_id.0) || !access.targets(&user_id.0) {
			return;
		}

		let stay_alpha = process_img
			.iter()
			.copied()
			.skip(3)
			.step_by(4)
			.collect::<Vec<_>>();

		let uvec: UVec2 = draw_pile.dimensions().into();
		let I64Vec2 { x, y } = (uvec / 2).as_i64vec2();

		let limit: UVec2 = process_img.dimensions().into();
		let draw_size = uvec.as_vec2();
		let expand = (draw_size.x.max(draw_size.y) / 2.0).ceil() + 0.5;
		let rected = Rect::from_corners(Vec2::ZERO, limit.as_vec2()).inflate(expand);
		let further = rected.inflate(2.0);

		let obj_pos = obj_pos.0;
		let start = from
			.pixel_pos_central(obj_pos)
			.clamp(further.min, further.max);
		let end = to
			.pixel_pos_central(obj_pos)
			.clamp(further.min, further.max);

		let draw_type = match trigger.event().1 {
			DrawingWay::Color => draw_type,
			DrawingWay::Erase => &DrawType::Replace,
		};
		match draw_type {
			DrawType::Normal => {
				for (at_x, at_y) in BresenhamLineIter::new(start.into(), end.into())
					.step_by(draw_spacing.get().into())
				{
					let pos = IVec2::new(at_x, at_y).as_vec2();
					if !rected.contains(pos) {
						continue;
					}
					overlay(
						&mut process_img.0,
						&draw_pile.0,
						at_x as i64 - x,
						at_y as i64 - y,
					);
				}
			},
			DrawType::Replace => {
				let mut eraser = draw_pile.0.clone();
				eraser.fill(0);
				for (at_x, at_y) in BresenhamLineIter::new(start.into(), end.into())
					.step_by(draw_spacing.get().into())
				{
					let pos = IVec2::new(at_x, at_y).as_vec2();
					if !rected.contains(pos) {
						continue;
					}
					replace(
						&mut process_img.0,
						&eraser,
						at_x as i64 - x,
						at_y as i64 - y,
					);
				}
			},
			DrawType::Behind => {
				let mut new_rgba = RgbaImage::new(limit.x, limit.y);
				for (at_x, at_y) in BresenhamLineIter::new(start.into(), end.into())
					.step_by(draw_spacing.get().into())
				{
					let pos = IVec2::new(at_x, at_y).as_vec2();
					if !rected.contains(pos) {
						continue;
					}
					overlay(
						&mut new_rgba,
						&draw_pile.0,
						at_x as i64 - x,
						at_y as i64 - y,
					);
				}
				overlay(&mut new_rgba, &process_img.0, 0, 0);
				process_img.0 = new_rgba;
			},
		}

		if alpha_lock.contains(&user_id.0) {
			process_img.pixels_mut().enumerate().for_each(|(n, pix)| {
				pix.0[3] = stay_alpha[n];
			});
		}
	});
}

fn process_image_changed(
	mut image_assets: ResMut<Assets<Image>>,
	query_object: Query<(&Handle<Image>, &ProcessImage), Changed<ProcessImage>>,
) {
	query_object.iter().for_each(|(handle_img, process_img)| {
		let detail_img = image_assets.get_mut(handle_img).unwrap();
		let dimension = process_img.dimensions();
		if detail_img.size() != dimension.into() {
			let (width, height) = dimension;
			detail_img.resize(Extent3d {
				width,
				height,
				..default()
			});
		}
		detail_img.data = process_img.as_bytes().to_vec();
	});
}

fn apply_change_img(
	mut cmd: Commands,
	mut query_object: Query<(Entity, &mut Handle<Image>, &PreviousImage), With<ObjectWorld>>,
	query_user: Query<&SelectedObject, With<UserId>>,
	mut events: EventReader<ClientMessageEvent<ApplyChange>>,
) {
	events.read().for_each(|event| {
		let ApplyChange(is_apply, ent_user) = event.message;
		let selected_obj = query_user.get(ent_user).unwrap();
		let mut iter = query_object.iter_many_mut(selected_obj.group.iter());
		while let Some((ent_obj, mut handle_img, previos_handle)) = iter.fetch_next() {
			if !is_apply {
				*handle_img = previos_handle.img.clone();
			}
			cmd.entity(ent_obj).remove::<PreviousImage>();
		}
	});
}

fn resize_img_fr(
	mut cmd: Commands,
	mut image_assets: ResMut<Assets<Image>>,
	mut query_object: Query<
		(&mut Transform, &mut Handle<Image>, Option<&PreviousImage>),
		With<ObjectWorld>,
	>,
	query_user: Query<
		(&SelectedObject, &ScaleAction, &ResizeKind, &ScalePosition),
		(With<UserId>, Changed<ScaleAction>),
	>,
) {
	query_user
		.iter()
		.for_each(|(selected_obj, scale_action, kind, scale_pos)| {
			let Some(ScaleKind::Pixel(mut size)) = scale_action.0 else {
				return;
			};

			selected_obj.group.iter().for_each(|ent_img| {
				let Ok((mut transform, mut img_handle, previous_img)) =
					query_object.get_mut(*ent_img)
				else {
					return;
				};

				let Some(previous_img_data) = previous_img else {
					let Some(imged) = image_assets.get(img_handle.id()).cloned() else {
						return;
					};
					cmd.entity(*ent_img).insert(PreviousImage::new(
						transform.translation.truncate(),
						img_handle.clone(),
					));
					let new_handle = image_assets.add(imged);
					*img_handle = new_handle;
					return;
				};
				let Some(prev_imged) = image_assets.get(previous_img_data.img.id()).cloned() else {
					return;
				};

				let Vec2 {
					x: prev_x,
					y: prev_y,
				} = previous_img_data.pos;

				let size_img = prev_imged.size();
				let img_float = size_img.as_vec2();
				let img_float_pos = img_float - 1.0;
				let ori_img =
					RgbaImage::from_vec(size_img.x, size_img.y, prev_imged.data.clone()).unwrap();

				match scale_pos {
					ScalePosition::Top => {
						transform.translation.y = (prev_y - size.y).max(prev_y - img_float_pos.y);
						size *= Vec2::new(0.0, -1.0);
					},
					ScalePosition::Bottom => size.x = 0.0,
					ScalePosition::Left => {
						transform.translation.x = prev_x - size.x.max(-img_float_pos.x);
						size.y = 0.0;
					},
					ScalePosition::Right => size *= Vec2::new(-1.0, 0.0),
					ScalePosition::TopLeft => {
						transform.translation.x = prev_x - size.x.max(-img_float_pos.x);
						transform.translation.y = (prev_y - size.y).max(prev_y - img_float_pos.y);
						size *= Vec2::new(1.0, -1.0);
					},
					ScalePosition::TopRight => {
						transform.translation.y = (prev_y - size.y).max(prev_y - img_float_pos.y);
						size *= -1.0;
					},
					ScalePosition::BottomLeft => {
						transform.translation.x = prev_x - size.x.max(-img_float_pos.x);
					},
					ScalePosition::BottomRight => size *= Vec2::new(-1.0, 1.0),
					ScalePosition::Middle => return,
				}

				let calculation = (img_float + size).clamp(Vec2::ONE, Vec2::splat(2000.0));

				let UVec2 { x, y } = calculation.as_uvec2();

				let data = match kind {
					ResizeKind::Scale => {
						let resized_img = resize(&ori_img, x, y, FilterType::Nearest);
						resized_img.into_vec()
					},
					ResizeKind::Resize => {
						let mut resized_img = RgbaImage::new(x, y);
						overlay(&mut resized_img, &ori_img, 0, 0);
						resized_img.into_vec()
					},
				};

				image_assets.remove(img_handle.id());
				*img_handle = image_assets.rgba8_image(data, calculation.as_uvec2());
			});
		});
}

fn update_img_aabb(
	image_assets: Res<Assets<Image>>,
	mut query_object: Query<
		(&Handle<Image>, &mut Aabb),
		(Changed<Handle<Image>>, With<ObjectWorld>),
	>,
) {
	query_object.iter_mut().for_each(|(handle_img, mut aabb)| {
		let img = image_assets.get(handle_img).unwrap();
		let Vec2 { x, y } = img.size_f32() / 2.0;
		aabb.half_extents.x = x;
		aabb.half_extents.y = y;
		aabb.center.x = x;
		aabb.center.y = -y;
	});
}

// #[derive(PartialEq)]
// enum FlipWay {
//     Horizontal,
//     Vertical,
// }

// fn img_fliper(
//     image_assets: &mut Assets<Image>,
//     single_image: &Query<(&Handle<Image>, &SharingName), (With<RdioImage>, With<MainSelectObject>)>,
//     flip_way: FlipWay,
// ) {
//     let Ok((handle_img, _)) = single_image.get_single() else {
//         return;
//     };
//     let detail_img = image_assets.get_mut(handle_img).unwrap();
//     let size_img = detail_img.size();
//     let rgba_img = RgbaImage::from_vec(size_img.x, size_img.y, detail_img.data.clone()).unwrap();
//     let rotated_img = match flip_way {
//         FlipWay::Horizontal => flip_horizontal(&rgba_img),
//         FlipWay::Vertical => flip_vertical(&rgba_img),
//     };
//     detail_img.data = rotated_img.to_vec();
// }

// pub(super) fn crop_2d(
//     mut query: Query<&mut Selection, With<UserId>>,
//     mut image_assets: ResMut<Assets<Image>>,
//     mut objects: Query<(&mut Transform, &Handle<Image>), (With<MainActiveObj>, With<RdioImage>)>,
//     actions_key: Res<ActionState<SettingsAction>>,
//     mut screen2d: GlobalScreen2D,
// ) {
//     let Ok((mut transform_obj, handle_img_obj)) = objects.get_single_mut() else {
//         return;
//     };
//     let Some(cur_world) = screen2d.cursor_world() else {
//         return;
//     };
//     let real_world_ray = cur_world.round();
//     let pos_pix = transform_obj.translation.truncate();
//     let imgy_size = detail_img.size();
//     let relative = (real_world_ray - pos_pix) * Vec2::new(1.0, -1.0);
//     let fix_ray = relative.clamp(Vec2::ZERO, imgy_size.as_vec2());

//     actions_key
//         .just_pressed(&SettingsAction::Primary)
//         .then(|| cropper.0 = Some(fix_ray));
//     actions_key
//         .pressed(&SettingsAction::Primary)
//         .then(|| cropper.1 = Some(fix_ray));
//     receive_click
//         .just_pressed(&SettingsAction::Escape)
//         .then(|| {
//             cropper.0 = None;
//             cropper.1 = None;
//         });

//     let (Some(pin_begin), Some(pin_end)) = (cropper.0, cropper.1) else {
//         return;
//     };

//     let rectness = Rect::from_corners(pin_begin, pin_end);
//     let i_real_pick = rectness.min.as_uvec2();
//     let well_pos =
//         transform_obj.translation.truncate() + Vec2::new(rectness.min.x, -rectness.min.y);

//     if receive_click.just_pressed(&SettingsAction::Enter) && !rectness.is_empty() {
//         let sizey = rectness.size().as_uvec2();
//         cropper.0 = None;
//         let mut imgy =
//             RgbaImage::from_vec(imgy_size.x, imgy_size.y, detail_img.data.clone()).unwrap();
//         let croped_img =
//             crop::<RgbaImage>(&mut imgy, i_real_pick.x, i_real_pick.y, sizey.x, sizey.y).to_image();
//         transform_obj.translation = well_pos.extend(transform_obj.translation.z);
//         detail_img.texture_descriptor.size = Extent3d {
//             width: croped_img.width(),
//             height: croped_img.height(),
//             depth_or_array_layers: 1,
//         };
//         detail_img.data = croped_img.to_vec();
//     }
// }
