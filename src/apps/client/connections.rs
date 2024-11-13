use async_compat::Compat;
use bevy::{
	prelude::*,
	tasks::{block_on, futures_lite::future, IoTaskPool},
};
use lightyear::prelude::*;
use lightyear::{connection::netcode::CONNECT_TOKEN_BYTES, prelude::client::*};
use std::net::SocketAddr;

use crate::apps::shared::shared_config;

use super::{ConnectTokenRequestTask, DisplayMsgEvent, CLIENT_ADDR};

pub(super) fn build_client_plugin() -> ClientPlugins {
	let auth = Authentication::None;
	let io = IoConfig {
		transport: ClientTransport::UdpSocket(CLIENT_ADDR),
		..default()
	};
	let net_config = NetConfig::Netcode {
		auth,
		config: NetcodeConfig::default(),
		io,
	};
	let config = ClientConfig {
		shared: shared_config(),
		net: net_config,
		replication: ReplicationConfig {
			send_updates_mode: SendUpdatesMode::SinceLastSend,
			..default()
		},
		..default()
	};
	ClientPlugins::new(config)
}

pub(super) fn fetch_connect_token(
	mut connect_token_request: ResMut<ConnectTokenRequestTask>,
	mut client_config: ResMut<ClientConfig>,
	mut cmd: Commands,
) {
	let Some(task) = &mut connect_token_request.task else {
		return;
	};
	let Some(op_connect_token) = block_on(future::poll_once(task)) else {
		return;
	};
	let Some(connect_token) = op_connect_token else {
		connect_token_request.task = None;
		return;
	};
	if let NetConfig::Netcode { auth, .. } = &mut client_config.net {
		*auth = Authentication::Token(connect_token);
	}
	cmd.connect_client();
	cmd.trigger(DisplayMsgEvent("Connected to server.".to_owned()));
	connect_token_request.task = None;
}

pub(super) fn activate_connect_token(
	mut cmd: Commands,
	mut connect_token_request: ResMut<ConnectTokenRequestTask>,
	client_config: Res<ClientConfig>,
) {
	let NetConfig::Netcode { auth, .. } = &client_config.net else {
		return;
	};
	if auth.has_token() {
		cmd.connect_client();
		return;
	}
	let auth_backend_addr = connect_token_request.auth_backend_addr;
	let task = IoTaskPool::get().spawn_local(Compat::new(async move {
		get_connect_token_from_auth_backend(auth_backend_addr).await
	}));
	connect_token_request.task = Some(task);
	cmd.trigger(DisplayMsgEvent(
		"Connecting server backend authorize.".to_owned(),
	));
}

pub(super) fn disconnect_from_server(mut cmd: Commands, mut client_config: ResMut<ClientConfig>) {
	cmd.disconnect_client();
	let NetConfig::Netcode { auth, .. } = &mut client_config.net else {
		return;
	};
	*auth = Authentication::None;
}

async fn get_connect_token_from_auth_backend(
	auth_backend_addr: SocketAddr,
) -> Option<ConnectToken> {
	let stream = match tokio::net::TcpStream::connect(auth_backend_addr).await {
		Ok(v) => v,
		Err(e) => {
			error!("Failed to connect to authentication server on: {:?}", e);
			return None;
		},
	};
	stream.readable().await.unwrap();
	let mut buffer = [0u8; CONNECT_TOKEN_BYTES];
	match stream.try_read(&mut buffer) {
		Ok(n) if n == CONNECT_TOKEN_BYTES => {
			trace!(
				"Received token bytes: {:?}. Token len: {:?}",
				buffer,
				buffer.len()
			);
			let Ok(token) = ConnectToken::try_from_bytes(&buffer) else {
				error!("Failed to parse token from authentication server");
				return None;
			};
			Some(token)
		},
		_ => {
			error!("Failed to read token from authentication server");
			None
		},
	}
}
