use bevy::prelude::*;
use bevy_cosmic_edit::*;
use client::{ClientConfig, ClientTransport};
use lightyear::prelude::*;
use local_ip_address::local_ip;
use server::{ServerConfig, ServerTransport};
use std::net::IpAddr;

use super::*;

pub(super) fn client_system(
	server_state: Res<State<RdioClientState>>,
	mut next_server_state: ResMut<NextState<RdioClientState>>,
	mut query_addr: ParamSet<(
		Query<(Entity, &mut CosmicBuffer), With<ConnectionIP>>,
		Query<(Entity, &mut CosmicBuffer), With<ClientPort>>,
	)>,
	mut client_config: ResMut<ClientConfig>,
	mut connect_token_request: ResMut<ConnectTokenRequestTask>,
	mut font_system: ResMut<CosmicFontSystem>,
) {
	let Ok(ip) = query_addr.p0().single().1.get_text().parse::<IpAddr>() else {
		let ip_ip = CLIENT_ADDR.ip().to_string();
		if let Ok(mut iped) = query_addr.p0().get_single_mut() {
			iped.1.set_text_only(&mut font_system, &ip_ip);
		}
		return;
	};
	let Ok(port) = query_addr.p1().single().1.get_text().parse::<u16>() else {
		let port_port = CLIENT_ADDR.port().to_string();
		if let Ok(mut ported) = query_addr.p1().get_single_mut() {
			ported.1.set_text_only(&mut font_system, &port_port);
		}
		return;
	};

	let client::NetConfig::Netcode { io, .. } = &mut client_config.net else {
		unreachable!();
	};
	let ClientTransport::UdpSocket(socker) = &mut io.transport else {
		unreachable!();
	};
	let ip_ready = if ip.is_loopback() {
		CLIENT_ADDR.ip()
	} else {
		local_ip().unwrap()
	};
	socker.set_ip(ip_ready);
	socker.set_port(port);
	let mut auth_back_sock = SERVER_ADDR_BACKEND;
	auth_back_sock.set_ip(ip);
	connect_token_request.auth_backend_addr = auth_back_sock;
	let state = match server_state.get() {
		RdioClientState::Offline => RdioClientState::Online,
		RdioClientState::Online => RdioClientState::Offline,
	};
	next_server_state.set(state);
}

pub(super) fn server_system(
	server_state: Res<State<RdioServerState>>,
	mut next_server_state: ResMut<NextState<RdioServerState>>,
	mut query_addr: ParamSet<(
		Query<(Entity, &mut CosmicBuffer), With<ConnectionIP>>,
		Query<(Entity, &mut CosmicBuffer), With<ServerPort>>,
	)>,
	mut server_config: ResMut<ServerConfig>,
	mut font_system: ResMut<CosmicFontSystem>,
) {
	let Ok(ip) = query_addr.p0().single().1.get_text().parse::<IpAddr>() else {
		let ip_ip = SERVER_ADDR.ip().to_string();
		if let Ok(mut iped) = query_addr.p0().get_single_mut() {
			iped.1.set_text_only(&mut font_system, &ip_ip);
		}
		return;
	};
	let Ok(port) = query_addr.p1().single().1.get_text().parse::<u16>() else {
		let port_port = SERVER_ADDR.port().to_string();
		if let Ok(mut ported) = query_addr.p1().get_single_mut() {
			ported.1.set_text_only(&mut font_system, &port_port);
		}
		return;
	};
	let server::NetConfig::Netcode { io, .. } = &mut server_config.net[0];
	let ServerTransport::UdpSocket(socker) = &mut io.transport else {
		unreachable!();
	};
	socker.set_ip(ip);
	socker.set_port(port);
	let state = match server_state.get() {
		RdioServerState::Offline => RdioServerState::Online,
		RdioServerState::Online => RdioServerState::Offline,
	};
	next_server_state.set(state);
}

pub(super) fn select_ip_self(
	mut ip: Query<&mut CosmicBuffer, With<ConnectionIP>>,
	mut font_system: ResMut<CosmicFontSystem>,
) {
	let ip_ip = CLIENT_ADDR.ip().to_string();
	ip.single_mut().set_text_only(&mut font_system, &ip_ip);
}

pub(super) fn select_ip_local(
	mut ip: Query<&mut CosmicBuffer, With<ConnectionIP>>,
	mut font_system: ResMut<CosmicFontSystem>,
) {
	let ip_ip = local_ip().unwrap_or(CLIENT_ADDR.ip()).to_string();
	ip.single_mut().set_text_only(&mut font_system, &ip_ip);
}
