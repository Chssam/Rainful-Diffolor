#![allow(unused)]
use bevy::prelude::*;
use std::fmt::Debug;
use std::time::Instant;

pub fn debug_comp<T: Debug + Component>(query: Query<(Entity, &T), Changed<T>>) {
	query.iter().for_each(|(ent, de)| {
		info!("De Change: {}", ent);
		dbg!(de);
	});
}

pub fn test_run_time(a_f: fn()) {
	let time_begin = Instant::now();
	a_f();
	info!("Time Per Round: {}", time_begin.elapsed().as_secs_f64());
}

pub fn debug_into_file(file_name: &str, contents: String) {
	let file_location = format!("assets/Debug/{}.txt", file_name);
	std::fs::write(file_location, contents).unwrap();
}

pub fn unhide_ent<T: Component>(mut enty: Query<&mut Visibility, With<T>>) {
	enty.iter_mut()
		.for_each(|mut visibility| *visibility = Visibility::Inherited);
}

pub fn hide_ent<T: Component>(mut enty: Query<&mut Visibility, With<T>>) {
	enty.iter_mut()
		.for_each(|mut visibility| *visibility = Visibility::Hidden);
}

// pub fn save_assets<T: Serialize>(saving_settings: &T, path: &str) {
//     let pretty_string = PrettyConfig::new()
//         .struct_names(true)
//         .separate_tuple_members(true)
//         .enumerate_arrays(true);
//     let prepare_well = match to_string_pretty(saving_settings, pretty_string) {
//         Ok(ready) => ready,
//         Err(err) => return warn!("FAILED MAKE STRING: {:#?}", err),
//     };
//     if let Err(err) = std::fs::write(path, prepare_well) {
//         warn!("FAILED FILE PATH: {:#?}", err);
//     }
// }
