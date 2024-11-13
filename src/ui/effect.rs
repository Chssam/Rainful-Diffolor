#![allow(unused)]
use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use bevy_mod_picking::prelude::*;
use i_cant_believe_its_not_bsn::*;

use crate::tool_tip::lib::ToolTipContent;

use super::*;

#[derive(Resource, Default)]
pub enum ThemeMode {
	#[default]
	Bevy,
	// Ayu,
	// Dark,
	// Light,
}

/// Theme Follower
#[derive(Default, Clone, PartialEq)]
pub enum EffectItem {
	#[default]
	Regular,
	Separable,
	None,
}

impl Component for EffectItem {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut ent_mut = world.entity_mut(entity);
			let effect = ent_mut.get::<EffectItem>().unwrap().clone();
			let is_text = ent_mut.contains::<Text>();
			if effect == EffectItem::None {
				return;
			}
			if let Some(mut border_color) = ent_mut.get_mut::<BorderColor>() {
				border_color.0 = Color::BLACK;
			}
			if let Some(mut background_color) = ent_mut.get_mut::<BackgroundColor>() {
				let color = if is_text {
					Color::NONE
				} else if effect == EffectItem::Separable {
					Color::BLACK
				} else {
					Color::BEVY_BLACK
				};
				background_color.0 = color;
			}
		});
	}
}

#[derive(Bundle, Default)]
pub struct EffectUIBundle {
	pub nodes: NodeBundle,
	effect: EffectItem,
	pickable: Pickable,
	button: Maybe<Button>,
	click: Maybe<ClickEffect>,
	item: Maybe<MenuItem>,
	menu: Maybe<MenuHold>,
	sub_menu: Maybe<SubMenu>,
	resize: Maybe<ResizeHandler>,
	scroll: Maybe<Scrollable>,
	tip: Maybe<ToolTipContent>,
	name: Maybe<Name>,
}

impl EffectUIBundle {
	pub fn row() -> Self {
		Self {
			nodes: NodeBundle {
				style: Style {
					width: Val::Percent(100.),
					padding: UiRect::all(Val::Px(2.0)),
					align_items: AlignItems::Center,
					flex_direction: FlexDirection::Row,
					overflow: Overflow::clip(),
					..default()
				},
				..default()
			},
			..default()
		}
	}
	pub fn column() -> Self {
		Self {
			nodes: NodeBundle {
				style: Style {
					height: Val::Percent(100.),
					padding: UiRect::all(Val::Px(2.0)),
					flex_direction: FlexDirection::Column,
					overflow: Overflow::clip(),
					..default()
				},
				..default()
			},
			..default()
		}
	}
	pub fn row_separate() -> Self {
		Self {
			effect: EffectItem::Separable,
			nodes: NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Px(2.0),
					..default()
				},
				background_color: Color::BEVY_DARK_GRAY.into(),
				..default()
			},
			..default()
		}
	}
	pub fn column_separate() -> Self {
		Self {
			effect: EffectItem::Separable,
			nodes: NodeBundle {
				style: Style {
					width: Val::Px(2.0),
					height: Val::Percent(100.0),
					..default()
				},
				background_color: Color::BEVY_DARK_GRAY.into(),
				..default()
			},
			..default()
		}
	}
	pub fn row_resize() -> Self {
		Self {
			effect: EffectItem::Separable,
			nodes: NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					border: UiRect::vertical(Val::Px(1.)),
					..default()
				},
				border_color: Color::BLACK.into(),
				..default()
			},
			resize: Maybe::new(ResizeHandler),
			..default()
		}
	}
	pub fn column_resize() -> Self {
		Self {
			effect: EffectItem::Separable,
			nodes: NodeBundle {
				style: Style {
					height: Val::Percent(100.0),
					border: UiRect::horizontal(Val::Px(1.)),
					..default()
				},
				border_color: Color::BLACK.into(),
				..default()
			},
			resize: Maybe::new(ResizeHandler),
			..default()
		}
	}
	pub fn text() -> Self {
		Self {
			nodes: NodeBundle {
				style: Style {
					padding: UiRect::axes(Val::Px(6.0), Val::Px(4.0)),
					margin: UiRect::all(Val::Px(1.0)),
					..default()
				},
				border_radius: BorderRadius::all(Val::Px(3.0)),
				..default()
			},
			pickable: Pickable {
				should_block_lower: false,
				is_hoverable: true,
			},
			name: Maybe::new(Name::new("Text Item")),
			..default()
		}
	}
	pub fn icon() -> Self {
		Self {
			nodes: NodeBundle {
				style: Style {
					padding: UiRect::all(Val::Px(2.)),
					margin: UiRect::all(Val::Px(1.)),
					..default()
				},
				border_radius: BorderRadius::all(Val::Px(3.0)),
				..default()
			},
			pickable: Pickable {
				should_block_lower: false,
				is_hoverable: true,
			},
			name: Maybe::new(Name::new("Icon Item")),
			..default()
		}
	}
	pub fn full(mut self) -> Self {
		let style = &mut self.nodes.style;
		style.flex_grow = 1.0;
		style.align_self = AlignSelf::Stretch;
		self
	}
	pub fn scroll(mut self) -> Self {
		self.scroll = Maybe::new(Scrollable);
		self.pickable.should_block_lower = true;
		let style = &mut self.nodes.style;
		style.left = Val::Px(0.);
		style.top = Val::Px(0.);
		style.flex_grow = 1.0;
		style.flex_wrap = FlexWrap::Wrap;
		self
	}
	pub fn scroll_full(mut self) -> Self {
		self.scroll = Maybe::new(Scrollable);
		self.pickable.should_block_lower = true;
		let style = &mut self.nodes.style;
		style.left = Val::Px(0.);
		style.top = Val::Px(0.);
		style.flex_grow = 1.0;
		style.flex_wrap = FlexWrap::NoWrap;
		self
	}
	pub fn flex_start(mut self) -> Self {
		let style = &mut self.nodes.style;
		style.align_items = AlignItems::Baseline;
		style.justify_content = JustifyContent::FlexStart;
		self
	}
	pub fn item(mut self) -> Self {
		self.button = Maybe::new(Button);
		self.item = Maybe::new(MenuItem);
		self.name = Maybe::new(Name::new("Item In Menu"));
		self
	}
	pub fn menu(mut self) -> Self {
		self.button = Maybe::new(Button);
		self.menu = Maybe::new(MenuHold);
		self.name = Maybe::new(Name::new("Menu"));
		self
	}
	pub fn no_padding(mut self) -> Self {
		self.nodes.style.padding = UiRect::ZERO;
		self
	}
	pub fn auto_size(mut self) -> Self {
		let style = &mut self.nodes.style;
		style.width = Val::Auto;
		style.height = Val::Auto;
		self
	}
	pub fn node(mut self, edit: impl FnOnce(&mut NodeBundle)) -> Self {
		edit(&mut self.nodes);
		self
	}
	pub fn click(mut self) -> Self {
		self.button = Maybe::new(Button);
		self.click = Maybe::new(ClickEffect);
		self
	}
	pub fn button(mut self) -> Self {
		self.button = Maybe::new(Button);
		self
	}
	pub fn tip<T: Reflect>(mut self, comp: T) -> Self {
		self.tip = Maybe::new(ToolTipContent::new(comp));
		self
	}
	pub fn no_effect(mut self) -> Self {
		self.effect = EffectItem::None;
		self
	}
	pub fn no_pick(mut self) -> Self {
		self.pickable = Pickable::IGNORE;
		self
	}
	pub fn only_pick(mut self) -> Self {
		self.pickable = Pickable::default();
		self
	}
	pub fn grow(mut self) -> Self {
		self.nodes.style.flex_grow = 1.;
		self
	}
}
