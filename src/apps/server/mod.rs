use aery::prelude::*;
use bevy::prelude::*;
// use bevy::tasks::IoTaskPool;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use lightyear::connection::netcode::PRIVATE_KEY_BYTES;
use lightyear::prelude::*;
// use rainful_diffolor::source_to_docs;
use server::*;
use std::collections::HashSet;
// use std::fs::File;
// use std::io::Write;
use std::sync::{Arc, RwLock};

use crate::apps::shared::prelude::*;
use crate::camera_control::lib::MAX_VALID_RANGE;
use crate::trait_bevy::ApplyDiff;
mod connection;
mod lib;
mod performing;
use connection::*;
use lib::*;
use performing::*;

use super::shared::proto::MainChannel;

pub(super) struct AppServerPlugin;
impl Plugin for AppServerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(build_server_plugin())
			.add_systems(
				OnEnter(RdioServerState::Online),
				(start_server, insert_server_source),
			)
			.add_systems(
				OnExit(RdioServerState::Online),
				(stop_connection, remove_server_source),
			)
			.add_systems(
				PreUpdate,
				(
					replicate_cursor.in_set(ServerReplicationSet::ClientReplication),
					(replicate_input_client, replicate_input_verify)
						.after(MainSet::EmitEvents)
						.in_set(InputManagerSystem::ManualControl),
				)
					.run_if(in_state(NetworkingState::Started)),
			)
			.add_systems(
				Update,
				(
					move_object,
					delete_object,
					spawn_point,
					spawn_text,
					process_incoming_message,
					receive_point_request,
					verify_action::<PenDraw>,
					verify_action::<ObjectBirNet>,
					// receive_marker_pen_draw,
					receive_img_data,
					spawn_new_image,
					send_image_data,
					obj_action,
					path_apply_color,
					toggle_close,
					obj_permission,
				)
					.run_if(in_state(NetworkingState::Started)),
			)
			.add_systems(
				PostUpdate,
				update_z_layer
					.run_if(resource_exists_and_changed::<ObjectOrderZ>)
					.run_if(in_state(NetworkingState::Started)),
			);
	}
}

fn path_apply_color(
	query_user: Query<(&ActionState<VerifyAction>, &SelectedObject, &PaintInk), With<UserId>>,
	query_point: Query<&Parent, With<ObjectPoint>>,
	mut query_object: Query<(&mut StrokeNet, &mut FillNet), With<ObjectWorld>>,
) {
	query_user.iter().for_each(|(action, selected_obj, paint)| {
		if !action.pressed(&VerifyAction::PathApplyColor) {
			return;
		}

		let Some(Some(point_parent)) = selected_obj
			.single
			.map(|ent_main| query_point.get(ent_main).ok())
		else {
			return;
		};
		let Ok((mut stroke, mut fill)) = query_object.get_mut(point_parent.get()) else {
			return;
		};
		let color_stroke = paint.0.into();
		let color_fill = paint.1.into();
		stroke.color.apply_diff(color_stroke);
		fill.color.apply_diff(color_fill);
	});
}

fn toggle_close(
	query_user: Query<(&ActionState<VerifyAction>, &SelectedObject), With<UserId>>,
	query_point: Query<&Parent, With<ObjectPoint>>,
	mut query_path: Query<&mut PathClose, With<ObjectWorld>>,
) {
	query_user.iter().for_each(|(action, selected_obj)| {
		if !action.just_pressed(&VerifyAction::ToggleClose) {
			return;
		}

		let all_path = query_point
			.iter_many(selected_obj.group.iter())
			.map(|parent| parent.get())
			.collect::<HashSet<Entity>>();
		let mut path_query = query_path.iter_many_mut(all_path.iter());
		while let Some(mut close) = path_query.fetch_next() {
			close.0 = !close.0;
		}
	});
}

pub(super) fn obj_action(
	mut query_object: Query<
		(
			AnyOf<(&mut MoveLock, &mut PixelLock, &mut AlphaLock)>,
			&ObjectAccess,
		),
		With<ObjectWorld>,
	>,
	mut events: EventReader<MessageEvent<ObjectActionToServer>>,
	mut layers: ResMut<ObjectOrderZ>,
) {
	events.read().for_each(|event| {
		let ent_obj = event.message().obj_ent;
		let Ok(((mut move_lock, mut pix_lock, mut alpha_lock), access)) =
			query_object.get_mut(ent_obj)
		else {
			return;
		};
		let action = event.message().action;
		let client_id = event.context();
		if !access.targets(client_id) {
			return;
		}
		let add_or_remove = |set: &mut HashSet<ClientId>| {
			if !set.remove(client_id) {
				set.insert(*client_id);
			}
		};
		match action {
			ObjectActionNet::LockMove => {
				if let Some(ref mut hashed_set) = move_lock {
					add_or_remove(&mut hashed_set.0);
				}
			},
			ObjectActionNet::LockPixel => {
				if let Some(ref mut hashed_set) = pix_lock {
					add_or_remove(&mut hashed_set.0);
				}
			},
			ObjectActionNet::LockAlpha => {
				if let Some(ref mut hashed_set) = alpha_lock {
					add_or_remove(&mut hashed_set.0);
				}
			},
			ObjectActionNet::LayerUp => {
				let Some(pull_up) = layers.iter().position(|ent| ent == &ent_obj) else {
					return;
				};
				let pull_down = pull_up + 1;
				if pull_down >= layers.len() {
					return;
				}
				layers.swap(pull_down, pull_up);
			},
			ObjectActionNet::LayerDown => {
				let Some(pull_down) = layers.iter().position(|ent| ent == &ent_obj) else {
					return;
				};
				let Some(pull_up) = pull_down.checked_sub(1) else {
					return;
				};
				layers.swap(pull_down, pull_up);
			},
		}
	});
}

pub(super) fn obj_permission(
	mut query_object: Query<(&ObjectOwner, &mut ObjectAccess), With<ObjectWorld>>,
	mut events: EventReader<MessageEvent<PerActionNet>>,
) {
	events.read().for_each(|event| {
		let ent_obj = event.message().obj_ent;
		let Ok((owner, mut access)) = query_object.get_mut(ent_obj) else {
			return;
		};
		let client_id = event.context();
		if &owner.0 != client_id {
			return;
		}
		match event.message().action.clone() {
			PerAction::Add(client_id) => access.add_id(client_id),
			PerAction::Remove(client_id) => access.remove_id(&client_id),
			PerAction::Change(mut network_target) => {
				if let NetworkTarget::Only(_) = network_target {
					network_target.add_id(*client_id);
				}
				access.0 = network_target;
			},
		}
	});
}

fn spawn_new_image(
	mut cmd: Commands,
	query_user: Query<(&ActionState<VerifyAction>, &CursorPos, &UserId)>,
) {
	query_user.iter().for_each(|(action, cur_pos, user_id)| {
		if !action.just_pressed(&VerifyAction::NewImage) {
			return;
		}
		let size = UVec2::splat(500);
		let invert = size.as_vec2() / Vec2::new(2.0, -2.0);
		let total = size.element_product() * 4;
		cmd.spawn(RdioImageBundle::new(
			"New Image",
			size,
			vec![0; total as usize],
			(cur_pos.xy() - invert).round(),
			user_id.0,
		));
	});
}

// fn save_server_scene(world: &mut World) {
// 	let mut scene_world = World::new();
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
// 			File::create(path.join("server_scene.scn.ron"))
// 				.and_then(|mut file| file.write(serialize_scened.as_bytes()))
// 				.expect("Error writing scene to file");
// 		})
// 		.detach();
// }

fn receive_point_request(
	mut events: EventReader<MessageEvent<RequestingPointRelation>>,
	mut server: ResMut<ConnectionManager>,
	query_point_to_point: Query<&Parent, With<ObjectPoint>>,
	query_path: Query<&Children, With<ObjectPath>>,
) {
	events.read().for_each(|event| {
		let ent_point = event.message().0;
		let Ok(parent) = query_point_to_point.get(ent_point) else {
			warn!("Invalid Request for point");
			return;
		};
		let pointes = query_path.get(parent.get()).unwrap();
		let pos_n = pointes.iter().position(|ent| ent == &ent_point).unwrap();
		let Some(below) = pos_n.checked_sub(1) else {
			return;
		};
		let point_last = pointes.get(below).unwrap();

		let mut relationser = ConnectRelations::<PointToPoint>::new(*point_last, ent_point);
		server
			.send_message_to_target::<MainChannel, _>(
				&mut relationser,
				NetworkTarget::Single(event.context),
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

fn spawn_point(
	mut cmd: Commands,
	query_user: Query<(
		&SelectedObject,
		&CursorPos,
		&ActionState<VerifyAction>,
		&UserId,
	)>,
	query_parent: Query<&Parent, With<ObjectPoint>>,
	query_leaf: Query<(Entity, &Parent), Leaf<PointToPoint>>,
) {
	query_user
		.iter()
		.for_each(|(selected_obj, cur_pos, action, user_id)| {
			if !action.just_pressed(&VerifyAction::AddPoint) {
				return;
			}

			let new_point = cmd.spawn(PointBundle::new(cur_pos.xy())).id();

			let mut spawn_pre_point = || -> Entity {
				let spawn_pre_point = cmd
					.spawn(PointBundle::new(cur_pos.xy() - Vec2::new(12.0, 0.0)))
					.id();
				cmd.spawn(RdioPathBundle::new("The Main Path", user_id.0))
					.push_children(&[spawn_pre_point, new_point]);
				spawn_pre_point
			};

			let pre_point = if let Some(main_ent) = selected_obj.single {
				if let Ok(parent) = query_parent.get(main_ent) {
					let ent_parent = parent.get();
					cmd.entity(ent_parent).add_child(new_point);
					query_leaf
						.iter()
						.find_map(|(ent, parent)| (parent.get() == ent_parent).then_some(ent))
						.unwrap()
				} else {
					spawn_pre_point()
				}
			} else {
				spawn_pre_point()
			};

			let relationser = ConnectRelations::<PointToPoint>::new(pre_point, new_point);
			cmd.trigger(relationser);
		});
}

fn spawn_text(
	query_user: Query<(&CursorPos, &ActionState<VerifyAction>, &UserId)>,
	mut cmd: Commands,
) {
	query_user.iter().for_each(|(cur_pos, action, user_id)| {
		if !action.just_pressed(&VerifyAction::AddText) {
			return;
		}
		cmd.spawn(WorldTextBundle::new(
			"New Text",
			"Hello World".to_owned(),
			cur_pos.xy(),
			user_id.0,
		));
	});
}

fn move_object(
	mut events: EventReader<MessageEvent<MovedPoint>>,
	mut query_object: Query<
		(
			&mut ObjectPosition,
			Option<&MoveLock>,
			Option<&Parent>,
			Has<ProcessImage>,
			Option<&ObjectAccess>,
		),
		With<ObjectWorld>,
	>,
	query_user: Query<&SelectedObject, With<UserId>>,
	query_path: Query<(&MoveLock, &ObjectAccess), With<ObjectPath>>,
	users: Res<Users>,
) {
	events.read().for_each(|event| {
		let client_id = event.context();
		let Some(ent_user) = users.get(client_id) else {
			return;
		};
		let Ok(selected_obj) = query_user.get(*ent_user) else {
			return;
		};

		let mut selected_exist = query_object.iter_many_mut(selected_obj.group.iter());
		while let Some((mut obj_pos, op_move_lock, op_parent, is_image, op_access)) =
			selected_exist.fetch_next()
		{
			let move_lock =
				op_move_lock.unwrap_or_else(|| query_path.get(op_parent.unwrap().get()).unwrap().0);
			if move_lock.contains(client_id)
				|| !op_access
					.unwrap_or_else(|| query_path.get(op_parent.unwrap().get()).unwrap().1)
					.targets(client_id)
			{
				continue;
			}
			if is_image {
				let moved = event.message().pixel.xy().as_vec2();
				obj_pos.0 = (obj_pos.0 + moved).round();
			} else {
				let moved = event.message().world.xy();
				obj_pos.0 += moved;
			}
			let maxed = Vec2::splat(MAX_VALID_RANGE);
			obj_pos.0 = obj_pos.0.clamp(-maxed, maxed);
		}
	});
}
