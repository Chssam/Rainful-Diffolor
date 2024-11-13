use bevy::{
	ecs::{
		component::{ComponentHooks, StorageType},
		system::IntoObserverSystem,
	},
	prelude::*,
};

use super::*;

pub(super) fn effect_run(
	mut cmd: Commands,
	mut on_up: EventReader<Pointer<Up>>,
	mut on_down: EventReader<Pointer<Down>>,
	mut on_over: EventReader<Pointer<Over>>,
	mut on_click: EventReader<Pointer<Click>>,
	query_ui: Query<
		(
			Entity,
			Has<SubMenuHold>,
			Has<MenuItem>,
			Has<DownEffect>,
			Has<ClickEffect>,
		),
		Without<MenuHold>,
	>,
) {
	on_up.read().for_each(|pointed| {
		let Ok((ent, is_item)) = query_ui.get(pointed.target()).map(|is_a| (is_a.0, is_a.2)) else {
			return;
		};
		is_item.then(|| cmd.trigger_targets(RunEffect, ent));
	});
	on_down.read().for_each(|pointed| {
		let Ok((ent, is_menu_a_down)) = query_ui.get(pointed.target()).map(|is_a| (is_a.0, is_a.3))
		else {
			return;
		};
		is_menu_a_down.then(|| cmd.trigger_targets(RunEffect, ent));
	});
	on_over.read().for_each(|pointed| {
		let Ok((ent, is_sub_menu)) = query_ui.get(pointed.target()).map(|is_a| (is_a.0, is_a.1))
		else {
			return;
		};
		is_sub_menu.then(|| cmd.trigger_targets(RunEffect, ent));
	});
	on_click.read().for_each(|pointed| {
		let Ok((ent, is_click)) = query_ui.get(pointed.target()).map(|is_a| (is_a.0, is_a.4))
		else {
			return;
		};
		is_click.then(|| cmd.trigger_targets(RunEffect, ent));
	});
}

pub(super) fn effect_run_menu(
	mut cmd: Commands,
	mut on_down: EventReader<Pointer<Down>>,
	mut on_over: EventReader<Pointer<Over>>,
	query_sub: Query<Entity, With<SubMenu>>,
	query_menu: Query<Entity, With<MenuHold>>,
	query_active: Query<Entity, With<MenuActive>>,
) {
	if !query_active.is_empty() {
		let overs = on_over.read().map(|pointed| pointed.target());
		if let Some(ent) = query_menu.iter_many(overs).fetch_next() {
			query_sub
				.iter()
				.for_each(|ent| cmd.entity(ent).despawn_recursive());
			query_active.iter().for_each(|ent_menu| {
				cmd.entity(ent_menu).remove::<MenuActive>();
			});
			cmd.trigger_targets(RunEffect, ent);
			cmd.entity(ent).insert(MenuActive);
		}
		return;
	}
	let downs = on_down.read().map(|pointed| pointed.target());
	if let Some(ent) = query_menu.iter_many(downs).fetch_next() {
		query_sub
			.iter()
			.for_each(|ent| cmd.entity(ent).despawn_recursive());
		query_active.iter().for_each(|ent_menu| {
			cmd.entity(ent_menu).remove::<MenuActive>();
		});
		cmd.trigger_targets(RunEffect, ent);
		cmd.entity(ent).insert(MenuActive);
	}
}

#[derive(Event)]
pub struct RunEffect;

#[derive(Component)]
pub struct DownEffect;

#[derive(Component)]
pub struct ClickEffect;

pub struct OwnObserve<E: Event, B: Bundle>(Option<Observer<E, B>>);

impl<E: Event, B: Bundle> OwnObserve<E, B> {
	pub fn new<M>(system: impl IntoObserverSystem<E, B, M>) -> Self {
		Self(Some(Observer::new(system)))
	}
}

impl<E: Event, B: Bundle> Component for OwnObserve<E, B> {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut binding = world.entity_mut(entity);
			let mut own_observe = binding.get_mut::<OwnObserve<E, B>>().unwrap();
			let Some(observe) = std::mem::take(&mut own_observe.0) else {
				world.commands().entity(entity).remove::<OwnObserve<E, B>>();
				return;
			};
			world
				.commands()
				.entity(entity)
				.insert(observe.with_entity(entity))
				.remove::<OwnObserve<E, B>>();
		});
	}
}
