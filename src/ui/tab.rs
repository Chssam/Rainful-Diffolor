use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use i_cant_believe_its_not_bsn::WithChild;

use crate::trait_bevy::ImproveEntityCommands;

use super::*;

pub(super) fn select_tab(
	mut on_down: EventReader<Pointer<Down>>,
	mut cmd: Commands,
	query_tab: Query<(), With<Tab>>,
) {
	on_down.read().for_each(|pointed| {
		let target = pointed.target();
		if query_tab.contains(target) {
			cmd.entity(target).insert(ActiveView);
		}
	});
}

pub(super) fn not_container_yet(
	mut cmd: Commands,
	query: Query<(Entity, &Parent), With<AsContainer>>,
) {
	query.iter().for_each(|(ent, parent)| {
		cmd.entity(parent.get()).insert(PointToTabContainer(ent));
		cmd.entity(ent).remove::<AsContainer>();
	});
}

#[derive(Component)]
pub struct Holding(pub Entity);

#[derive(Component)]
pub struct Tab;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NotTabYet(Option<String>);

impl NotTabYet {
	pub fn new(name: &str) -> Self {
		Self(Some(name.to_owned()))
	}
}

pub(super) fn not_tab_yet(
	mut cmd: Commands,
	mut query_container: Query<(Entity, &mut Style, &mut NotTabYet, &Parent)>,
	query_tab_con: Query<&PointToTabContainer>,
) {
	query_container
		.iter_mut()
		.for_each(|(ent, mut style, mut not_tab_yet, parent)| {
			style.display = Display::None;
			let parent = parent.get();
			let Ok(point_to) = query_tab_con.get(parent) else {
				return;
			};
			let name = not_tab_yet.0.take().unwrap();
			cmd.entity(point_to.0).with_child((
				EffectUIBundle::text().button(),
				Holding(ent),
				Tab,
				WithChild(TextBuild::single(&name)),
			));
			cmd.entity(ent).remove::<NotTabYet>();
		});
}

#[derive(Component)]
pub struct PointToTabContainer(pub Entity);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AsContainer;

pub struct ActiveView;

impl Component for ActiveView {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_insert(|mut world, entity, _component_id| {
			let ent_mut = world.entity(entity);
			let parent = ent_mut.get::<Parent>().unwrap().get();
			let hold = ent_mut.get::<Holding>().unwrap().0;
			let mut ent_mut = world.entity_mut(hold);
			let mut style = ent_mut.get_mut::<Style>().unwrap();
			style.display = Display::DEFAULT;
			let children = world
				.entity(parent)
				.get::<Children>()
				.unwrap()
				.iter()
				.cloned()
				.collect::<Vec<_>>();
			children.into_iter().for_each(|ent| {
				if ent == entity {
					return;
				}
				let hold = world.entity(ent).get::<Holding>().unwrap().0;
				let mut ent_mut = world.entity_mut(hold);
				let mut style = ent_mut.get_mut::<Style>().unwrap();
				style.display = Display::None;
				world.commands().entity(ent).remove::<ActiveView>();
			});
		});
	}
}
