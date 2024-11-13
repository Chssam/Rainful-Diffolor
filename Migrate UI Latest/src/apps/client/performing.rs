use arboard::{Clipboard, ImageData};
use backend::PointerHits;
use bevy::{
	ecs::{system::SystemState, world::CommandQueue},
	math::I64Vec2,
	prelude::*,
	render::primitives::Aabb,
	sprite::Anchor,
	tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, IoTaskPool},
	window::PrimaryWindow,
};
use bevy_cosmic_edit::{BufferExtras, CosmicBuffer};
use bevy_mod_picking::prelude::*;
use client::ConnectionManager;
use image::*;
use imageops::overlay;
use leafwing_input_manager::{
	action_diff::{ActionDiff, ActionDiffEvent, SummarizedActionState},
	prelude::*,
};
use lightyear::prelude::*;
use rainful_diffolor::{source_to_docs, RDIO_IN_SAVE_DISK};
use std::{collections::HashSet, thread, time::Duration};

use crate::{apps::shared::prelude::*, camera_control::lib::*, trait_bevy::BuildCommonImage};

use super::*;

pub(super) fn setup_client_resources(world: &mut World) {
	let id = world.resource::<ClientConnection>().id();
	let name = world
		.query_filtered::<&CosmicBuffer, With<NameInput>>()
		.single(world)
		.get_text();
	let mut image_assets = world.resource_mut::<Assets<Image>>();

	let paint = PaintInk::default();
	let rgba_size = UVec2::new(256, 1);
	let sl_size = UVec2::new(101, 1);

	let red = image_assets.rgba8_image(paint.red(), rgba_size);
	let blue = image_assets.rgba8_image(paint.blue(), rgba_size);
	let green = image_assets.rgba8_image(paint.green(), rgba_size);
	let alpha = image_assets.rgba8_image(paint.alpha(), rgba_size);
	let hue = image_assets.rgba8_image(paint.hue(), UVec2::new(361, 1));
	let saturation = image_assets.rgba8_image(paint.saturation(), sl_size);
	let lightness = image_assets.rgba8_image(paint.lightness(), sl_size);
	let sat_light = image_assets.rgba8_image(paint.sat_light(), UVec2::new(101, 101));
	let pre_brush_display = image_assets.rgba8_image(vec![255; 4], UVec2::ONE);
	let create_rgbahsl = RGBAhsl {
		red,
		blue,
		green,
		alpha,
		hue,
		saturation,
		lightness,
		sat_light,
	};
	world.spawn((
		MainUserBundle::new(),
		ClientUserBundle::new(id, name),
		StateScoped(RdioClientState::Online),
		create_rgbahsl,
	));
	world.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::Z * (CAMERA_VIEW_RANGE - 10.0)),
			sprite: Sprite {
				anchor: Anchor::TopLeft,
				color: Color::default().with_alpha(0.8),
				..default()
			},
			texture: pre_brush_display,
			visibility: Visibility::Hidden,
			..default()
		},
		DisplayBrush,
		Pickable::IGNORE,
	));

	let all_client_observe = [
		world.observe(handle_connection).id(),
		world.observe(unpickable_predicted).id(),
		world.observe(insert_as_objects).id(),
		// world.observe(set_selected_change).id(),
	];
	all_client_observe.into_iter().for_each(|ent_obs| {
		world
			.commands()
			.entity(ent_obs)
			.insert(StateScoped(RdioClientState::Online));
	});
}

fn unpickable_predicted(
	trigger: Trigger<OnAdd, Predicted>,
	mut cmd: Commands,
	query_object: Query<Entity, With<Predicted>>,
) {
	let ent_predict = trigger.entity();
	if !query_object.contains(ent_predict) {
		return;
	}
	cmd.entity(ent_predict).insert(Pickable::IGNORE);
}

fn handle_connection(
	trigger: Trigger<ConnectEvent>,
	mut query_user: Query<&mut UserId, With<MainUser>>,
) {
	let event = trigger.event();
	let client_id = event.client_id();
	let mut user_id = query_user.single_mut();
	user_id.0 = client_id;
}

pub(super) fn update_cursor_position(
	mut screen2d: GlobalScreen2D,
	mut query_user: Query<(&mut CursorPos, &ActionState<SettingsAction>), With<MainUser>>,
) {
	let Some(cur_world) = screen2d.cursor_world() else {
		return;
	};
	let Ok((mut cur_pos, action)) = query_user.get_single_mut() else {
		return;
	};
	let fixed_pos = if action.pressed(&SettingsAction::RoundPos) {
		cur_world.round()
	} else {
		cur_world
	};
	cur_pos.set_if_neq(CursorPos(fixed_pos));
}

pub(super) fn change_editor_tools(
	mut editor_tool_state: ResMut<NextState<EditorTools>>,
	query_user: Query<&ActionState<EditorTools>, With<MainUser>>,
) {
	let Ok(action_tools) = query_user.get_single() else {
		return;
	};
	if let Some(action) = action_tools.get_just_pressed().first() {
		editor_tool_state.set(*action);
	}
}

// fn img_rotate(
//     mut image_assets: ResMut<Assets<Image>>,
//     query_object: Query<&Handle<Image>, With<ObjectWorld>>,
// ) {
//     let Ok((handle_img, _)) = query_object.get_single() else {
//         return;
//     };
//     let detail_img = image_assets.get_mut(handle_img).unwrap();
//     let size_img = detail_img.size();
//     let rgba_img = RgbaImage::from_vec(size_img.x, size_img.y, detail_img.data.clone()).unwrap();
//     let rotated_img = rotate90(&rgba_img);
//     detail_img.resize(Extent3d {
//         width: rotated_img.width(),
//         height: rotated_img.height(),
//         ..default()
//     });
//     detail_img.data = rotated_img.to_vec();
// }

/// From Leafwing Code (Without [dbg!])
pub(super) fn generate_action_diffs<A: Actionlike>(
	global_action_state: Option<Res<ActionState<A>>>,
	action_state_query: Query<(Entity, &ActionState<A>)>,
	mut previous_action_state: Local<SummarizedActionState<A>>,
	mut action_diff_events: EventWriter<ActionDiffEvent<A>>,
) {
	let current_action_state =
		SummarizedActionState::summarize(global_action_state, action_state_query);
	current_action_state.send_diffs(&previous_action_state, &mut action_diff_events);
	*previous_action_state = current_action_state;
}

pub fn export_single_image_png(
	query_object: Query<(
		Entity,
		Option<&ProcessImage>,
		Has<ObjectPath>,
		&SharingName,
		Option<&SaveLocation>,
	)>,
	query_point: Query<&Parent, (With<ObjectPoint>, With<ObjectWorld>)>,
	query_user: Query<(&ActionState<ToolsStandAlone>, &SelectedObject), With<MainUser>>,
	path_param: PathParam,
) {
	let Some(docs_dir) = source_to_docs() else {
		return;
	};
	let Ok((action, selected_obj)) = query_user.get_single() else {
		return;
	};
	if !action.just_pressed(&ToolsStandAlone::ExportImage) {
		return;
	}
	let point_path = query_point
		.iter_many(selected_obj.group.iter())
		.map(|parent| parent.get())
		.collect::<HashSet<Entity>>();

	let rdio_img_dir = docs_dir.join(RDIO_IN_SAVE_DISK).join("Image");

	for (ent_obj, op_process_img, is_path, name, op_location) in
		query_object.iter_many(selected_obj.group.iter().chain(point_path.iter()))
	{
		let mut image_file = name.to_string();
		if !image_file.ends_with(".png") {
			image_file.push_str(".png");
		}
		let path = op_location
			.map(|path| path.0.clone())
			.unwrap_or(rdio_img_dir.clone())
			.join(image_file);

		let img_ready = if let Some(img) = op_process_img {
			img.0.clone()
		} else if is_path {
			let Some((data, placement)) = path_param.to_image(ent_obj) else {
				warn!("Invalid Path");
				return;
			};
			RgbaImage::from_vec(placement.width, placement.height, data).unwrap()
		} else {
			unreachable!();
		};

		IoTaskPool::get()
			.spawn(async move {
				img_ready
					.save_with_format(path, ImageFormat::Png)
					.unwrap_or_else(|e| error!("Failed to save image: {:?}", e));
			})
			.detach();
	}
}

pub fn copy_or_canvas(
	path_param: PathParam,
	query_object: Query<(
		Entity,
		Option<(&ProcessImage, &ObjectPosition)>,
		Has<ObjectPath>,
		&ObjectZLayer,
	)>,
	query_point: Query<&Parent, (With<ObjectPoint>, With<ObjectWorld>)>,
	query_user: Query<(&ActionState<ToolsStandAlone>, &SelectedObject), With<MainUser>>,
) {
	let Some(docs_dir) = source_to_docs() else {
		return;
	};
	let Ok((action, selected_obj)) = query_user.get_single() else {
		return;
	};

	let grouped_img = || -> Option<RgbaImage> {
		let point_path = query_point
			.iter_many(selected_obj.group.iter())
			.map(|parent| parent.get())
			.collect::<HashSet<Entity>>();

		let mut recty: Option<IRect> = None;
		let mut q = query_object
			.iter_many(selected_obj.group.iter().chain(point_path.iter()))
			.filter_map(|(ent_obj, op_process_img, is_path, z_object)| {
				if let Some((img, obj_pos)) = op_process_img {
					let pos = obj_pos.as_ivec2() * IVec2::new(1, -1);
					let size = UVec2::new(img.width(), img.height());
					let new_rect = IRect::from_corners(pos, pos + size.as_ivec2());
					if let Some(rected) = &mut recty {
						*rected = rected.union(new_rect);
					} else {
						recty = Some(new_rect);
					}

					Some((img.0.clone(), pos.as_i64vec2(), z_object.0))
				} else if is_path {
					let Some((data, placement)) = path_param.to_image(ent_obj) else {
						warn!("Invalid Path");
						return None;
					};
					let pos = IVec2::new(placement.left, placement.top);
					let size = UVec2::new(placement.width, placement.height);
					let new_rect = IRect::from_corners(pos, pos + size.as_ivec2());
					if let Some(rected) = &mut recty {
						*rected = rected.union(new_rect);
					} else {
						recty = Some(new_rect);
					}
					Some((
						RgbaImage::from_vec(size.x, size.y, data).unwrap(),
						pos.as_i64vec2(),
						z_object.0,
					))
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		q.sort_by_key(|a| a.2);

		if let Some(rected) = recty {
			let size = rected.size().as_uvec2();
			let mut imged = RgbaImage::new(size.x, size.y);
			q.into_iter().for_each(|(img, pos, _)| {
				let I64Vec2 { x, y } = pos - rected.min.as_i64vec2();
				overlay(&mut imged, &img, x, y);
			});
			return Some(imged);
		}
		None
	};

	if action.just_pressed(&ToolsStandAlone::ExportCanvas) {
		let Some(imged) = grouped_img() else {
			return;
		};
		let path = docs_dir
			.join(RDIO_IN_SAVE_DISK)
			.join("Image")
			.join("Canvas.png");
		IoTaskPool::get()
			.spawn(async move {
				imged
					.save_with_format(path, ImageFormat::Png)
					.unwrap_or_else(|e| error!("Failed to save image: {:?}", e));
			})
			.detach();
	}
	if action.just_pressed(&ToolsStandAlone::Copy) {
		let Some(imged) = grouped_img() else {
			return;
		};
		let mut clip_board = Clipboard::new().unwrap();
		let to_clip_img = ImageData {
			width: imged.width() as usize,
			height: imged.height() as usize,
			bytes: imged.to_vec().into(),
		};
		clip_board.set_image(to_clip_img).unwrap();
	}
}

pub fn export_svg(
	query_object: Query<(
		Entity,
		Option<(&ProcessImage, &ObjectPosition)>,
		Has<ObjectPath>,
		&ObjectZLayer,
	)>,
	query_point: Query<&Parent, (With<ObjectPoint>, With<ObjectWorld>)>,
	query_user: Query<(&ActionState<ToolsStandAlone>, &SelectedObject), With<MainUser>>,
	path_param: PathParam,
) {
	let Some(docs_dir) = source_to_docs() else {
		return;
	};
	let Ok((action, selected_obj)) = query_user.get_single() else {
		return;
	};

	// let grouped_img = || -> Option<RgbaImage> {
	// 	let point_path = query_point
	// 		.iter_many(selected_obj.group.iter())
	// 		.map(|parent| parent.get())
	// 		.collect::<HashSet<Entity>>();

	// 	let mut recty: Option<IRect> = None;
	// 	let mut q = query_object
	// 		.iter_many(selected_obj.group.iter().chain(point_path.iter()))
	// 		.filter_map(|(ent_obj, op_process_img, is_path, z_object)| {
	// 			if let Some((img, obj_pos)) = op_process_img {
	// 				let pos = obj_pos.as_ivec2() * IVec2::new(1, -1);
	// 				let size = UVec2::new(img.width(), img.height());
	// 				let new_rect = IRect::from_corners(pos, pos + size.as_ivec2());
	// 				if let Some(rected) = &mut recty {
	// 					*rected = rected.union(new_rect);
	// 				} else {
	// 					recty = Some(new_rect);
	// 				}

	// 				Some((img.0.clone(), pos.as_i64vec2(), z_object.0))
	// 			} else if is_path {
	// 				let Some((data, placement)) = path_param.fill_stroke(ent_obj) else {
	// 					warn!("Invalid Path");
	// 					return None;
	// 				};
	// 				let pos = IVec2::new(placement.left, placement.top);
	// 				let size = UVec2::new(placement.width, placement.height);
	// 				let new_rect = IRect::from_corners(pos, pos + size.as_ivec2());
	// 				if let Some(rected) = &mut recty {
	// 					*rected = rected.union(new_rect);
	// 				} else {
	// 					recty = Some(new_rect);
	// 				}
	// 				Some((
	// 					RgbaImage::from_vec(size.x, size.y, data).unwrap(),
	// 					pos.as_i64vec2(),
	// 					z_object.0,
	// 				))
	// 			} else {
	// 				None
	// 			}
	// 		})
	// 		.collect::<Vec<_>>();
	// 	q.sort_by_key(|a| a.2);

	// 	if let Some(rected) = recty {
	// 		let size = rected.size().as_uvec2();
	// 		let mut imged = RgbaImage::new(size.x, size.y);
	// 		q.into_iter().for_each(|(img, pos, _)| {
	// 			let I64Vec2 { x, y } = pos - rected.min.as_i64vec2();
	// 			overlay(&mut imged, &img, x, y);
	// 		});
	// 		return Some(imged);
	// 	}
	// 	None
	// };

	if action.just_pressed(&ToolsStandAlone::ExportSvgRelative) {
		let point_path = query_point
			.iter_many(selected_obj.group.iter())
			.map(|parent| parent.get())
			.collect::<HashSet<Entity>>();

		query_object
			.iter_many(selected_obj.group.iter().chain(point_path.iter()))
			.for_each(|(ent_obj, op_process_img, is_path, z_object)| {
				if is_path {
					let Some(document) = path_param.to_svg(ent_obj) else {
						return;
					};
					let path = docs_dir
						.join(RDIO_IN_SAVE_DISK)
						.join("SVG")
						.join("Canvas.svg");
					IoTaskPool::get()
						.spawn(async move {
							svg::save(path, &document)
								.unwrap_or_else(|e| error!("Failed to save SVG: {:?}", e));
						})
						.detach();
				}
			});
	}
	if action.just_pressed(&ToolsStandAlone::ExportSvgAbsolute) {}
}

pub(super) fn detect_file_drop(
	mut cmd: Commands,
	mut dnd_evr: EventReader<FileDragAndDrop>,
	windows: Query<Entity, With<PrimaryWindow>>,
) {
	let Ok(win_ent) = windows.get_single() else {
		return;
	};
	let thread_pool = AsyncComputeTaskPool::get();
	dnd_evr.read().for_each(|ev| {
		let FileDragAndDrop::DroppedFile { path_buf, window } = ev else {
			return;
		};
		if window != &win_ent {
			return;
		};

		let ent_file = cmd.spawn_empty().id();

		let file_path = path_buf.clone();
		let task = thread_pool.spawn(async move {
			thread::sleep(Duration::from_secs_f32(0.16));
			let mut cmd_queue = CommandQueue::default();

			cmd_queue.push(move |world: &mut World| {
				let name = file_path
					.file_name()
					.unwrap()
					.to_str()
					.unwrap_or("Image")
					.to_string();
				let Ok(Ok(dyn_img)) =
					ImageReader::open(file_path).map(|readed_file| readed_file.decode())
				else {
					warn!("Not Image Format");
					world.entity_mut(ent_file).despawn_recursive();
					return;
				};

				let mut client = {
					let mut system_state = SystemState::<ResMut<ConnectionManager>>::new(world);
					let resource = system_state.get_mut(world);
					resource
				};

				let size = dyn_img.dimensions().into();
				let mut img_net = ImageNetwork::new(name, dyn_img.as_bytes(), size);

				let compress_len = img_net.data().len();
				if compress_len > 290000 {
					warn!("Image Data unable to send upto 290000 bit: {compress_len}");
					world.entity_mut(ent_file).despawn_recursive();
					return;
				}

				client
					.send_message::<MainChannel, ImageNetwork>(&mut img_net)
					.unwrap_or_else(|e| error!("Fail to send message: {:?}", e));
				world.entity_mut(ent_file).despawn_recursive();

				// match file_format {
				// 	ImageFormat::Png
				// 	| ImageFormat::Pnm
				// 	| ImageFormat::Tiff
				// 	| ImageFormat::Tga
				// 	| ImageFormat::Dds
				// 	| ImageFormat::Bmp
				// 	| ImageFormat::Ico
				// 	| ImageFormat::Hdr
				// 	| ImageFormat::OpenExr
				// 	| ImageFormat::Farbfeld
				// 	| ImageFormat::Avif
				// 	| ImageFormat::Qoi
				// 	| ImageFormat::Jpeg => image_file_drop(world, ent_file, asset_path, real_world_ray),
				// 	ImageFormat::Gif | ImageFormat::WebP => {
				// 		animate_file_drop(world, ent_file, asset_path, real_world_ray)
				// 	},
				// 	_ => {
				// 		info!("Invalid / Not supported file");
				// 		world.entity_mut(ent_file).despawn_recursive();
				// 	},
				// }
			});
			cmd_queue
		});

		cmd.entity(ent_file).insert(FileReaded(task));
	});
}

pub(super) fn detect_path_directory(
	mut cmd: Commands,
	mut dnd_evr: EventReader<FileDragAndDrop>,
	query_user: Query<&SelectedObject, With<MainUser>>,
	query_object: Query<Entity, (Or<(With<ObjectImage>, With<ObjectPath>)>, With<ObjectWorld>)>,
	windows: Query<Entity, With<PrimaryWindow>>,
) {
	let (Ok(win_ent), Ok(selected_obj)) = (windows.get_single(), query_user.get_single()) else {
		return;
	};

	let Some(ev) = dnd_evr.read().next() else {
		return;
	};
	let FileDragAndDrop::DroppedFile { path_buf, window } = ev else {
		return;
	};
	if window != &win_ent || !path_buf.is_dir() {
		return;
	};
	for ent_obj in query_object.iter_many(selected_obj.group.iter()) {
		cmd.entity(ent_obj).insert(SaveLocation(path_buf.clone()));
	}
	cmd.insert_resource(NextState::Pending(DropPathMode::AsObject));
}

// fn animate_file_drop(
// 	world: &mut World,
// 	ent_file: Entity,
// 	asset_path: AssetPath,
// 	real_world_ray: Vec2,
// ) {
// 	let (assets_server, mut layer_handler) = {
// 		let mut system_state = SystemState::<(Res<AssetServer>, ResMut<ZLayer>)>::new(world);
// 		let resource = system_state.get_mut(world);
// 		resource
// 	};

// 	let pathed_asset = asset_path.clone();
// 	let name = pathed_asset
// 		.path()
// 		.file_name()
// 		.and_then(|name| name.to_str())
// 		.unwrap_or("Animated Image");

// 	let a_handle_image = assets_server.load(asset_path);

// 	let total_obj = layer_handler.len();
// 	let center_img = real_world_ray
// 		.floor()
// 		.extend(BEGIN_OBJ_Z_INDEX + total_obj as f32);
// 	layer_handler.push(ent_file);
// 	world
// 		.entity_mut(ent_file)
// 		.insert((RdioAnimatedImageBundle::new(
// 			name,
// 			a_handle_image,
// 			center_img,
// 		),))
// 		.remove::<FileReaded>();
// }

pub(super) fn load_when_ready(mut cmd: Commands, mut file_task: Query<(Entity, &mut FileReaded)>) {
	file_task.iter_mut().for_each(|(ent_task, mut task)| {
		if let Some(mut cmd_queue) = block_on(future::poll_once(&mut task.0)) {
			cmd.append(&mut cmd_queue);
			cmd.entity(ent_task).despawn();
		}
	});
}

pub(super) fn pick_object(
	mut on_click: EventReader<Pointer<Down>>,
	mut query_user: Query<
		(
			&mut BeginSelectPoint,
			&mut SelectedObject,
			&ActionState<SettingsAction>,
			&CursorPos,
		),
		With<MainUser>,
	>,
	image_assets: Res<Assets<Image>>,
	query_object: Query<
		(
			Entity,
			Option<&ViewVisibility>,
			&ObjectPosition,
			Option<&Handle<Image>>,
			Option<&Aabb>,
			Has<ObjectPoint>,
		),
		With<ObjectWorld>,
	>,
) {
	let Ok((mut pin_point, mut selected_obj, actions_key, cur_pos)) = query_user.get_single_mut()
	else {
		return;
	};
	let mut sorted_depth = on_click.read().collect::<Vec<_>>();
	sorted_depth.sort_by_key(|a| a.hit.depth as i64);

	for pointer in sorted_depth {
		if pointer.button != PointerButton::Primary {
			continue;
		}
		let ent_obj = pointer.target();
		if let Ok((_, op_view, obj_pos, op_handle_img, _, is_point)) = query_object.get(ent_obj) {
			if op_view.is_some_and(|view| !view.get()) {
				continue;
			}
			if !actions_key.pressed(&SettingsAction::Alt) && !is_point {
				if let Some(handle_img) = op_handle_img {
					let img = image_assets.get(handle_img.id()).unwrap();
					let pix_pos = cur_pos.pixel_to_img(obj_pos.0).floor().as_uvec2();

					let pix_pos = 4 * (1 + pix_pos.x + pix_pos.y * img.size().x) - 1;

					if img.data.get(pix_pos as usize).is_some_and(|v| v < &102) {
						continue;
					}
				}
			}
		} else {
			continue;
		};

		if actions_key.pressed(&SettingsAction::Control) {
			selected_obj.deselect_single(ent_obj);
			return;
		}

		if actions_key.pressed(&SettingsAction::Shift) {
			selected_obj.add_select(ent_obj);
		} else {
			selected_obj.select_single(ent_obj);
		}
		return;
	}

	if actions_key.just_pressed(&SettingsAction::Primary) {
		if !actions_key.pressed(&SettingsAction::Shift) {
			selected_obj.deselect_all();
		}
		pin_point.0 = Some(cur_pos.xy());
	}

	if actions_key.just_released(&SettingsAction::Primary) {
		let Some(pointer) = pin_point.0 else {
			return;
		};
		pin_point.0 = None;
		let recter = Rect::from_corners(pointer, cur_pos.xy());
		query_object
			.iter()
			.for_each(|(ent_obj, op_view, obj_pos, _, op_aabb, is_point)| {
				if is_point {
					if recter.contains(obj_pos.0) {
						selected_obj.add_select(ent_obj);
					}
					return;
				}
				let position = obj_pos.0 + op_aabb.unwrap().center.xy();
				let recty =
					Rect::from_center_half_size(position, op_aabb.unwrap().half_extents.truncate());
				if op_view.is_some_and(|view| !view.get())
					|| !recter.contains(recty.min)
					|| !recter.contains(recty.max)
				{
					return;
				}
				selected_obj.add_select(ent_obj);
			});
	}
}

pub(super) fn color_pick(
	mut read_pointer_hit: EventReader<PointerHits>,
	mut query_user: Query<
		(&mut PaintInk, &CursorPos, &ActionState<SettingsAction>),
		With<MainUser>,
	>,
	query_object: Query<
		(
			&GlobalTransform,
			&Handle<Image>,
			&ViewVisibility,
			&PickingInteraction,
		),
		With<ObjectWorld>,
	>,
	image_assets: Res<Assets<Image>>,
) {
	let Ok((mut paint, cur_pos, action)) = query_user.get_single_mut() else {
		return;
	};
	if !action.pressed(&SettingsAction::Primary) {
		return;
	}
	let Some(pointer) = read_pointer_hit.read().next() else {
		return;
	};
	let mut rgba = Rgba([0_u8; 4]);
	let mut is_changed = false;
	pointer.picks.iter().rev().for_each(|(ent_hit, _hit_data)| {
		let Ok((obj_pos, handle_img, visibility, pick_inter)) = query_object.get(*ent_hit) else {
			return;
		};
		if !visibility.get() || pick_inter != &PickingInteraction::Pressed {
			return;
		}
		let pos_obj = obj_pos.translation().truncate();
		let pix_pos = cur_pos.pixel_to_img(pos_obj).floor().as_uvec2();
		let obj_img = image_assets.get(handle_img.id()).unwrap();
		let skip_to = pix_pos.x + pix_pos.y * obj_img.size().x;
		let mut chunked = obj_img.data.chunks(4);
		let [r, g, b, a] = chunked.nth(skip_to as usize).unwrap() else {
			return;
		};
		rgba.blend(&Rgba([*r, *g, *b, *a]));
		is_changed = true;
	});

	if is_changed {
		let picked_color = Srgba::from_u8_array(rgba.0).with_alpha(paint.0.alpha());
		let new_paint = PaintInk(picked_color, paint.1);
		paint.set_if_neq(new_paint);
	}
}

pub(super) fn main_move_object(
	mut prev_pos: Local<MovedPoint>,
	mut query_user: Query<(&CursorPos, &ActionState<SettingsAction>), With<MainUser>>,
	mut client: ResMut<ConnectionManager>,
	unfocus_on_ui: Res<IsUnFocusOnUI>,
) {
	let Ok((cur_pos, actions_key)) = query_user.get_single_mut() else {
		return;
	};

	let cur_posed = cur_pos.xy();
	if actions_key.pressed(&SettingsAction::Move) && unfocus_on_ui.get() {
		let world_pos = -(prev_pos.world - cur_posed);
		let pixel_pos = -(prev_pos.pixel - cur_posed.floor().as_ivec2());
		if world_pos == Vec2::splat(0.0) && pixel_pos == IVec2::splat(0) {
			return;
		}
		client
			.send_message::<MainChannel, MovedPoint>(&mut MovedPoint {
				world: world_pos,
				pixel: pixel_pos,
			})
			.unwrap_or_else(|e: ClientError| {
				error!("Fail to send message: {:?}", e);
			});
	}
	prev_pos.world = cur_posed;
	prev_pos.pixel = cur_posed.floor().as_ivec2();
}

pub(super) fn pencel_line(
	mut gizmos: Gizmos,
	mut query_user: Query<
		(
			&mut LastDrawPos,
			&mut PreviousDrawPos,
			&mut ActionState<ClientAction>,
			&CursorPos,
			&SelectedObject,
			&ActionState<SettingsAction>,
			&PaintInk,
		),
		With<MainUser>,
	>,
	mut client: ResMut<ConnectionManager>,
) {
	let (
		mut last_draw,
		mut on_previous_draw,
		mut action_client,
		cur_pos,
		selected_obj,
		action,
		paint,
	) = query_user.single_mut();
	let real_world_ray = CursorPos(cur_pos.xy().floor() + Vec2::splat(0.5));

	let real_last_drawed = if action.pressed(&SettingsAction::Shift) {
		gizmos.line_2d(last_draw.xy(), *real_world_ray, paint.0.with_alpha(1.0));
		last_draw.0
	} else {
		on_previous_draw.0.unwrap_or(real_world_ray)
	};

	if (!action.pressed(&SettingsAction::Primary) && !action.pressed(&SettingsAction::Secondary))
		|| selected_obj.group.is_empty()
		|| (action.pressed(&SettingsAction::Shift)
			&& !action.just_pressed(&SettingsAction::Primary))
	{
		on_previous_draw.set_if_neq(PreviousDrawPos(None));
		action_client.release(&ClientAction::Drawing);
		return;
	}

	if on_previous_draw.set_if_neq(PreviousDrawPos(Some(real_world_ray))) {
		action_client.press(&ClientAction::Drawing);
		let draw_from_to = CursorFromTo {
			from: real_last_drawed,
			to: real_world_ray,
		};
		let draw_way = if action.pressed(&SettingsAction::Secondary) {
			DrawingWay::Erase
		} else {
			DrawingWay::Color
		};
		client
			.send_message_to_target::<MainChannel, PenDraw>(
				&mut PenDraw(draw_from_to, draw_way),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	}

	last_draw.set_if_neq(LastDrawPos(real_world_ray));
}

pub(super) fn hide_object(
	query_user: Query<(&SelectedObject, &ActionState<ToolsStandAlone>), With<MainUser>>,
	mut query_object: Query<&mut Visibility, With<ObjectWorld>>,
) {
	let Ok((selected_obj, action)) = query_user.get_single() else {
		return;
	};

	if !action.just_pressed(&ToolsStandAlone::Hide) {
		return;
	}

	selected_obj.group.iter().for_each(|ent| {
		let Ok(mut visibility) = query_object.get_mut(*ent) else {
			return;
		};
		*visibility = match *visibility {
			Visibility::Hidden => Visibility::Inherited,
			_ => Visibility::Hidden,
		};
	});
}

pub(super) fn color_swap(
	mut query_user: Query<(&ActionState<ToolsStandAlone>, &mut PaintInk), With<MainUser>>,
) {
	let (action, mut paint) = query_user.single_mut();
	if action.just_pressed(&ToolsStandAlone::ColorSwap) {
		(paint.0, paint.1) = (paint.1, paint.0);
	}
}

pub(super) fn receive_draw_pen(
	mut cmd: Commands,
	mut events: EventReader<MessageEvent<ToClientEntDataEvent<PenDraw>>>,
) {
	events.read().for_each(|event| {
		let msg = event.message();
		cmd.trigger_targets(msg.data, msg.ent);
	});
}

pub(super) fn pending_image_object(
	query_object: Query<
		Entity,
		(
			With<Confirmed>,
			With<ObjectImage>,
			Without<Handle<Image>>,
			Without<PendingImage>,
		),
	>,
	mut client: ResMut<ConnectionManager>,
	mut cmd: Commands,
) {
	query_object.iter().for_each(|ent_obj| {
		cmd.entity(ent_obj).insert(PendingImage);
		client
			.send_message_to_target::<MainChannel, RequestImageData>(
				&mut RequestImageData(ent_obj),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

pub(super) fn receive_image_data(
	mut events: EventReader<MessageEvent<ReceiveImageData>>,
	mut cmd: Commands,
) {
	events.read().for_each(|event| {
		let rec_img = event.message();
		let decoded_data = rec_img.data().uncompress().unwrap();
		let (width, height) = rec_img.size.into();
		let new_img = RgbaImage::from_vec(width, height, decoded_data).unwrap();
		cmd.entity(rec_img.ent)
			.insert(ProcessImage(new_img))
			.remove::<PendingImage>();
	});
}

pub(super) fn send_action_net(
	mut client: ResMut<ConnectionManager>,
	mut action_diff_events: EventReader<ActionDiffEvent<VerifyAction>>,
	mut action_diff_common_events: EventReader<ActionDiffEvent<ClientAction>>,
	query_user: Query<Entity, With<MainUser>>,
) {
	action_diff_events.read().for_each(|event| {
		if !query_user.contains(event.owner.unwrap()) {
			return;
		}
		let mut whole = event.clone();
		client
			.send_message::<MainChannel, Vec<ActionDiff<VerifyAction>>>(&mut whole.action_diffs)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});

	action_diff_common_events.read().for_each(|event| {
		if !query_user.contains(event.owner.unwrap()) {
			return;
		}
		let mut whole = event.clone();
		client
			.send_message::<MainChannel, Vec<ActionDiff<ClientAction>>>(&mut whole.action_diffs)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

pub(super) fn replicate_input_client(
	mut events: EventReader<MessageEvent<ToClientEntDataEvent<Vec<ActionDiff<ClientAction>>>>>,
	mut query_user: Query<&mut ActionState<ClientAction>, With<UserId>>,
) {
	events.read().for_each(|event| {
		let user = event.message().ent;
		let mut action = query_user.get_mut(user).unwrap();
		event.message().data.iter().for_each(|diff| {
			action.apply_diff(diff);
		});
	});
}

pub(super) fn request_point(
	mut client: ResMut<ConnectionManager>,
	mut cmd: Commands,
	query_point: Query<
		Entity,
		(
			With<ObjectPoint>,
			Without<Predicted>,
			Without<RequestedPoint>,
		),
	>,
) {
	query_point.iter().for_each(|ent_point| {
		client
			.send_message_to_target::<MainChannel, RequestingPointRelation>(
				&mut RequestingPointRelation(ent_point),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
		cmd.entity(ent_point).insert(RequestedPoint);
	});
}

pub(super) fn connect_point(
	mut events_point_to_point: EventReader<MessageEvent<ConnectRelations<PointToPoint>>>,
	mut cmd: Commands,
	query_point: Query<Entity, (With<ObjectPoint>, With<Confirmed>)>,
) {
	events_point_to_point.read().for_each(|event| {
		let ConnectRelations { ent1, ent2, .. } = event.message().clone();
		if query_point.contains(ent1) && query_point.contains(ent2) {
			cmd.trigger(event.message().clone());
		}
	});
}

pub(super) fn insert_as_objects(
	trigger: Trigger<OnAdd, (TextValue, ObjectPoint, ObjectPath)>,
	query_object: Query<Entity, Without<Predicted>>,
	mut cmd: Commands,
) {
	let ent = trigger.entity();
	if query_object.contains(ent) {
		cmd.entity(ent).insert(ObjectWorld);
	}
}

// fn set_selected_change(
// 	_trigger: Trigger<OnAdd, UserId>,
// 	mut query_user: Query<&mut SelectedObject, With<MainUser>>,
// ) {
// 	let previous = query_user.single().clone();
// 	let mut selected_obj = query_user.single_mut();
// 	if let Some(single) = previous.single {
// 		selected_obj.single = Some(single);
// 	}
// 	previous.group.iter().for_each(|ent| {
// 		selected_obj.group.insert(*ent);
// 	});
// }

pub(super) fn pen_marker(
	mut gizmos: Gizmos,
	mut query_user: Query<
		(
			&mut LastDrawPos,
			&mut PreviousDrawPos,
			&CursorPos,
			&ActionState<SettingsAction>,
			&PaintInk,
		),
		With<MainUser>,
	>,
	mut client: ResMut<ConnectionManager>,
) {
	let (mut last_draw, mut on_previous_draw, cur_pos, action, paint) = query_user.single_mut();

	let real_last_drawed = if action.pressed(&SettingsAction::Shift) {
		gizmos.line_2d(last_draw.xy(), cur_pos.xy(), paint.0.with_alpha(1.0));
		last_draw.0
	} else {
		on_previous_draw.0.unwrap_or(*cur_pos)
	};

	let mut mark_send = if action.just_pressed(&SettingsAction::Secondary) {
		MarkerType::Circle(cur_pos.xy())
	} else {
		if !action.pressed(&SettingsAction::Primary) {
			on_previous_draw.set_if_neq(PreviousDrawPos(None));
			return;
		}

		if let Some(prev_draw_pos) = on_previous_draw.0 {
			if prev_draw_pos.xy().distance(cur_pos.xy()) < 0.4 {
				return;
			}
		}
		if on_previous_draw.set_if_neq(PreviousDrawPos(Some(*cur_pos))) {
			last_draw.set_if_neq(LastDrawPos(*cur_pos));
			let draw_from_to = CursorFromTo {
				from: real_last_drawed,
				to: *cur_pos,
			};
			MarkerType::Line(draw_from_to)
		} else {
			return;
		}
	};

	client
		.send_message_to_target::<DisplayChannel, MarkerType>(&mut mark_send, NetworkTarget::All)
		.unwrap_or_else(|e| {
			error!("Fail to send message: {:?}", e);
		});
}

pub(super) fn paste_from_clip_board(
	query_user: Query<&ActionState<ToolsStandAlone>, With<MainUser>>,
	mut client: ResMut<ConnectionManager>,
	mut cmd: Commands,
) {
	let Ok(action) = query_user.get_single() else {
		return;
	};
	if !action.just_pressed(&ToolsStandAlone::Paste) {
		return;
	}
	let mut clip_board = Clipboard::new().unwrap();

	let Ok(ImageData {
		width,
		height,
		bytes,
	}) = clip_board.get_image()
	else {
		return;
	};

	let size = (width as u32, height as u32).into();
	let mut img_net = ImageNetwork::new("Pasted Image".to_owned(), &bytes, size);

	let compress_len = img_net.data().len();
	if compress_len > 290000 {
		let warned = format!("Image Data unable to compress below 290000 bit: {compress_len}");
		cmd.trigger(DisplayMsgEvent(warned));
		return;
	}

	client
		.send_message::<MainChannel, ImageNetwork>(&mut img_net)
		.unwrap();
}
