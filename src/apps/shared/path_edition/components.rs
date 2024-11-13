use super::*;
use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	sprite::Mesh2dHandle,
};
use lightyear::prelude::ClientId;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Component, Reflect, Default, Clone, Copy, EnumIter, PartialEq)]
pub enum RenderPathAs {
	Image,
	#[default]
	Path,
}

#[derive(Component, Reflect, Clone, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub struct ObjectPoint;

/// Default LineTo without [PathType] attribute
#[derive(Event, Relation, Clone)]
#[aery(Counted)]
pub struct PointToPoint;

#[derive(Component, Reflect, Clone, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Component)]
pub enum PointType {
	#[default]
	LineTo,
	QuadraticBezier {
		to: Vec2,
	},
	/// One of these unwanted 1 / 2
	CubricBezier {
		ctrl1: Vec2,
		ctrl2: Vec2,
		to: Vec2,
	},
	Arc {
		radii: Vec2,
		sweep_angle: f32,
		x_rotation: f32,
	},
}

#[derive(Bundle, Default)]
pub struct PointBundle {
	object: ObjectWorld,
	marker: ObjectPoint,
	position: ObjectPosition,
	obj_z: ObjectZLayer,
	point_type: PointType,
}

impl PointBundle {
	pub fn new(position: Vec2) -> Self {
		Self {
			position: ObjectPosition(position),
			..default()
		}
	}
}

#[derive(Bundle, Default)]
pub struct RdioPathBundle {
	object: NetObjectBundle,
	mark: ObjectPath,
	z_pos: ObjectZLayer,
	stroke: StrokeNet,
	fill: FillNet,
	close: PathClose,
}

#[derive(Bundle, Default)]
pub struct PathAsImageBundle {
	pub sprite: Sprite,
	pub texture: Handle<Image>,
}

#[derive(Bundle)]
pub struct PathAsSvgBundle {
	pub path: Path,
	pub mesh: Mesh2dHandle,
	pub material: Handle<ColorMaterial>,
	pub stroke: Stroke,
	pub fill: Fill,
}

impl RdioPathBundle {
	pub fn new(name: &str, owner: ClientId) -> Self {
		Self {
			object: NetObjectBundle::new(name, owner),
			..default()
		}
	}
}

impl PathAsSvgBundle {
	pub fn new(stroke: Stroke, fill: Fill) -> Self {
		Self {
			path: default(),
			mesh: default(),
			material: Handle::weak_from_u128(0x7CC6_61A1_0CD6_C147_129A_2C01_882D_9580),
			stroke,
			fill,
		}
	}
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ObjectPath;

impl Component for ObjectPath {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let ref_ent = world.entity(entity);
			if ref_ent.contains::<Predicted>() {
				return;
			}
			world.commands().entity(entity).insert((
				RenderPathAs::default(),
				Aabb::default(),
				SpatialBundle::default(),
			));
		});
	}
}

#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PathClose(pub bool);

#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrokeNet {
	pub options: StrokeOptions,
	pub color: Color,
}

impl Default for StrokeNet {
	fn default() -> Self {
		Self {
			options: StrokeOptions::DEFAULT.with_line_width(5.0),
			color: Color::BLACK,
		}
	}
}

impl From<StrokeNet> for Stroke {
	fn from(value: StrokeNet) -> Self {
		Stroke {
			options: value.options,
			color: value.color,
		}
	}
}

#[derive(Component, Clone, Serialize, Deserialize, PartialEq)]
pub struct FillNet {
	pub options: FillOptions,
	pub color: Color,
}

impl Default for FillNet {
	fn default() -> Self {
		Self {
			options: FillOptions::non_zero(),
			color: Color::WHITE,
		}
	}
}

impl From<FillNet> for Fill {
	fn from(value: FillNet) -> Self {
		Fill {
			options: value.options,
			color: value.color,
		}
	}
}

// #[derive(Component, Debug, Deserialize, Serialize, Clone)]
// pub enum SvgType {
//     Path(Vec<PathData>),
//     Rectangle(Vec2),
//     RoundTangle(Vec2, Vec2),
//     Polygon(Vec<Vec2>),
//     Polyline(Vec<Vec2>),
//     Text(String, f32),
//     Circle(f32),
//     Ellipse(Vec2),
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub enum PathData {
//     Moveto(Vec2),
//     Lineto(Vec2),
//     HorizontalLineto,
//     VerticalLineto,
//     Curveto,
//     SmoothCurveto,
//     QuadraticBézierCurve,
//     SmoothQuadraticBézierCurveto,
//     EllipticalArc,
//     Closepath,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct DashStyle(pub Vec<f32>);

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct Offset(pub f32);
// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub enum SvgPaint {
//     NoPaint,
//     FlatColor(Color),
//     LinearGradient(Vec<(Color, Offset)>),
//     RadialGradient(Vec<(Color, Offset)>),
//     MeshGradient,
//     Pattern,
//     Swatch,
//     UnsetPaint,
// }

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
// pub enum PathEditType {
// 	NewSvg {
// 		path_type: PathType,
// 		stroke: Option<StrokeOptions>,
// 		fill: Option<Color>,
// 		display_only: bool,
// 	},
// 	NewPoint(PathType),
// 	MovePoint(Vec2),
// 	DeletePoint,
// }

// #[derive(Component, Debug, Deserialize, Serialize, Clone)]
// pub enum SvgType {
//     Path(Vec<PathData>),
//     Rectangle(Vec2),
//     RoundTangle(Vec2, Vec2),
//     Polygon(Vec<Vec2>),
//     Polyline(Vec<Vec2>),
//     Text(String, f32),
//     Circle(f32),
//     Ellipse(Vec2),
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub enum PathData {
//     Moveto(Vec2),
//     Lineto(Vec2),
//     HorizontalLineto,
//     VerticalLineto,
//     Curveto,
//     SmoothCurveto,
//     QuadraticBézierCurve,
//     SmoothQuadraticBézierCurveto,
//     EllipticalArc,
//     Closepath,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct DashStyle(pub Vec<f32>);

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct Offset(pub f32);
// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub enum SvgPaint {
//     NoPaint,
//     FlatColor(Color),
//     LinearGradient(Vec<(Color, Offset)>),
//     RadialGradient(Vec<(Color, Offset)>),
//     MeshGradient,
//     Pattern,
//     Swatch,
//     UnsetPaint,
// }

// impl SvgType {
//     pub fn size(&self, stroke_width: StrokeWidth) -> Vec2 {
//         let mut cal_uvec2 = Vec2::ZERO;
//         cal_uvec2 += Vec2::splat(stroke_width.0 / 2.0);
//         match self {
//             SvgType::Path(_) => todo!(),
//             SvgType::Rectangle(vec2) => cal_uvec2 += vec2.clone(),
//             SvgType::RoundTangle(vec2, _) => cal_uvec2 += vec2.clone(),
//             SvgType::Polygon(_) => todo!(),
//             SvgType::Polyline(_) => todo!(),
//             SvgType::Text(_, _) => todo!(),
//             SvgType::Circle(radials) => cal_uvec2 += Vec2::ONE * radials.clone(),
//             SvgType::Ellipse(vec2) => cal_uvec2 += vec2.clone(),
//         }
//         cal_uvec2
//     }
// }

// fn export_svg(svg_type: SvgType) {
//     // let mut scripter = Svg {
//     // name: "SVG FILE".to_string(),
//     // size: Vec2::new(500.0, 500.0),
//     // view_box: ViewBox,
//     // paths: Vec<PathDescriptor>,
//     // mesh: Handle<Mesh>,
//     // };
//     let data = Data::new()
//         .move_to((10, 10))
//         .line_by((0, 50))
//         .line_by((50, 0))
//         .line_by((0, -50))
//         .close();
//     let path = SvgPath::new()
//         .set("fill", "none")
//         .set("stroke", "black")
//         .set("stroke-width", 3)
//         .set("d", data);
//     // match svg_type {
//     //     SvgType::Path(_) => todo!(),
//     //     SvgType::Rectangle(_) => todo!(),
//     //     SvgType::Polygon(_) => todo!(),
//     //     SvgType::Polyline(_) => todo!(),
//     //     SvgType::Text(_, _) => todo!(),
//     //     SvgType::Circle(_) => todo!(),
//     //     SvgType::Ellipse(_) => todo!(),
//     // }

//     let document = Document::new().add(path);

//     svg::save(&format!("{}some.svg", TEMP_SPACE), &document).unwrap();
// }

// impl From<StrokeNet> for zeno::Stroke {
// 	fn from(value: StrokeNet) -> Self {
// 		let Stroke {
// 			options:
// 				StrokeOptions {
// 					start_cap,
// 					end_cap,
// 					line_join,
// 					line_width,
// 					variable_line_width,
// 					miter_limit,
// 					tolerance,
// 					..
// 				},
// 			color,
// 		} = value;
// 		Self {
// 			width: line_width,
// 			join: (),
// 			miter_limit: (),
// 			start_cap: (),
// 			end_cap: (),
// 			dashes: (),
// 			offset: (),
// 			scale: (),
// 		}
// 	}
// }
