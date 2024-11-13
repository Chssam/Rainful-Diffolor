// use std::{fs::File, io::Write};

use super::{world_view::LocalViewPlugin, *};
use crate::camera_control::ui_unfocus;

// use bevy::tasks::IoTaskPool;
use bevy_mod_picking::prelude::*;
use editor::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};
// use moonshine_save::load::load_from_file;
// use rainful_diffolor::source_to_docs;

// pub const USER_SETTING: &str = "user_setting.ron";

pub(super) struct MegaEditorPlugin;
impl Plugin for MegaEditorPlugin {
	fn build(&self, app: &mut App) {
		// let doc_path = source_to_docs().unwrap().join(USER_SETTING);
		app.init_state::<EditorTools>()
			.add_sub_state::<DropPathMode>()
			.add_plugins((
				LocalViewPlugin,
				InputManagerPlugin::<EditorTools>::default(),
				InputManagerPlugin::<SettingsAction>::default(),
				InputManagerPlugin::<ToolsStandAlone>::default(),
			))
			.add_systems(
				OnEnter(RdioClientState::Online),
				(
					// load_from_file(doc_path),
					setup_client_resources,
					editor_ui,
					add_brush,
				)
					.chain(),
			)
			.add_systems(
				PreUpdate,
				(
					request_point.after(MainSet::Send),
					update_cursor_position.run_if(ui_unfocus),
					hide_object.after(InputManagerSystem::ManualControl),
					(connect_point, replicate_input_client).after(MainSet::Receive),
				)
					.run_if(in_state(RdioClientState::Online)),
			)
			.add_systems(
				Update,
				(
					(
						detect_file_drop.run_if(in_state(DropPathMode::AsObject)),
						detect_path_directory.run_if(in_state(DropPathMode::SaveLocation)),
						load_when_ready,
					),
					(
						pick_object.run_if(in_state(EditorTools::Pick)),
						color_pick.run_if(in_state(EditorTools::ColorPick)),
						edit_path.run_if(in_state(EditorTools::Path)),
						pencel_line.run_if(in_state(EditorTools::Pencel)),
						pen_marker.run_if(in_state(EditorTools::Marker)),
						resize_img.run_if(
							in_state(EditorTools::Resize).or_else(in_state(EditorTools::Scale)),
						),
					)
						.run_if(ui_unfocus),
					(
						change_editor_tools,
						main_move_object,
						color_swap,
						export_single_image_png,
						copy_or_canvas,
						export_svg,
						paste_from_clip_board,
					),
					// (
					//     // line_path.run_if(in_state(EditorTools::Path)),
					//     // crop_2d.run_if(in_state(EditorTools::Crop)),
					//     // geometric_transform.run_if(in_state(EditorTools::Scale)),
					// )
					//     .run_if(cured_zone),
					pending_image_object,
					receive_image_data,
					receive_draw_pen,
				)
					.run_if(in_state(RdioClientState::Online)),
			)
			.add_systems(
				PostUpdate,
				(
					release_done::<EditorTools>,
					release_done::<VerifyAction>,
					generate_action_diffs::<VerifyAction>.before(send_action_net),
					generate_action_diffs::<ClientAction>.before(send_action_net),
					send_action_net,
				)
					.run_if(in_state(RdioClientState::Online)),
			);
	}
}

fn release_done<T: Actionlike>(mut query_action: Query<&mut ActionState<T>>) {
	query_action.iter_mut().for_each(|mut action| {
		for act in action.get_just_pressed() {
			action.release(&act);
		}
	});
}

// fn save_user_tooling_scene(world: &mut World) {
// 	let mut scene_world = World::new();
// 	// let main_user_ent = world.query_filtered::<Entity, With<MainUser>>().single(world);
// 	// let main_user = world;
// 	// let a = RdioImageBundle::from_world(world);

// 	let type_registry = world.resource::<AppTypeRegistry>().clone();
// 	scene_world.insert_resource(type_registry);
// 	let scene = DynamicScene::from_world(&scene_world);

// 	let type_registry = world.resource::<AppTypeRegistry>();
// 	let type_registry = type_registry.read();
// 	let serialize_scened = scene.serialize(&type_registry).unwrap();

// 	let Some(path) = source_to_docs() else {
// 		return;
// 	};
// 	IoTaskPool::get()
// 		.spawn(async move {
// 			File::create(path.join("user_tooling_scene.scn.ron"))
// 				.and_then(|mut file| file.write(serialize_scened.as_bytes()))
// 				.expect("Error writing scene to file");
// 		})
// 		.detach();
// }

fn edit_path(
	mut on_click: EventReader<Pointer<Down>>,
	mut query_user: Query<
		(
			&mut BeginSelectPoint,
			&ActionState<SettingsAction>,
			&mut ActionState<VerifyAction>,
			&mut SelectedObject,
			&CursorPos,
		),
		With<MainUser>,
	>,
	query_point: Query<(Entity, &ObjectPosition), (With<ObjectPoint>, With<ObjectWorld>)>,
) {
	let Ok((mut pin_point, actions_key, mut action_client, mut selected_obj, cur_pos)) =
		query_user.get_single_mut()
	else {
		return;
	};

	action_client.release(&VerifyAction::AddPoint);

	let mut sorted_depth = on_click.read().collect::<Vec<_>>();
	sorted_depth.sort_by_key(|a| a.hit.depth as i64);

	for pointer in sorted_depth {
		if pointer.button != PointerButton::Primary {
			continue;
		}
		let ent_obj = if query_point.contains(pointer.target()) {
			pointer.target()
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

	if actions_key.just_pressed(&SettingsAction::Primary)
		&& actions_key.pressed(&SettingsAction::Control)
	{
		action_client.press(&VerifyAction::AddPoint);
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
		query_point.iter().for_each(|(ent_obj, point_pos)| {
			if recter.contains(point_pos.xy()) {
				selected_obj.add_select(ent_obj);
			}
		});
	}
}

fn resize_img(
	query_object: Query<&ObjectPosition, With<ObjectWorld>>,
	mut query_user: Query<
		(
			Entity,
			&CursorPos,
			&mut ResizePinPoint,
			// &mut ResizeKind,
			&mut ScaleAction,
			&mut ScalePosition,
			&SelectedObject,
			&ActionState<SettingsAction>,
		),
		With<MainUser>,
	>,
	mut client: ResMut<ConnectionManager>,
) {
	let Ok((
		ent_user,
		position,
		mut resize_point,
		// mut resize_kind,
		mut scale_action,
		mut scale_pos_main,
		selected_obj,
		actions_key,
	)) = query_user.get_single_mut()
	else {
		return;
	};

	let real_world_ray = position.xy().ceil();

	let Some(ent_main) = selected_obj.group.iter().last() else {
		resize_point.0 = None;
		scale_action.0 = None;
		return;
	};

	let Ok(obj_pos) = query_object.get(*ent_main) else {
		return;
	};

	if actions_key.just_pressed(&SettingsAction::Primary) {
		resize_point.0 = Some(real_world_ray);
		let pos_obj = obj_pos.0;
		let little_box = Rect::from_center_half_size(pos_obj, Vec2::splat(20.0));
		let Rect { min, max } = little_box;
		let recty = little_box.size() * 0.3;

		let top_left = real_world_ray.x < min.x + recty.x && real_world_ray.y > max.y - recty.y;
		let top_right = real_world_ray.cmpge(max - recty).all();
		let bottom_left = real_world_ray.cmple(min + recty).all();
		let bottom_right = real_world_ray.x > max.x - recty.x && real_world_ray.y < min.y + recty.y;

		let top = real_world_ray.y > max.y - recty.y;
		let bottom = real_world_ray.y < min.y + recty.y;
		let left = real_world_ray.x < min.x + recty.x;
		let right = real_world_ray.x > max.x - recty.x;

		let scale_pos = if top_left {
			ScalePosition::TopLeft
		} else if top_right {
			ScalePosition::TopRight
		} else if bottom_left {
			ScalePosition::BottomLeft
		} else if bottom_right {
			ScalePosition::BottomRight
		} else if top {
			ScalePosition::Top
		} else if bottom {
			ScalePosition::Bottom
		} else if left {
			ScalePosition::Left
		} else if right {
			ScalePosition::Right
		} else {
			ScalePosition::Middle
		};

		*scale_pos_main = scale_pos;
	}

	let Some(a_local_pin) = resize_point.0 else {
		return;
	};

	if actions_key.just_pressed(&SettingsAction::Enter) {
		client
			.send_message_to_target::<MainChannel, ApplyChange>(
				&mut ApplyChange(true, ent_user),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
		resize_point.0 = None;
		return;
	}

	if actions_key.just_pressed(&SettingsAction::Escape) {
		client
			.send_message_to_target::<MainChannel, ApplyChange>(
				&mut ApplyChange(false, ent_user),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
		resize_point.0 = None;
		scale_action.0 = None;
		return;
	}

	if !actions_key.pressed(&SettingsAction::Primary) {
		return;
	}

	let diff_distance = a_local_pin - real_world_ray;

	scale_action.0 = Some(ScaleKind::Pixel(diff_distance));
}

// fn detect_specify_file(file_extension: &str) -> Vec<DirEntry> {
//     let mut vec_path = vec![];
//     let path = read_dir(file_extension).unwrap();
//     for file in path.into_iter() {
//         let deep_file = file.unwrap();
//         vec_path.push(deep_file);
//     }
//     vec_path
// }

// fn type_value_fn(query: Query<Entity, With<TypeActive>>) {}

// fn update_value_reflection() {}
