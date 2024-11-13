use bevy::{
	ecs::{
		component::{ComponentHooks, StorageType},
		system::SystemParam,
	},
	prelude::*,
	tasks::Task,
};
use bevy_cosmic_edit::{CosmicBuffer, CosmicEditBundle, CosmicFontSystem, CosmicSource};
use lightyear::prelude::*;
use std::{
	collections::HashMap,
	net::{IpAddr, Ipv4Addr, SocketAddr},
	sync::{Arc, RwLock},
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
pub struct EditorParts(HashMap<EditorPosition, Entity>);

impl EditorParts {
	pub fn part(&self, pos: &EditorPosition) -> Entity {
		*self.get(pos).unwrap()
	}
}

#[derive(SystemParam)]
pub struct EditorRdio<'w, 's> {
	pub cmd: Commands<'w, 's>,
	pub editor_part: Res<'w, EditorParts>,
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

impl Component for EditorPosition {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let ent_pos = world
				.entity(entity)
				.get::<EditorPosition>()
				.unwrap()
				.clone();
			let mut parts = world.resource_mut::<EditorParts>();
			parts.insert(ent_pos, entity);
		});
	}
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

pub struct OwnCosmicEdit<T: Bundle>(Option<(CosmicEditBundle, T)>);

impl<T: Bundle> OwnCosmicEdit<T> {
	pub fn new(cosmic: CosmicEditBundle, comp: T) -> Self {
		Self(Some((cosmic, comp)))
	}
}

impl<T: Bundle> Component for OwnCosmicEdit<T> {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let cosmiced = std::mem::take(
				&mut world
					.entity_mut(entity)
					.get_mut::<OwnCosmicEdit<T>>()
					.unwrap()
					.0,
			)
			.unwrap();
			let target = world.commands().spawn(cosmiced).id();
			world
				.commands()
				.entity(entity)
				.insert(CosmicSource(target))
				.remove::<OwnCosmicEdit<T>>();
		});
	}
}

#[derive(Resource, Clone)]
pub(super) struct IPAccept(
	pub(super) Arc<RwLock<Option<IpAddr>>>,
	pub(super) Option<IpAddr>,
);

impl Default for IPAccept {
	fn default() -> Self {
		Self(Arc::new(RwLock::new(None)), None)
	}
}
