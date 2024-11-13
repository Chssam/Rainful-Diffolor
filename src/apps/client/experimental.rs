#![allow(unused)]

use leafwing_input_manager::prelude::*;
use std::any::TypeId;
use std::{marker::PhantomData, ops::RangeInclusive};

use bevy::{prelude::*, reflect::ReflectMut};
// use bevy::reflect::{Reflect, TypeInfo, Typed};

use super::{MainUser, SettingsAction};

#[derive(Component)]
pub struct SliderType<T: Component + Reflect>(PhantomData<T>);

#[derive(Component, Reflect)]
pub struct OMEGA(
	// #[reflect(@0.0..=1.0_f32)]
	f32,
);

fn slider<T: Component + Reflect>(
	query_panel: Query<&Interaction, With<SliderType<T>>>,
	mut query_user: Query<(&ActionState<SettingsAction>, &mut T), With<MainUser>>,
) {
	let (action, mut slider_v) = query_user.single_mut();
	query_panel.iter().for_each(|inter| {
		if inter == &Interaction::None {
			return;
		}

		let is_pressing = inter == &Interaction::Pressed;
		let point_movement = action.axis_pair(&SettingsAction::Movement);
		let increase = action.just_pressed(&SettingsAction::Increase) as i32;
		let decrease = action.just_pressed(&SettingsAction::Decrease) as i32;
		let scroll = action.value(&SettingsAction::ScrollWheel);
		let put = point_movement.x * is_pressing as i32 as f32 + scroll + increase as f32
			- decrease as f32;
		if put == 0.0 {
			return;
		}

		// match slider_v.reflect_mut() {
		// 	// ReflectMut::Struct(v) => {
		// 	//     for n in 0..v.field_len() {
		// 	//         let a = v.field_at_mut(n).unwrap();
		// 	//         let b = a.downcast_mut::<>();
		// 	//     }
		// 	// },
		// 	// ReflectMut::TupleStruct(v) => v.get_field_mut(index),
		// 	ReflectMut::Tuple(v) => {
		// 		// let a = v.field_mut(0).unwrap();
		// 		let TypeInfo::Tuple(a) = Test1::type_info();
		// 		let b = a.field_at(0).unwrap();
		// 		let c = b.get_attribute::<RangeInclusive<f32>>();
		// 		// let a = TypeId::of::<Test1>;
		// 		// let TypeInfo::Struct(type_info) = SliderType::t() else {
		// 		// 	panic!("expected struct");
		// 		// };
		// 	},
		// 	// ReflectMut::List(v) => v,
		// 	// ReflectMut::Array(v) => v,
		// 	// ReflectMut::Map(v) => v,
		// 	// ReflectMut::Enum(v) => v.reflect_short_type_path(),
		// 	// ReflectMut::Value(v) => v,
		// 	_ => panic!("Unsupported"),
		// }

		// let ori = match color_panel {
		// 	ColorPanelChanger::Red => &mut paint.0.red,
		// 	ColorPanelChanger::Green => &mut paint.0.green,
		// 	ColorPanelChanger::Blue => &mut paint.0.blue,
		// 	ColorPanelChanger::Alpha => &mut paint.0.alpha,
		// };
		// *ori = (*ori + put / 255.0).clamp(0.0, 1.0);
	});
}
