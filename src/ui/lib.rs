use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use leafwing_input_manager::prelude::*;

#[derive(Component)]
pub(super) struct CoverMenu;

pub struct UILayer;
impl UILayer {
	pub const COVER: ZIndex = ZIndex::Global(-9);
	pub const MENU_HOLDER: ZIndex = ZIndex::Global(-8);
	pub const SCROLL_VIEW: ZIndex = ZIndex::Global(-7);
	// pub const SCROLL_BAR: ZIndex = ZIndex::Global(-6);
	pub const RESIZE_BAR: ZIndex = ZIndex::Global(-5);
	pub const SUB_MENU: ZIndex = ZIndex::Global(-4);
}

// #[derive(Bundle, Default)]
// pub struct ResizeBundle {
// 	mark: ResizeHandler,
// 	button: Button,
// 	node_bundle: NodeBundle,
// 	effect: EffectItem,
// 	pickable: Pickable,
// 	on_drag: Maybe<On<Pointer<Drag>>>,
// }

// impl ResizeBundle {
// 	pub fn row(min: f32) -> Self {
// 		Self {
// 			node_bundle: NodeBundle::row_separate().edit(|edit| edit.style.top = Val::Px(min)),
// 			on_drag: Maybe::new(On::<Pointer<Drag>>::target_component_mut::<Style>(
// 				move |listener, style| {
// 					let y_move = listener.delta.y;
// 					let y = if let Val::Px(y) = &mut style.top {
// 						*y += y_move;
// 						y
// 					} else if let Val::Px(y) = &mut style.bottom {
// 						*y -= y_move;
// 						y
// 					} else {
// 						unreachable!();
// 					};
// 					*y = y.max(min);
// 				},
// 			)),
// 			pickable: Pickable {
// 				should_block_lower: false,
// 				is_hoverable: true,
// 			},
// 			..default()
// 		}
// 	}
// 	pub fn column(min: f32) -> Self {
// 		Self {
// 			node_bundle: NodeBundle::column_separate().edit(|edit| edit.style.left = Val::Px(min)),
// 			on_drag: Maybe::new(On::<Pointer<Drag>>::target_component_mut::<Style>(
// 				move |listener, style| {
// 					let x_move = listener.delta.x;
// 					let x = if let Val::Px(x) = &mut style.left {
// 						*x += x_move;
// 						x
// 					} else if let Val::Px(x) = &mut style.right {
// 						*x -= x_move;
// 						x
// 					} else {
// 						unreachable!();
// 					};
// 					*x = x.max(min);
// 				},
// 			)),
// 			pickable: Pickable {
// 				should_block_lower: false,
// 				is_hoverable: true,
// 			},
// 			..default()
// 		}
// 	}
// }

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedUI;

#[derive(Component)]
pub(super) struct SubMenuHold;

pub(super) struct MenuHold;

impl Component for MenuHold {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			if let Some(mut z_index) = world.entity_mut(entity).get_mut::<ZIndex>() {
				*z_index = UILayer::MENU_HOLDER;
			}
		});
	}
}

/// Close from taking action
pub struct SubMenu;

impl Component for SubMenu {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut ent_mut = world.entity_mut(entity);
			if let Some(mut style) = ent_mut.get_mut::<Style>() {
				style.padding = UiRect::all(Val::Px(3.0));
				style.border = UiRect::all(Val::Px(1.0));
				style.margin = UiRect::all(Val::Px(1.0));
				style.position_type = PositionType::Absolute;
				style.justify_content = JustifyContent::SpaceBetween;
			}
			if let Some(mut z_index) = ent_mut.get_mut::<ZIndex>() {
				*z_index = UILayer::SUB_MENU;
			}
			if let Some(mut border_radius) = ent_mut.get_mut::<BorderRadius>() {
				*border_radius = BorderRadius::all(Val::Px(3.0));
			}
		});
	}
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(super) struct MenuActive;

pub(super) struct MenuItem;

impl Component for MenuItem {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			if let Some(mut style) = world.entity_mut(entity).get_mut::<Style>() {
				style.padding = UiRect::horizontal(Val::Px(8.0));
				style.margin = UiRect::all(Val::Px(2.0));
			}
		});
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Reflect, Hash)]
pub enum UIAction {
	Scroll,
	Pull,
	SwapScroll,
	Primary,
}

impl Actionlike for UIAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			Self::Scroll | Self::Pull => InputControlKind::DualAxis,
			_ => InputControlKind::Button,
		}
	}
}

impl UIAction {
	pub fn bind_default() -> InputMap<Self> {
		InputMap::new([(Self::Primary, MouseButton::Left)])
			.with_one_to_many(Self::SwapScroll, [KeyCode::ShiftLeft, KeyCode::ShiftRight])
			.with_dual_axis(Self::Scroll, MouseScroll::default())
			.with_dual_axis(Self::Pull, MouseMove::default())
	}
}
