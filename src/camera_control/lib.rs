use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

pub const CAMERA_VIEW_RANGE: f32 = 10000.0;
pub(super) const TILE_DISTANCE: f32 = -CAMERA_VIEW_RANGE + 1.0;
pub const MAX_VALID_RANGE: f32 = 1e+8;

// #[derive(Component)]
// pub struct CamFollowNodeSize;

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Default)]
pub struct IsUnFocusOnUI(pub(super) bool);

impl IsUnFocusOnUI {
	pub fn get(&self) -> bool {
		self.0
	}
}

#[derive(Component)]
pub(super) struct BackGroundTile;

#[derive(SystemParam, Deref)]
pub struct GlobalScreen2D<'w, 's>(
	Query<
		'w,
		's,
		(
			&'static GlobalTransform,
			&'static Camera,
			&'static ActionState<GlobalCamAction>,
		),
		(With<Camera2d>, With<MainCamera>),
	>,
	#[deref] Query<'w, 's, &'static Window, With<PrimaryWindow>>,
	EventReader<'w, 's, CursorMoved>,
);

impl<'w, 's> GlobalScreen2D<'w, 's> {
	pub fn cursor_ui(&mut self) -> Option<Vec2> {
		let window = self.1.single();
		window
			.cursor_position()
			.or(self.2.read().last().map(|cur_moved| cur_moved.position))
	}
	pub fn cursor_world(&mut self) -> Option<Vec2> {
		let op_cursor = self.cursor_ui();
		let (transform, cam, _) = self.0.single();
		op_cursor.and_then(|cur_pos| cam.viewport_to_world_2d(transform, cur_pos))
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Reflect, Hash)]
pub(super) enum GlobalCamAction {
	ZoomIn,
	ZoomOut,
	Scroll,
	Control,
	PanAxisX,
	FreeMove,
	Pull,
}

impl Actionlike for GlobalCamAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			GlobalCamAction::Scroll => InputControlKind::Axis,
			GlobalCamAction::Pull => InputControlKind::DualAxis,
			_ => InputControlKind::Button,
		}
	}
}

impl GlobalCamAction {
	pub(super) fn bind_default() -> InputMap<Self> {
		InputMap::new([
			(Self::ZoomIn, KeyCode::Equal),
			(Self::ZoomOut, KeyCode::Minus),
			(Self::Control, KeyCode::ControlLeft),
			(Self::PanAxisX, KeyCode::AltLeft),
			(Self::FreeMove, KeyCode::Space),
		])
		.with(Self::FreeMove, MouseButton::Middle)
		.with_axis(Self::Scroll, MouseScrollAxis::Y)
		.with_dual_axis(Self::Pull, MouseMove::default())
	}
}

pub(super) trait TiledBgColor {
	fn bg_color(&self) -> Vec<u8>;
}

impl TiledBgColor for ClearColor {
	/// Guarateen total (2x2) * 4 = 16
	fn bg_color(&self) -> Vec<u8> {
		let color_1 = self.lighter(0.05).to_srgba().to_u8_array();
		let color_2 = self.to_srgba().to_u8_array();
		[color_1, color_2, color_2, color_1].concat()
	}
}
