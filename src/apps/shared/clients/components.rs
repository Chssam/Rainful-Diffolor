use std::num::{NonZero, NonZeroU8};

use crate::{
	apps::shared::prelude::*, camera_control::lib::CAMERA_VIEW_RANGE, tool_tip::lib::ToolTip,
	trait_bevy::ToolPath,
};

use bevy::{
	color::palettes::css::{BLACK, WHITE},
	ecs::entity::{EntityHashSet, MapEntities},
	prelude::*,
};
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

pub const BEGIN_OBJ_Z_INDEX: f32 = -CAMERA_VIEW_RANGE + 1000.0;

// #[derive(Component, Default)]
// pub struct ProcessDraw(pub Vec<Task<CommandQueue>>);

#[derive(Bundle, Default)]
pub struct ClientUserBundle {
	position: CursorPos,
	name: SharingName,
	users: UserId,
	color: PaintInk,
	draw_width: BrushScale,
	pin_point: BeginSelectPoint,
	resize_point: ResizePinPoint,
	scale_type: ScaleAction,
	resize_kind: ResizeKind,
	scale_pos: ScalePosition,
	spacing: DrawingSpacing,
	ref_draw: BrushRef,
	select: SelectedObject,
	action: InputManagerBundle<VerifyAction>,
	hard_edge: HardEdgeDraw,
	draw_type: DrawType,
	selection: Selection,
	blur_scale: BlurScale,
}

impl ClientUserBundle {
	pub fn new(id: ClientId, name: String) -> Self {
		Self {
			name: SharingName(name),
			users: UserId(id),
			..default()
		}
	}
}

#[derive(Bundle, Default)]
pub struct LocalUserBundle {
	piled_draw: DrawPiled,
	action: InputManagerBundle<ClientAction>,
}

// #[derive(Clone, Serialize, Deserialize)]
// pub enum TextKind {
// 	Insert(char),
// 	Remove(DeleteType),
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub enum DeleteType {
// 	Backspace,
// 	Delete,
// }

#[derive(Component, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalePosition {
	#[default]
	Top,
	Bottom,
	Left,
	Right,
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,
	Middle,
}

#[derive(Component, Default, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ScaleAction(pub Option<ScaleKind>);

#[derive(Component, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ScaleKind {
	Pixel(Vec2),
	Percent(Vec2),
}

// pub struct UserKey();

// #[derive(Component)]
// pub struct ObjectOwner(UserKey);

// #[derive(Bundle)]
// pub struct ObjBundle {
//     owner: ObjectOwner,
//     default_perm: DefaultUserAccess,
//     object_perm: ObjectPermissionToUser,
// }

// /// If [ObjectPermissionToUser] does not represent user, default access will be used.
// #[derive(Component)]
// pub struct DefaultUserAccess(HashSet<AccessOfObject>);

// #[derive(Component)]
// pub struct ObjectPermissionToUser(HashMap<UserKey, HashSet<AccessOfObject>>);

// /// Not exist will not grant access
// pub enum AccessOfObject {
//     View,
//     Edit,
// }
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub struct ResizeInfo {
//     pub size: UVec2,
//     pub kind: ResizeKind,
// }

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum ResizeKind {
	#[default]
	Scale,
	Resize,
}

#[derive(Component, Clone, Deserialize, Serialize, PartialEq)]
pub struct UserId(pub ClientId);

impl Default for UserId {
	fn default() -> Self {
		Self(ClientId::Local(0))
	}
}

#[derive(Component, Clone, Copy, Default, Deserialize, Serialize, Deref, DerefMut, PartialEq)]
pub struct CursorPos(pub Vec2);

impl CursorPos {
	pub fn pixel_to_img(&self, img_pos: Vec2) -> Vec2 {
		((self.xy() - img_pos) * Vec2::new(1.0, -1.0)).floor()
	}
	pub fn pixel_pos_central(&self, img_pos: Vec2) -> Vec2 {
		self.pixel_to_img(img_pos) + Vec2::splat(0.5)
	}
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Deref, DerefMut)]
pub struct DrawingSpacing(NonZeroU8);

impl Default for DrawingSpacing {
	fn default() -> Self {
		Self(NonZero::new(1).unwrap())
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SimpleObjectEvent {
	Delete,
	Hide,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CursorFromTo {
	pub from: CursorPos,
	pub to: CursorPos,
}

/// Begin | End
#[derive(Component, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Selection(pub Option<Vec2>, pub Option<Vec2>);

/// New Client didn't receive the correct entity from old one
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SelectedObject {
	pub single: Option<Entity>,
	pub group: EntityHashSet,
}

impl MapEntities for SelectedObject {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		self.single = self.single.map(|ent_obj| entity_mapper.map_entity(ent_obj));
		self.group = self
			.group
			.iter()
			.fold(EntityHashSet::default(), |mut set, ent| {
				let local_ent = entity_mapper.map_entity(*ent);
				set.insert(local_ent);
				set
			});
	}
}

impl SelectedObject {
	pub fn select_single(&mut self, ent: Entity) {
		self.single = Some(ent);
		self.group.clear();
		self.group.insert(ent);
	}
	pub fn add_select(&mut self, ent: Entity) {
		self.single = Some(ent);
		self.group.insert(ent);
	}
	pub fn deselect_single(&mut self, ent: Entity) {
		self.group.remove(&ent);
		if let Some(ent_obj) = self.single {
			if ent_obj == ent {
				self.single = self.group.iter().next().cloned();
			}
		}
	}
	pub fn deselect_all(&mut self) {
		self.single = None;
		self.group.clear();
	}
}

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ResizePinPoint(pub Option<Vec2>);

// #[derive(Component)]
// pub struct RevertChange(RevertType);

// pub enum RevertType {
// 	Move(),
// 	Draw(),
// 	SvgPath(),
// }

#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BeginSelectPoint(pub Option<Vec2>);

/// Main | Background Color
#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintInk(pub Srgba, pub Srgba);

impl Default for PaintInk {
	fn default() -> Self {
		Self(BLACK, WHITE)
	}
}

#[derive(
	Actionlike, Reflect, Debug, EnumIter, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize,
)]
pub enum VerifyAction {
	// Escape,
	// GLOBAL
	#[reflect(@ToolTip("Delete data depends types"))]
	Delete,
	#[reflect(@ToolTip("Delete selected objects"))]
	DeleteObject,

	// TEXT
	#[reflect(@ToolTip("Add new Text to the world"))]
	AddText,

	// IMAGE
	#[reflect(@ToolTip("Add new Image to the world (500^2)"), @ToolPath("document-new.png"))]
	NewImage,

	// SVG
	#[reflect(@ToolTip("Add point at selected Path"))]
	AddPoint,
	#[reflect(@ToolTip("Close/Disclose selected Path"))]
	ToggleClose,
	#[reflect(@ToolTip("Apply color to selected Path"))]
	PathApplyColor,

	#[reflect(@ToolTip("Undo objects"))]
	Undo,
	#[reflect(@ToolTip("Redo objects"))]
	Redo,
	// Control,
	// Add,
	// SvgClose,
	// Scale,
}

#[derive(Actionlike, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ClientAction {
	Drawing,
	Cut,
}
