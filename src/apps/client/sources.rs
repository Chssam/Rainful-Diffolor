use crate::{
	tool_tip::lib::{ToolName, ToolTip},
	trait_bevy::ToolPath,
};

use super::*;
use bevy::{
	color::palettes::css::{BLACK, WHITE},
	ecs::world::CommandQueue,
	tasks::Task,
};
use client::Replicate;
use image::*;
use imageproc::drawing::draw_line_segment_mut;
use leafwing_input_manager::prelude::*;
use strum::EnumIter;

#[derive(Reflect, Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(RdioClientState = RdioClientState::Online)]
pub enum DropPathMode {
	#[default]
	AsObject,
	SaveLocation,
}

#[derive(Component, Default, Reflect)]
#[reflect(@ToolName("Visual Grid"), @ToolTip("Display Grid on main selected object\n(High performance impact)"))]
pub(super) struct VisualGrid(pub bool);

#[derive(Component)]
pub(super) struct FileReaded(pub Task<CommandQueue>);

#[derive(Component)]
pub struct EditorInfoComp;

#[derive(Component)]
pub struct ObjInfoController;

#[derive(Component, Reflect, Clone, Copy, EnumIter)]
pub enum ColorPanelChanger {
	Red,
	Green,
	Blue,
	Alpha,
	Hue,
	Saturation,
	Lightness,
	SatLight,
}

#[derive(Component)]
pub struct RGBAhsl {
	pub red: Handle<Image>,
	pub blue: Handle<Image>,
	pub green: Handle<Image>,
	pub alpha: Handle<Image>,
	pub hue: Handle<Image>,
	pub saturation: Handle<Image>,
	pub lightness: Handle<Image>,
	pub sat_light: Handle<Image>,
}

#[derive(Component, Default)]
pub struct MainUser;

pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

#[derive(Bundle, Default)]
pub struct MainUserBundle {
	main_user: MainUser,
	replicate: Replicate,
	grid: VisualGrid,
	last_draw: LastDrawPos,
	previous_draw_pos: PreviousDrawPos,
	action_tool: InputManagerBundle<EditorTools>,
	action_one: InputManagerBundle<ToolsStandAlone>,
	action_normal: InputManagerBundle<SettingsAction>,
}

impl MainUserBundle {
	pub fn new() -> Self {
		Self {
			action_tool: InputManagerBundle::with_map(EditorTools::bind_default()),
			action_one: InputManagerBundle::with_map(ToolsStandAlone::bind_default()),
			action_normal: InputManagerBundle::with_map(SettingsAction::bind_default()),
			replicate: Replicate {
				group: REPLICATION_GROUP,
				..default()
			},
			..default()
		}
	}
}

#[derive(
	Actionlike, States, Debug, EnumIter, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect,
)]
pub enum EditorTools {
	#[default]
	#[reflect(@ToolTip("Pick Screen Objects"), @ToolPath("tool-pointer.png"))]
	Pick,
	// Pen,
	// Paint,
	#[reflect(@ToolTip("Hard edge painting using brush"), @ToolPath("gimp-tool-pencil.png"))]
	Pencel,
	#[reflect(@ToolTip("Set color from image pixels"), @ToolPath("gimp-tool-color-picker.png"))]
	ColorPick,
	#[reflect(@ToolTip("Create temporary effect"), @ToolPath("gimp-tool-paintbrush.png"))]
	Marker,
	#[reflect(@ToolTip("Scale object"), @ToolPath("gimp-tool-scale.png"))]
	Scale,
	#[reflect(@ToolTip("Resize image without moving pixel"), @ToolPath("gimp-tool-scale.png"))]
	Resize,
	#[reflect(@ToolTip("Create and edit Text"), @ToolPath("gimp-tool-text.png"))]
	Text,
	#[reflect(@ToolTip("Create and edit Path"), @ToolPath("gimp-tool-path.png"))]
	Path,
	// Rectangle,
	// Ellipse,
	// Fill,

	// Crop,
}

impl EditorTools {
	pub(super) fn bind_default() -> InputMap<Self> {
		InputMap::new([(Self::Pencel, KeyCode::KeyP)])
	}
}

#[derive(Actionlike, Component, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolsStandAlone {
	New,
	NewImage,
	NewGroup,
	Import,
	OpenRecent,
	Undo,
	Redo,
	Cut,
	Copy,
	Paste,
	Open,
	Save,
	SaveAs,
	Preference,
	SelectAll,
	ExportImage,
	ExportCanvas,
	ExportSvgRelative,
	ExportSvgAbsolute,
	ColorSwap,
	SpawnColorNode,
	FlipHorizontal,
	FlipVertical,
	Rotate,
	Hide,
}

impl ToolsStandAlone {
	pub(super) fn bind_default() -> InputMap<Self> {
		InputMap::new([(Self::Hide, KeyCode::KeyH)])
			.with(
				ToolsStandAlone::Save,
				ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyS]),
			)
			.with(
				Self::Copy,
				ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyC]),
			)
			.with(
				Self::Paste,
				ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyV]),
			)
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Reflect, Hash, Deserialize, Serialize)]
pub enum SettingsAction {
	Primary,
	Secondary,
	Tertiary,
	ScrollWheel,
	Movement,
	Shift,
	Alt,
	Enter,
	Escape,
	Delete,
	Move,
	Control,
	Increase,
	Decrease,
	RoundPos,
}

impl Actionlike for SettingsAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			Self::ScrollWheel => InputControlKind::Axis,
			Self::Movement => InputControlKind::DualAxis,
			_ => InputControlKind::Button,
		}
	}
}

impl SettingsAction {
	pub fn bind_default() -> InputMap<Self> {
		InputMap::new([
			(Self::Shift, KeyCode::ShiftLeft),
			(Self::Alt, KeyCode::AltLeft),
			(Self::Enter, KeyCode::Enter),
			(Self::Escape, KeyCode::Escape),
			(Self::Delete, KeyCode::Delete),
			(Self::Move, KeyCode::KeyG),
			(Self::RoundPos, KeyCode::KeyB),
		])
		.with_one_to_many(Self::Increase, [KeyCode::ArrowUp, KeyCode::ArrowRight])
		.with_one_to_many(Self::Decrease, [KeyCode::ArrowDown, KeyCode::ArrowLeft])
		.with_one_to_many(Self::Control, [KeyCode::ControlLeft, KeyCode::ControlRight])
		.with_multiple([
			(Self::Primary, MouseButton::Left),
			(Self::Secondary, MouseButton::Right),
			(Self::Tertiary, MouseButton::Middle),
		])
		.with_axis(Self::ScrollWheel, MouseScrollAxis::Y)
		.with_dual_axis(Self::Movement, MouseMove::default())
	}
}

#[derive(Component)]
pub(super) struct BrushCollection;

// MARK: Unused
#[derive(Bundle)]
pub(super) struct BrushCollectionBundle {
	brush: BrushRef,
	id: BrushCollection,
}

fn white_black(v: &mut [[u8; 4]], value: impl Into<i16>) {
	let converted: i16 = value.into();
	for (index, colored) in [
		(converted - 1, Srgba::BLACK),
		(converted + 1, Srgba::BLACK),
		(converted - 2, Srgba::WHITE),
		(converted + 2, Srgba::WHITE),
	] {
		if let Some(inner) = v.get_mut(index as usize) {
			*inner = colored.to_u8_array();
		}
	}
}

impl PaintInk {
	pub(super) fn red(&self) -> Vec<u8> {
		let [red, green, blue, _] = self.0.to_u8_array();
		let mut v: Vec<[u8; 4]> = (0..=255).map(|r| [r, green, blue, 255]).collect();
		white_black(&mut v, red);
		v.concat()
	}
	pub(super) fn green(&self) -> Vec<u8> {
		let [red, green, blue, _] = self.0.to_u8_array();
		let mut v: Vec<[u8; 4]> = (0..=255).map(|g| [red, g, blue, 255]).collect();
		white_black(&mut v, green);
		v.concat()
	}
	pub(super) fn blue(&self) -> Vec<u8> {
		let [red, green, blue, _] = self.0.to_u8_array();
		let mut v: Vec<[u8; 4]> = (0..=255).map(|b| [red, green, b, 255]).collect();
		white_black(&mut v, blue);
		v.concat()
	}
	pub(super) fn alpha(&self) -> Vec<u8> {
		let [red, green, blue, alpha] = self.0.to_u8_array();
		let mut v: Vec<[u8; 4]> = (0..=255).map(|a| [red, green, blue, a]).collect();
		white_black(&mut v, alpha);
		v.concat()
	}
	pub(super) fn hue(&self) -> Vec<u8> {
		let Hsla {
			hue,
			saturation,
			lightness,
			..
		} = self.0.into();
		let mut v: Vec<[u8; 4]> = (0..=360)
			.map(|h| {
				Color::hsl(h as f32, saturation, lightness)
					.to_srgba()
					.to_u8_array()
			})
			.collect();
		white_black(&mut v, hue as i16);
		v.concat()
	}
	pub(super) fn saturation(&self) -> Vec<u8> {
		let Hsla {
			hue,
			saturation,
			lightness,
			..
		} = self.0.into();
		let mut v: Vec<[u8; 4]> = (0..=100)
			.map(|s| {
				Color::hsl(hue, s as f32 / 100.0, lightness)
					.to_srgba()
					.to_u8_array()
			})
			.collect();
		white_black(&mut v, (saturation * 100.0) as i16);
		v.concat()
	}
	pub(super) fn lightness(&self) -> Vec<u8> {
		let Hsla {
			hue,
			saturation,
			lightness,
			..
		} = self.0.into();
		let mut v: Vec<[u8; 4]> = (0..=100)
			.map(|l| {
				Color::hsl(hue, saturation, l as f32 / 100.0)
					.to_srgba()
					.to_u8_array()
			})
			.collect();
		white_black(&mut v, (lightness * 100.0) as i16);
		v.concat()
	}
	pub(super) fn sat_light(&self) -> Vec<u8> {
		let Hsla {
			hue,
			mut saturation,
			mut lightness,
			..
		} = self.0.into();
		saturation *= 100.0;
		lightness *= 100.0;
		let mut data_color = Vec::new();
		for light in 0..=100 {
			for sat in 0..=100 {
				let data = Color::hsl(hue, sat as f32 / 100.0, (100 - light) as f32 / 100.0)
					.to_srgba()
					.to_u8_array();
				data_color.push(data);
			}
		}
		let mut temp_rgba = RgbaImage::from_vec(101, 101, data_color.concat()).unwrap();
		for (line, is_x, colored) in [
			(saturation - 1.0, true, BLACK),
			(saturation + 1.0, true, BLACK),
			(lightness - 1.0, false, BLACK),
			(lightness + 1.0, false, BLACK),
			(saturation - 2.0, true, WHITE),
			(saturation + 2.0, true, WHITE),
			(lightness - 2.0, false, WHITE),
			(lightness + 2.0, false, WHITE),
		] {
			let (start, end) = if is_x {
				((line, 0.0), (line, 101.0))
			} else {
				((0.0, 101.0 - line), (101.0, 101.0 - line))
			};
			draw_line_segment_mut(&mut temp_rgba, start, end, Rgba(colored.to_u8_array()));
		}
		temp_rgba.into_vec()
	}
}

#[derive(Component, Deref, DerefMut, Default, PartialEq)]
pub(super) struct LastDrawPos(pub CursorPos);

#[derive(Component, Deref, DerefMut, Default, PartialEq)]
pub(super) struct PreviousDrawPos(pub Option<CursorPos>);

#[derive(Component)]
pub(super) struct PendingImage;

#[derive(Component)]
pub(super) struct RequestedPoint;

#[derive(Component)]
pub struct BrushChoice;

#[derive(Component)]
pub struct DisplayBrush;
