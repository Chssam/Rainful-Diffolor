use bevy::prelude::*;
use image::EncodableLayout;
use leafwing_input_manager::action_diff::ActionDiff;
use serde::de::DeserializeOwned;

use super::*;

pub(super) fn insert_server_source(world: &mut World) {
	let client_ids = Arc::new(RwLock::new(BackendItem::default()));
	world.init_resource::<Users>();
	world.init_resource::<ObjectOrderZ>();
	world.init_resource::<ObjectIncrementCount>();
	world.insert_resource(BackendTaskServer(client_ids));

	let all_server_observe = [
		world.observe(on_remove_objects).id(),
		world.observe(on_add_objects).id(),
		world.observe(handle_connect_events).id(),
		// world.observe(handle_disconnect_events).id(),
		world.observe(new_user_id).id(),
		world.observe(remove_user).id(),
	];
	all_server_observe.into_iter().for_each(|ent_obs| {
		world
			.commands()
			.entity(ent_obs)
			.insert(StateScoped(RdioServerState::Online));
	});
}

pub(super) fn remove_server_source(world: &mut World) {
	world.remove_resource::<Users>();
	world.remove_resource::<ObjectOrderZ>();
	world.remove_resource::<ObjectIncrementCount>();
	world.remove_resource::<BackendTaskServer>();
}

pub(super) fn replicate_cursor(
	mut cmd: Commands,
	mut users: ResMut<Users>,
	replicated_cursor: Query<(Entity, &Replicated), (With<CursorPos>, Added<Replicated>)>,
) {
	replicated_cursor.iter().for_each(|(ent_user, replicated)| {
		let client_id = replicated.client_id();
		users.insert(client_id, ent_user);
		cmd.entity(ent_user).insert((
			Replicate {
				target: ReplicationTarget {
					target: NetworkTarget::AllExceptSingle(client_id),
				},
				authority: AuthorityPeer::Client(client_id),
				sync: SyncTarget {
					interpolation: NetworkTarget::AllExceptSingle(client_id),
					..default()
				},
				group: ReplicationGroup::new_id(0).set_priority(0.5),
				..default()
			},
			InputManagerBundle::<VerifyAction>::default(),
		));
	});
}

pub(super) fn verify_action<T: Event + Clone + Copy + Serialize + DeserializeOwned>(
	mut events: EventReader<MessageEvent<T>>,
	mut server: ResMut<ConnectionManager>,
	mut cmd: Commands,
	users: Res<Users>,
) {
	events.read().for_each(|event| {
		let Some(user) = users.get(event.context()).copied() else {
			return;
		};
		let action = *event.message();
		server
			.send_message_to_target::<MainChannel, ToClientEntDataEvent<T>>(
				&mut ToClientEntDataEvent::<T>::new(action, user),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
		cmd.trigger_targets(action, user);
	});
}

pub(super) fn send_image_data(
	mut events: EventReader<MessageEvent<RequestImageData>>,
	mut server: ResMut<ConnectionManager>,
	query_object: Query<(&ProcessImage, &ReplicationTarget), With<ObjectImage>>,
) {
	events.read().for_each(|event| {
		let ent_obj = event.message().0;
		let Ok((proc_img, rep_target)) = query_object.get(ent_obj) else {
			warn!("Receive invalid entity image request");
			return;
		};
		let client_id = event.context;
		if !rep_target.target.targets(&client_id) {
			return;
		}
		server
			.send_message_to_target::<MainChannel, ReceiveImageData>(
				&mut ReceiveImageData::new(
					ent_obj,
					proc_img.as_bytes(),
					proc_img.dimensions().into(),
				),
				NetworkTarget::Single(client_id),
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

pub(super) fn update_z_layer(
	mut query_object: Query<&mut ObjectZLayer, (Without<ObjectPoint>, With<ObjectWorld>)>,
	layers: Res<ObjectOrderZ>,
) {
	let mut n = 0;
	let mut many_obj = query_object.iter_many_mut(layers.iter());
	while let Some(mut obj_z) = many_obj.fetch_next() {
		let cal = BEGIN_OBJ_Z_INDEX as i16 + n;
		obj_z.set_if_neq(ObjectZLayer(cal));
		n += 1;
	}
}

pub(super) fn receive_img_data(
	mut events: EventReader<MessageEvent<ImageNetwork>>,
	mut cmd: Commands,
	query_user: Query<&CursorPos>,
	users: Res<Users>,
) {
	if events.is_empty() {
		return;
	}
	events.read().for_each(|event| {
		let img_net = event.message();
		let user = users.get(event.context()).unwrap();
		let real_world_ray = query_user.get(*user).unwrap();
		let decode_data = match img_net.data().uncompress() {
			Ok(data) => data,
			Err(e) => {
				error!("Unable to uncompress image data: {:?}", e);
				return;
			},
		};

		let cost_pos = img_net.size.as_vec2() / Vec2::new(-2.0, 2.0);
		let center_img = (real_world_ray.floor() + cost_pos).round();

		cmd.spawn(RdioImageBundle::new(
			&img_net.name,
			img_net.size,
			decode_data,
			center_img,
			*event.context(),
		));
	});
}

fn new_user_id(
	trigger: Trigger<OnAdd, UserId>,
	query_user: Query<&UserId>,
	mut users: ResMut<Users>,
) {
	let ent_user = trigger.entity();
	let user_id = query_user.get(ent_user).unwrap();
	users.insert(user_id.0, ent_user);
}

fn on_remove_objects(trigger: Trigger<OnRemove, ObjectWorld>, mut layers: ResMut<ObjectOrderZ>) {
	let ent = trigger.entity();
	let Some(pos) = layers.iter().position(|v| v == &ent) else {
		return;
	};
	layers.remove(pos);
}

fn on_add_objects(
	trigger: Trigger<OnAdd, ObjectWorld>,
	query_point: Query<(), With<ObjectPoint>>,
	mut cmd: Commands,
	mut layers: ResMut<ObjectOrderZ>,
	mut count: ResMut<ObjectIncrementCount>,
) {
	let ent = trigger.entity();

	count.0 += 1;
	let replicate = Replicate {
		target: ReplicationTarget {
			target: NetworkTarget::All,
		},
		sync: SyncTarget {
			prediction: NetworkTarget::All,
			..default()
		},
		group: ReplicationGroup::new_id(count.0),
		..default()
	};
	cmd.entity(ent)
		.insert((replicate, StateScoped(RdioServerState::Online)));
	if !query_point.contains(ent) {
		layers.push(ent);
	}
}

pub(super) fn process_incoming_message(
	mut events: EventReader<MessageEvent<MessageCtx>>,
	mut server: ResMut<ConnectionManager>,
	mut cmd: Commands,
	users: Res<Users>,
	query_user: Query<&SharingName, With<UserId>>,
) {
	events.read().for_each(|event| {
		let Some(user_ent) = users.get(&event.context) else {
			return;
		};
		let Ok(user_name) = query_user.get(*user_ent) else {
			return;
		};
		let content = format!("[{}] {}", user_name.0, event.message.0);
		cmd.trigger(DisplayMsgEvent(content.clone()));
		server
			.send_message_to_target::<MessageChannel, MessageCtx>(
				&mut MessageCtx(content),
				NetworkTarget::All,
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

pub(super) fn delete_object(
	query_user: Query<(&SelectedObject, &ActionState<VerifyAction>, &UserId)>,
	query_object: Query<(Entity, &ObjectAccess), With<ObjectWorld>>,
	mut cmd: Commands,
) {
	query_user
		.iter()
		.for_each(|(selected_obj, action, user_id)| {
			if !action.just_pressed(&VerifyAction::DeleteObject) {
				return;
			}
			for (ent_obj, perm) in query_object.iter_many(selected_obj.group.iter()) {
				if !perm.targets(&user_id.0) {
					return;
				}
				if let Some(mut ent_cmd) = cmd.get_entity(ent_obj) {
					ent_cmd.despawn();
				}
			}
		});
}

pub(super) fn replicate_input_client(
	users: Res<Users>,
	mut events: EventReader<MessageEvent<Vec<ActionDiff<ClientAction>>>>,
	mut query_user: Query<&mut ActionState<ClientAction>, With<UserId>>,
	mut server: ResMut<ConnectionManager>,
) {
	events.read().for_each(|event| {
		let client_id = *event.context();
		let user = *users.get(&client_id).unwrap();
		let mut action = query_user.get_mut(user).unwrap();
		event.message().iter().for_each(|diff| {
			action.apply_diff(diff);
		});
		server
			.send_message_to_target::<MessageChannel, ToClientEntDataEvent<Vec<ActionDiff<ClientAction>>>>(
				&mut ToClientEntDataEvent {
					ent: user,
					data: event.message().clone(),
				},
				NetworkTarget::AllExceptSingle(client_id),
			)
			.unwrap_or_else(|e| {
				error!("Fail to send message: {:?}", e);
			});
	});
}

pub(super) fn replicate_input_verify(
	users: Res<Users>,
	mut events: EventReader<MessageEvent<Vec<ActionDiff<VerifyAction>>>>,
	mut query_user: Query<&mut ActionState<VerifyAction>, With<UserId>>,
) {
	events.read().for_each(|event| {
		let user = users.get(event.context()).unwrap();
		let mut action = query_user.get_mut(*user).unwrap();
		event.message().iter().for_each(|diff| {
			action.apply_diff(diff);
		});
	});
}

pub(super) fn remove_user(
	trigger: Trigger<DisconnectClient>,
	mut server: ResMut<ServerConnections>,
) {
	let client_id = trigger.event().0;
	server
		.disconnect(client_id)
		.unwrap_or_else(|err| warn!("Disconnect invalid ID: {:?}", err));
}

// pub(super) fn replicate_input<T: Actionlike>(
// 	users: Res<Users>,
// 	mut events: EventReader<MessageEvent<Vec<ActionDiff<T>>>>,
// 	mut query_user: Query<&mut ActionState<T>, With<UserId>>,
// ) {
// 	events.read().for_each(|event| {
// 		let user = users.get(event.context()).unwrap();
// 		let mut action = query_user.get_mut(*user).unwrap();
// 		event.message().iter().for_each(|diff| {
// 			action.apply_diff(diff);
// 		});
// 	});
// }
