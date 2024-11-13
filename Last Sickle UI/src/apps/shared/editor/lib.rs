use bevy::{ecs::system::SystemParam, prelude::*, tasks::Task};
use bevy_cosmic_edit::{CosmicBuffer, CosmicFontSystem};
use lightyear::prelude::*;
use sickle_ui::{
	prelude::*,
	widgets::{layout::tab_container::TabContainer, menus::menu_bar::MenuBar},
};
use std::{
	collections::HashMap,
	net::{IpAddr, Ipv4Addr, SocketAddr},
};

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6000);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000);
pub const SERVER_ADDR_BACKEND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5005);

#[derive(Component)]
pub struct UiMainRootNode;

#[derive(Component)]
pub struct LockableOnUI;

#[derive(Component)]
pub struct NameInput;

#[derive(Component)]
pub(super) struct ChatMessageHolder;

#[derive(Component)]
pub(super) struct TypeChat;

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct EditorParts(HashMap<EditorPosition, Entity>);

impl EditorParts {
	fn part(&self, pos: &EditorPosition) -> Entity {
		*self.get(pos).unwrap()
	}
}

#[derive(SystemParam)]
pub struct EditorRdio<'w, 's> {
	pub cmd: Commands<'w, 's>,
	editor_part: Res<'w, EditorParts>,
	query_tab_container: Query<'w, 's, &'static TabContainer>,
}

#[allow(unused)]
impl<'w, 's> EditorRdio<'w, 's> {
	pub fn bar_1(&mut self) -> UiBuilder<(Entity, MenuBar)> {
		let ent = self.editor_part.part(&EditorPosition::MenuBar1);
		self.cmd.ui_builder((ent, MenuBar))
	}
	pub fn bar_2(&mut self) -> UiBuilder<(Entity, MenuBar)> {
		let ent = self.editor_part.part(&EditorPosition::MenuBar2);
		self.cmd.ui_builder((ent, MenuBar))
	}
	pub fn middle_panel(&mut self) -> UiBuilder<Entity> {
		let ent = self.editor_part.part(&EditorPosition::PanelMiddle);
		self.cmd.ui_builder(ent)
	}
	pub fn left_panel(&mut self) -> UiBuilder<Entity> {
		let ent = self.editor_part.part(&EditorPosition::PanelLeft);
		self.cmd.ui_builder(ent)
	}
	pub fn left_top_panel(&mut self) -> UiBuilder<Entity> {
		let ent = self.editor_part.part(&EditorPosition::PanelLeftTop);
		self.cmd.ui_builder(ent)
	}
	pub fn left_bottom_panel(&mut self) -> UiBuilder<(Entity, TabContainer)> {
		let ent = self.editor_part.part(&EditorPosition::PanelLeftBottom);
		let tab_container = self.query_tab_container.get(ent).unwrap();
		self.cmd.ui_builder((ent, *tab_container))
	}
	pub fn right_panel(&mut self) -> UiBuilder<Entity> {
		let ent = self.editor_part.part(&EditorPosition::PanelRight);
		self.cmd.ui_builder(ent)
	}
	pub fn right_top_panel(&mut self) -> UiBuilder<(Entity, TabContainer)> {
		let ent = self.editor_part.part(&EditorPosition::PanelRightTop);
		let tab_container = self.query_tab_container.get(ent).unwrap();
		self.cmd.ui_builder((ent, *tab_container))
	}
	pub fn right_bottom_panel(&mut self) -> UiBuilder<(Entity, TabContainer)> {
		let ent = self.editor_part.part(&EditorPosition::PanelRightBottom);
		let tab_container = self.query_tab_container.get(ent).unwrap();
		self.cmd.ui_builder((ent, *tab_container))
	}
	pub fn bottom_panel(&mut self) -> UiBuilder<Entity> {
		let ent = self.editor_part.part(&EditorPosition::PanelBottom);
		self.cmd.ui_builder(ent)
	}
	pub fn bottom_left_panel(&mut self) -> UiBuilder<(Entity, TabContainer)> {
		let ent = self.editor_part.part(&EditorPosition::PanelBottomLeft);
		let tab_container = self.query_tab_container.get(ent).unwrap();
		self.cmd.ui_builder((ent, *tab_container))
	}
	pub fn bottom_right_panel(&mut self) -> UiBuilder<(Entity, TabContainer)> {
		let ent = self.editor_part.part(&EditorPosition::PanelBottomRight);
		let tab_container = self.query_tab_container.get(ent).unwrap();
		self.cmd.ui_builder((ent, *tab_container))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorPosition {
	MenuBar1,
	MenuBar2,
	PanelMiddle,
	PanelLeft,
	PanelLeftTop,
	PanelLeftBottom,
	PanelRight,
	PanelRightTop,
	PanelRightBottom,
	PanelBottom,
	PanelBottomLeft,
	PanelBottomRight,
}

#[derive(Resource)]
pub struct ConnectTokenRequestTask {
	pub auth_backend_addr: SocketAddr,
	pub task: Option<Task<Option<ConnectToken>>>,
}

impl Default for ConnectTokenRequestTask {
	fn default() -> Self {
		Self {
			auth_backend_addr: SERVER_ADDR_BACKEND,
			task: None,
		}
	}
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum RdioClientState {
	#[default]
	Offline,
	Online,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum RdioServerState {
	#[default]
	Offline,
	Online,
}

#[derive(Component)]
pub(super) struct ClientPort;
#[derive(Component)]
pub(super) struct ServerPort;

#[derive(Component)]
pub(super) struct ConnectionIP;

pub(super) trait SetTextOnly {
	fn set_text_only(&mut self, font_system: &mut CosmicFontSystem, text: &str);
}

impl SetTextOnly for CosmicBuffer {
	fn set_text_only(&mut self, font_system: &mut CosmicFontSystem, text: &str) {
		let buffed = self.0.lines.first().unwrap();
		let attrs_list = buffed.attrs_list().clone();
		self.set_text(font_system, text, attrs_list.defaults());
	}
}
