use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use focus::HoverMap;
use leafwing_input_manager::prelude::*;
use picking_core::PickSet;

use crate::trait_bevy::BevyColorTheme;

pub mod drop_at;
pub mod effect;
pub mod icon_build;
pub mod lib;
pub mod observe_own;
pub mod resizer;
pub mod scrollable;
pub mod tab;
pub mod text_build;

pub use drop_at::*;
pub use effect::*;
pub use icon_build::*;
pub use lib::*;
pub use observe_own::*;
use resizer::*;
pub use scrollable::*;
pub use tab::*;
pub use text_build::*;

pub(super) struct EffectUIPlugin;
impl Plugin for EffectUIPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(InputManagerPlugin::<UIAction>::default())
			.init_resource::<ActionState<UIAction>>()
			.insert_resource(UIAction::bind_default())
			.init_resource::<ThemeMode>()
			.add_systems(Startup, setup_menu_cover)
			.add_systems(PreUpdate, effect_run.after(PickSet::Last))
			.add_systems(
				PostUpdate,
				(
					change_on_effect,
					has_active_menu,
					despawn_once_button,
					scroll_ui,
					effect_run_menu,
					select_tab,
					not_container_yet,
					not_tab_yet,
					resize_handler,
					// render_all_ui::<EffectItem>,
				),
			);
	}
}

// #[derive(Component)]
// pub struct DebugUIColor(Srgba);

// fn render_all_ui<T: Component>(
// 	mut gizmos: Gizmos,
// 	mut cmd: Commands,
// 	query_ui: Query<(Entity, &Node, &GlobalTransform, Option<&DebugUIColor>), With<T>>,
// ) {
// 	query_ui.iter().for_each(|(ent, node, form, op_color)| {
// 		let color = if let Some(color) = op_color {
// 			color.0
// 		} else {
// 			let color = Srgba::rgb_u8(
// 				rand::thread_rng().gen(),
// 				rand::thread_rng().gen(),
// 				rand::thread_rng().gen(),
// 			);
// 			cmd.entity(ent).insert(DebugUIColor(color));
// 			color
// 		};
// 		let mut rect = node.logical_rect(form);
// 		rect.min.y *= -1.;
// 		rect.max.y *= -1.;
// 		gizmos.rect_2d(rect.center(), 0., rect.size(), color);
// 	});
// }

fn setup_menu_cover(world: &mut World) {
	world.spawn((
		NodeBundle {
			style: Style {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				position_type: PositionType::Absolute,
				..default()
			},
			z_index: UILayer::COVER,
			..default()
		},
		Name::new("UI COVER"),
		Pickable::IGNORE,
		CoverMenu,
		On::<Pointer<Click>>::run(
			|mut cmd: Commands,
			 query_sub: Query<Entity, With<SubMenu>>,
			 query_active: Query<Entity, With<MenuActive>>| {
				query_sub.iter().for_each(|ent| {
					if let Some(ent_cmd) = cmd.get_entity(ent) {
						ent_cmd.despawn_recursive();
					}
				});
				query_active.iter().for_each(|ent| {
					if let Some(mut ent_cmd) = cmd.get_entity(ent) {
						ent_cmd.remove::<MenuActive>();
					}
				});
			},
		),
	));
}

fn has_active_menu(
	mut query_cover: Query<&mut Pickable, With<CoverMenu>>,
	query_menu: Query<(), Or<(With<SubMenu>, With<MenuActive>)>>,
) {
	let Ok(mut pickable) = query_cover.get_single_mut() else {
		return;
	};
	let pick = if query_menu.is_empty() {
		Pickable::IGNORE
	} else {
		Pickable::default()
	};
	*pickable = pick;
}

fn change_on_effect(
	mut query_text: Query<
		(&mut BackgroundColor, &PickingInteraction, Has<SelectedUI>),
		(With<EffectItem>, With<Button>),
	>,
) {
	query_text
		.iter_mut()
		.for_each(|(mut background, inter, is_selected)| {
			let color = if is_selected {
				Color::BEVY_GRAY
			} else {
				match inter {
					PickingInteraction::Pressed => Color::BEVY_GRAY,
					PickingInteraction::Hovered => Color::BEVY_DARK_GRAY,
					PickingInteraction::None => Color::BEVY_BLACK,
				}
			};
			background.set_if_neq(color.into());
		});
}

fn despawn_once_button(
	query_once: Query<Entity, With<SubMenu>>,
	query_active: Query<Entity, With<MenuActive>>,
	mut on_up: EventReader<Pointer<Up>>,
	mut cmd: Commands,
) {
	if on_up
		.read()
		.any(|pointed| query_once.contains(pointed.target()))
	{
		query_once
			.iter()
			.for_each(|ent| cmd.entity(ent).despawn_recursive());
		query_active.iter().for_each(|ent| {
			cmd.entity(ent).remove::<MenuActive>();
		});
	}
}

// fn sub_active(
// 	trigger: Trigger<OnInsert, SubSelected>,
// 	query_sub: Query<&Parent, With<SubSelected>>,
// 	query_menu: Query<&Children>,
// 	mut cmd: Commands,
// ) {
// 	let new_ent_sub = trigger.entity();
// 	let Ok(Ok(children)) = query_sub
// 		.get(new_ent_sub)
// 		.map(|parent| query_menu.get(parent.get()))
// 	else {
// 		return;
// 	};
// }
