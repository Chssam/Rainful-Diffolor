use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Debug, PartialEq, Eq, Clone, Reflect, Hash)]
pub(super) enum StaticBind {
	Fps,
	ScreenMode,
	ScreenShot,
}

impl StaticBind {
	pub(super) fn bind_default() -> InputMap<Self> {
		InputMap::new([
			(Self::Fps, KeyCode::F6),
			(Self::ScreenMode, KeyCode::F10),
			(Self::ScreenShot, KeyCode::F12),
		])
	}
}
