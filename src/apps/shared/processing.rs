use std::marker::PhantomData;

use aery::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::apps::shared::prelude::*;

pub fn connect_relations<T: Event + Relation + Clone>(
	trigger: Trigger<ConnectRelations<T>>,
	mut cmd: Commands,
) {
	let ConnectRelations { ent1, ent2, .. } = trigger.event();
	cmd.entity(*ent2).set::<T>(*ent1);
}

// pub(super) fn revert_change(trigger: Trigger<ObjectBirNet>) {
// 	let ent_obj = trigger.entity();
// }

pub(super) fn convert_right<
	T: Component + Clone + Serialize + PartialEq + DeserializeOwned + Into<U>,
	U: Component + Clone,
>(
	mut query: Query<(&T, &mut U), Changed<T>>,
) {
	query.iter_mut().for_each(|(t, mut u)| {
		*u = t.clone().into();
	});
}

pub(super) struct VerifyActionPlugin<T: Event + Clone + Serialize>(PhantomData<T>);
impl<T: Event + Clone + Serialize + DeserializeOwned> Plugin for VerifyActionPlugin<T> {
	fn build(&self, app: &mut App) {
		use ChannelDirection::*;
		app.add_event::<T>();
		app.register_message::<T>(ClientToServer);
		app.register_message::<ToClientEntDataEvent<T>>(ServerToClient)
			.add_map_entities();
	}
}

impl<T: Event + Clone + Serialize> Default for VerifyActionPlugin<T> {
	fn default() -> Self {
		Self(default())
	}
}

pub(super) struct NetToLocalPlugin<
	T: Component + Clone + Serialize + PartialEq + DeserializeOwned + Into<U>,
	U: Component + Clone,
>(PhantomData<T>, PhantomData<U>);
impl<
		T: Component + Clone + Serialize + PartialEq + DeserializeOwned + Into<U>,
		U: Component + Clone,
	> Plugin for NetToLocalPlugin<T, U>
{
	fn build(&self, app: &mut App) {
		use client::ComponentSyncMode::*;
		use ChannelDirection::*;
		app.register_component::<T>(ServerToClient)
			.add_prediction(Full);
		app.add_systems(Update, convert_right::<T, U>);
	}
}

impl<
		T: Component + Clone + Serialize + PartialEq + DeserializeOwned + Into<U>,
		U: Component + Clone,
	> Default for NetToLocalPlugin<T, U>
{
	fn default() -> Self {
		Self(default(), default())
	}
}
