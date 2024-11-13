use async_compat::Compat;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, IoTaskPool};
use lightyear::prelude::{ClientId::Netcode, *};
use server::*;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncWriteExt;

use crate::apps::shared::shared_config;

use super::*;

pub(super) fn build_server_plugin() -> ServerPlugins {
	let io = IoConfig {
		transport: ServerTransport::UdpSocket(SERVER_ADDR),
		..default()
	};
	let net_config = NetConfig::Netcode {
		config: NetcodeConfig::default(),
		io,
	};
	let config = ServerConfig {
		shared: shared_config(),
		net: vec![net_config],
		replication: ReplicationConfig {
			send_updates_mode: SendUpdatesMode::SinceLastSend,
			..default()
		},
		..default()
	};
	ServerPlugins::new(config)
}

pub(super) fn start_server(
	mut cmd: Commands,
	server_config: Res<ServerConfig>,
	backend_task_server: Res<BackendTaskServer>,
) {
	let server::NetConfig::Netcode { io, .. } = &server_config.net[0];
	let ServerTransport::UdpSocket(socker) = &io.transport else {
		unreachable!();
	};
	let mut back_end_socket = *socker;
	back_end_socket.set_port(SERVER_ADDR_BACKEND.port());
	backend_task_server.write().unwrap().inactive = false;
	start_netcode_authentication_task(
		*socker,
		back_end_socket,
		0,
		[0; PRIVATE_KEY_BYTES],
		backend_task_server.clone(),
	);

	cmd.trigger(DisplayMsgEvent("START SERVER".to_owned()));
	cmd.start_server();
}

pub(super) fn stop_connection(mut cmd: Commands, backend_task_server: Res<BackendTaskServer>) {
	backend_task_server.write().unwrap().inactive = true;
	cmd.trigger(DisplayMsgEvent("Closed Server.".to_owned()));
	cmd.stop_server();
}

pub(super) fn handle_connect_events(
	trigger: Trigger<ConnectEvent>,
	client_ids: Res<BackendTaskServer>,
) {
	if let Netcode(client_id) = trigger.event().client_id {
		client_ids.write().unwrap().clients.insert(client_id);
	}
}

// pub(super) fn handle_disconnect_events(
// 	trigger: Trigger<DisconnectEvent>,
// 	client_ids: Res<BackendTaskServer>,
// 	query_user: Query<&SharingName>,
// 	mut server: ResMut<ConnectionManager>,
// 	mut users: ResMut<Users>,
// 	mut cmd: Commands,
// ) {
// 	let raw_client_id = trigger.event().client_id;
// 	let ent_user = users.get(&raw_client_id).unwrap();
// 	cmd.entity(*ent_user).despawn_recursive();
// 	users.remove(&raw_client_id);
// 	if let Netcode(client_id) = raw_client_id {
// 		client_ids.write().unwrap().clients.remove(&client_id);
// 		let name = query_user.get(trigger.entity()).unwrap();
// 		let msg = format!("{} left the server.", name.0);
// 		server
// 			.send_message_to_target::<MessageChannel, MessageCtx>(
// 				&mut MessageCtx(msg),
// 				NetworkTarget::AllExceptSingle(raw_client_id),
// 			)
// 			.unwrap_or_else(|e| {
// 				error!("Fail to send message: {:?}", e);
// 			});
// 	}
// }

pub(super) fn start_netcode_authentication_task(
	game_server_addr: SocketAddr,
	auth_backend_addr: SocketAddr,
	protocol_id: u64,
	private_key: Key,
	backend_runner: Arc<RwLock<BackendItem>>,
) {
	IoTaskPool::get()
		.spawn(Compat::new(async move {
			info!(
				"Listening for ConnectToken request on {}",
				auth_backend_addr
			);

			let listener = match tokio::net::TcpListener::bind(auth_backend_addr).await {
				Ok(v) => v,
				Err(e) => {
					error!("Fail to start server: {:?}", e);
					return;
				},
			};

			loop {
				if backend_runner.read().unwrap().inactive {
					break;
				}
				let Some(accepted) = block_on(future::poll_once(listener.accept())) else {
					continue;
				};
				let (mut stream, _) = accepted.unwrap();

				info!("Authing User");
				let client_id = loop {
					let client_id = rand::random();
					if !backend_runner.read().unwrap().clients.contains(&client_id) {
						break client_id;
					}
				};

				let token =
					ConnectToken::build(game_server_addr, protocol_id, client_id, private_key)
						.generate()
						.expect("Failed to generate token");

				let serialized_token = token.try_into_bytes().expect("Failed to serialize token");
				trace!(
					"Sending token {:?} to client {}, Token len: {}",
					serialized_token,
					client_id,
					serialized_token.len()
				);
				stream
					.write_all(&serialized_token)
					.await
					.expect("Failed to send token to client");
			}
			info!("Stopped listening: {:?}", listener.local_addr());
		}))
		.detach();
}
