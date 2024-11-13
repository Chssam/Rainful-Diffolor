// use std::path::{Path, PathBuf};

// use crate::editor::{prelude::TEMP_SPACE, sources::RDIO_IN_SAVE_DISK};

// use super::prelude::*;
// use bevy::prelude::*;
// use serde::{Deserialize, Serialize};

// /// Path including File SharingName
// #[derive(Resource, Debug, Deserialize, Serialize, Clone, Deref, DerefMut)]
// pub struct Rdio(PathBuf);

// #[derive(Bundle, Debug, Deserialize, Serialize, Clone)]
// pub struct MainRdioBundle {
//     pub path: Rdio2D,
//     pub canvas: Canvas2D,
// }

// impl Default for Rdio {
// 	fn default() -> Self {
// 		let file_path = Path::new("assets")
// 			.join("Rdio Save")
// 			.join("Auto")
// 			.join("Rdio Canvas");
// 		Self(PathBuf::new().with_file_name(file_path))
// 	}
// }

// #[derive(Resource, Debug, Deserialize, Serialize, Clone, Deref, DerefMut)]
// pub struct Canvas2D(pub UVec2);

// impl Default for Canvas2D {
// 	fn default() -> Self {
// 		Self(UVec2::splat(500))
// 	}
// }

// #[derive(Component)]
// pub struct NewPathDrop {
//     pub handle_image: Handle<Image>,
//     pub name: String
// }

// #[derive(Debug, Deserialize, Serialize, TypeUuid, Clone)]
// #[uuid = "02cadc66-aa4c-4243-2640-b018b69b5052"]
// pub struct SavedRdio {
//     pub main: MainRdioBundle,
//     // pub svg: Vec<LayerGroup<SvgType>>,
//     // pub png: Vec<LayerGroup<ImageAlphaness>>,
//     // pub z_lay: Vec<LayerGroup<SaveType>>,
// }

// #[derive(Component, Debug, Deserialize, Serialize, Clone)]
// pub enum SaveType {
//     Image(RdioImageSaveAbleBundle),
//     Svg(RdioSvgSaveAbleBundle),
// }

// pub fn eqio_extracter() {
//     let a_saved = SavedRdio::default();
//     for a_very in a_saved.z_lay.iter() {
//         let mut still_valid: Vec<LayerGroup<SaveType>> = vec![];
//         still_valid.push(a_very.clone());
//         match a_very {
//             LayerGroup::Single(something) => todo!(),
//             LayerGroup::Group(more_box) => {
//                 // let some_else = more_box.clone().iter().collect::<LayerGroup<SaveType>>();
//                 for koko in more_box {
//                     koko
//                 }
//             }
//         };
//     }
// }

// impl Rdio {
//     pub fn save_name(&self) -> String {
//         format!("{}.rd", self.0)
//     }
// }

// fn open_eqio() {}

// fn save_as_eqio(path: String, rd: SavedRdio) {
//     let ref_path = &path;
// }

// fn new_eqio(world: &mut World) {
//     let main_canvas_bundle = MainCanvasBundle::new(TEMP_SPACE.to_string());
//     let path_name = format!("{TEMP_SPACE}{}.rd", main_canvas_bundle.main.0);
//     world.spawn(main_canvas_bundle);
//     let saving_eqio = SavedRdio::default();
//     save_assets(&saving_eqio, &path_name);
// }

// fn resize_canvas(canvas: Res<MainCanvas>) {
//     // canvas.0.resize(500, 500, FilterType::Lanczos3);
//     // RgbaImage::new(width, height)
// }
