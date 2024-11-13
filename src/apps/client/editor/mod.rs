mod lib;
mod ui_trait_fn;
mod view_ui;
use i_cant_believe_its_not_bsn::{WithChild, WithChildren};
use lib::*;
use ui_trait_fn::*;
use view_ui::*;

use std::{marker::PhantomData, num::NonZero, path::Path};

use aery::{edges::RelationCommands, prelude::*};
use arboard::Clipboard;
use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};
use bevy_mod_picking::prelude::{Up as PickUp, *};
use image::{EncodableLayout, Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use leafwing_input_manager::prelude::*;
use rainful_diffolor::{source_to_docs, ToEmbedPath};
use strum::IntoEnumIterator;

use crate::{tool_tip::lib::ToolsInfo, trait_bevy::*, ui::*};

use super::*;

pub(super) struct ClientEditorPlugin;
impl Plugin for ClientEditorPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<NewBrush>()
			.add_systems(OnEnter(RdioClientState::Online), editor_observes)
			.add_systems(
				Update,
				(
					update_ui_content,
					display_color_update,
					update_color_image,
					color_slider,
					display_directory_object,
					hex_color_text,
					select_brush,
					select_color,
					update_option_perm,
					opting_permission,
					sort_object_z_info,
				)
					.run_if(in_state(RdioClientState::Online)),
			)
			.add_systems(
				PostUpdate,
				(op_menu_object, update_visiblity).run_if(in_state(RdioClientState::Online)),
			);
	}
}

fn editor_observes(world: &mut World) {
	let all_client_observe = [
		world.observe(new_brush_collection).id(),
		world.observe(add_obj_info).id(),
	];
	all_client_observe.into_iter().for_each(|ent_obs| {
		world
			.commands()
			.entity(ent_obs)
			.insert(StateScoped(RdioClientState::Online));
	});
}

pub(super) fn editor_ui(
	mut editors: EditorRdio,
	main_user_query: Query<(Entity, &RGBAhsl), With<MainUser>>,
) {
	use EditorPosition::*;

	let (main_user_ent, color_handle) = main_user_query.single();
	let path_icon = Path::new("Icon");

	let mut bar_1 = editors.cmd.entity(editors.editor_part.part(&MenuBar1));
	bar_1.with_childs([
		(
			EffectUIBundle::text().menu(),
			WithChild(TextBuild::single("File")),
			OwnObserve::new(move |trigger: Trigger<RunEffect>, mut cmd: Commands| {
				use ToolsStandAlone::*;
				cmd.spawn((
					DropDownAt::bottom(trigger.entity()),
					WithChildren(
						[
							Save,
							ExportImage,
							ExportCanvas,
							ExportSvgRelative,
							ExportSvgAbsolute,
						]
						.iter()
						.map(|action| {
							(
								EffectUIBundle::text().item(),
								WithChild(TextBuild::single(&action.as_reflect().tool_name())),
								instant_action(main_user_ent, *action),
							)
						})
						.collect::<Vec<_>>(),
					),
				));
			}),
		),
		(
			EffectUIBundle::text().menu(),
			WithChild(TextBuild::single("Edit")),
			OwnObserve::new(move |trigger: Trigger<RunEffect>, mut cmd: Commands| {
				use ToolsStandAlone::*;
				cmd.spawn((
					DropDownAt::bottom(trigger.entity()),
					WithChildren(
						[Undo, Redo, Cut, Copy, Paste, Preference]
							.iter()
							.map(|action| {
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single(&action.as_reflect().tool_name())),
									instant_action(main_user_ent, *action),
								)
							})
							.collect::<Vec<_>>(),
					),
				));
			}),
		),
		(
			EffectUIBundle::text().menu(),
			WithChild(TextBuild::single("View")),
			OwnObserve::new(|trigger: Trigger<RunEffect>, mut cmd: Commands| {
				let grider = VisualGrid(false);
				cmd.spawn((
					DropDownAt::bottom(trigger.entity()),
					WithChild((
						EffectUIBundle::text().item(),
						WithChild(TextBuild::single(&grider.as_reflect().tool_name())),
						OwnObserve::new(
							|_trigger: Trigger<RunEffect>,
							 mut query_user: Query<&mut VisualGrid, With<MainUser>>| {
								let Ok(mut visual_grid) = query_user.get_single_mut() else {
									return;
								};
								visual_grid.0 = !visual_grid.0;
							},
						),
					)),
				));
			}),
		),
		(
			EffectUIBundle::text().menu(),
			WithChild(TextBuild::single("Action")),
			OwnObserve::new(move |trigger: Trigger<RunEffect>, mut cmd: Commands| {
				cmd.spawn((
					DropDownAt::bottom(trigger.entity()),
					WithChildren(
						VerifyAction::iter()
							.map(|action| {
								(
									EffectUIBundle::text().item().tip(action),
									WithChild(TextBuild::single(&action.as_reflect().tool_name())),
									instant_action(main_user_ent, action),
								)
							})
							.collect::<Vec<_>>(),
					),
				));
			}),
		),
	]);

	// MARK: Second Root
	let mut bar_2 = editors.cmd.entity(editors.editor_part.part(&MenuBar2));
	bar_2.with_child((
		EffectUIBundle::row().scroll().flex_start(),
		WithChildren({
			use VerifyAction::*;
			[
				StrokeCapButt,
				StrokeCapRound,
				StrokeCapSquare,
				StrokeJoinBevel,
				StrokeJoinMiter,
				StrokeJoinRound,
			]
			.map(|action| {
				(
					EffectUIBundle::icon().click().tip(action),
					WithChild(IconBuild::medium(
						path_icon.join(action.as_reflect().path_img()).embed(),
					)),
					instant_action(main_user_ent, action),
				)
			})
		}),
	));

	let mut left_top_panel = editors.cmd.entity(editors.editor_part.part(&PanelLeftTop));
	left_top_panel.with_child((
		EffectUIBundle::column().full(),
		WithChild((
			EffectUIBundle::row().scroll().flex_start(),
			WithChildren(EditorTools::iter().map(move |action| {
				(
					EffectUIBundle::icon().click().tip(action),
					WithChild(
						IconBuild::medium(path_icon.join(action.as_reflect().path_img()).embed())
							.large(),
					),
					OwnObserve::new(
						move |_trigger: Trigger<RunEffect>,
						      mut query_target: Query<&mut ActionState<EditorTools>>| {
							let Ok(mut action_state) = query_target.get_mut(main_user_ent) else {
								error!("Targeting Non Exist Entity");
								return;
							};
							action_state.release(&action);
							action_state.press(&action);
						},
					),
				)
			})),
		)),
	));

	let mut left_bottom_panel = editors
		.cmd
		.entity(editors.editor_part.part(&PanelLeftBottom));

	#[derive(Component)]
	struct DisplayDrawType;
	#[derive(Component)]
	struct DisplayScaleBrush;
	#[derive(Component)]
	struct DisplaySpacing;
	#[derive(Component)]
	struct DisplayBlur;

	left_bottom_panel.with_child((
		EffectUIBundle::column().full(),
		WithChild((
			EffectUIBundle::column().scroll_full(),
			WithChild((
				EffectUIBundle::text().click(),
				WithChild((
					TextBuild::multiple(vec!["Draw Type: ", "Normal"]),
					DisplayDrawType,
				)),
				OwnObserve::new(
					|_trigger: Trigger<RunEffect>,
					 mut query_text: Query<&mut Text, With<DisplayDrawType>>,
					 mut query_user: Query<&mut DrawType, With<MainUser>>| {
						let Ok(mut draw_type) = query_user.get_single_mut() else {
							return;
						};
						*draw_type = match *draw_type {
							DrawType::Normal => DrawType::Behind,
							DrawType::Behind => DrawType::Normal,
							_ => unreachable!(),
						};

						let mut text = query_text.single_mut();
						text.sections[1].value = draw_type.as_reflect().tool_name();
					},
				),
			)),
			WithChild((
				EffectUIBundle::text().click(),
				WithChild((TextBuild::multiple(vec!["Scale: ", "1"]), DisplayScaleBrush)),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text, With<DisplayScaleBrush>>,
					 mut query_user: Query<&mut BrushScale, With<MainUser>>| {
						let Ok(mut brush_scale) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.single_mut();
						brush_scale.add(event.delta.x as i8);
						text.sections[1].value = brush_scale.get().to_string();
					},
				),
			)),
			WithChild((
				EffectUIBundle::text().click(),
				WithChild((TextBuild::multiple(vec!["Spacing: ", "1"]), DisplaySpacing)),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text, With<DisplaySpacing>>,
					 mut query_user: Query<&mut DrawingSpacing, With<MainUser>>| {
						let Ok(mut draw_spacing) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.single_mut();
						let calculate = (draw_spacing.get() as i16 + event.delta.x as i16)
							.clamp(1, u8::MAX as i16) as u8;
						let Some(non_zero) = NonZero::new(calculate) else {
							return;
						};
						**draw_spacing = non_zero;
						text.sections[1].value = draw_spacing.get().to_string();
					},
				),
			)),
			WithChild((
				EffectUIBundle::text().click(),
				WithChild((TextBuild::multiple(vec!["Blur: ", "0"]), DisplayBlur)),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text, With<DisplayBlur>>,
					 mut query_user: Query<&mut BlurScale, With<MainUser>>| {
						let Ok(mut blur_scale) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.single_mut();

						blur_scale.0 = (blur_scale.0 * 100.0 + event.delta.x).round() / 100.0;
						text.sections[1].value = blur_scale.0.to_string();
					},
				),
			)),
		)),
	));

	let mut left_panel = editors.cmd.entity(editors.editor_part.part(&PanelLeft));
	left_panel.with_child((
		EffectUIBundle::row(),
		WithChildren(
			[DisplayColor::Foreground, DisplayColor::Background].map(|comp| {
				(
					NodeBundle {
						style: Style {
							width: Val::Px(25.0),
							height: Val::Px(25.0),
							border: UiRect::all(Val::Px(2.0)),
							margin: UiRect::all(Val::Px(2.0)),
							..default()
						},
						border_color: Color::BLACK.into(),
						..default()
					},
					DownEffect,
					instant_action(main_user_ent, ToolsStandAlone::ColorSwap),
					comp,
				)
			}),
		),
	));

	let mut middle_panel = editors.cmd.entity(editors.editor_part.part(&PanelMiddle));
	middle_panel.with_child((
		TextBuild::single("Cord: -"),
		EditorInfoComp,
		Pickable::IGNORE,
	));

	let mut right_top_panel = editors.cmd.entity(editors.editor_part.part(&PanelRightTop));
	right_top_panel.with_child((
		EffectUIBundle::row()
			.scroll()
			.flex_start()
			.no_effect()
			.node(|node| node.background_color = Color::WHITE.into()),
		BrushCollector,
	));

	let mut right_bottom = editors
		.cmd
		.entity(editors.editor_part.part(&PanelRightBottom));

	let new_img = VerifyAction::NewImage;
	right_bottom.with_child((
		EffectUIBundle::row(),
		WithChild((
			EffectUIBundle::icon().click(),
			WithChild(
				IconBuild::medium(path_icon.join(new_img.as_reflect().path_img()).embed()).large(),
			),
			instant_action(main_user_ent, new_img),
		)),
		WithChildren(
			[ObjectActionNet::LayerUp, ObjectActionNet::LayerDown].map(|action| {
				let main_only = |action: ObjectActionNet| {
					OwnObserve::new(
						move |_trigger: Trigger<RunEffect>,
						      mut client: ResMut<ClientConnectionManager>,
						      query_user: Query<&SelectedObject, With<MainUser>>,
						      query_point: Query<&Parent, With<ObjectPoint>>| {
							let Ok(Some(obj_ent)) = query_user
								.get_single()
								.map(|selected_obj| selected_obj.single)
							else {
								return;
							};
							let obj_ent = query_point
								.get(obj_ent)
								.map(|parent| parent.get())
								.unwrap_or(obj_ent);
							client
								.send_message_to_target::<MainChannel, ObjectActionToServer>(
									&mut ObjectActionToServer { obj_ent, action },
									NetworkTarget::All,
								)
								.unwrap_or_else(|e| {
									error!("Fail to send message: {:?}", e);
								});
						},
					)
				};
				(
					EffectUIBundle::icon().button().click(),
					WithChild(
						IconBuild::medium(path_icon.join(action.as_reflect().path_img()).embed())
							.large(),
					),
					main_only(action),
				)
			}),
		),
	));

	right_bottom.with_child((
		EffectUIBundle::column().full(),
		WithChild((EffectUIBundle::column().scroll_full(), ObjInfoController)),
	));

	let mut bottom_left = editors
		.cmd
		.entity(editors.editor_part.part(&PanelBottomLeft));

	bottom_left.with_child((
		EffectUIBundle::column().full(),
		NotTabYet::new("Colors"),
		WithChild((
			EffectUIBundle::column().scroll_full().flex_start(),
			WithChild((
				EffectUIBundle::text()
					.only_pick()
					.button()
					.tip("Left Click: Copy hex color code\nRight Click: Paste hex color code"),
				WithChild((TextBuild::multiple(vec!["Hex: ", ""]), HexColorText)),
				On::<Pointer<Down>>::run(
					|event: Listener<Pointer<Down>>,
					 query_hex: Query<&Text, With<HexColorText>>,
					 mut query_user: Query<&mut PaintInk, With<MainUser>>| {
						let (Ok(text_hex), Ok(mut paint)) =
							(query_hex.get_single(), query_user.get_single_mut())
						else {
							return;
						};
						if let PointerButton::Primary = event.button {
							let mut clip_board = Clipboard::new().unwrap();
							clip_board
								.set_text(text_hex.sections[1].value.clone())
								.unwrap();
						} else if let PointerButton::Secondary = event.button {
							let mut clip_board = Clipboard::new().unwrap();
							let Ok(Ok(pasted_hex)) =
								clip_board.get_text().map(|text| Srgba::hex(text))
							else {
								return;
							};
							paint.0 = pasted_hex;
						}
					},
				),
			)),
			WithChildren(
				ColorPanelChanger::iter()
					.map(|comp| {
						use ColorPanelChanger::*;
						let (handle_img, width, height) = match comp {
							Red => (color_handle.red.clone_weak(), 256.0, Val::Auto),
							Green => (color_handle.green.clone_weak(), 256.0, Val::Auto),
							Blue => (color_handle.blue.clone_weak(), 256.0, Val::Auto),
							Alpha => (color_handle.alpha.clone_weak(), 256.0, Val::Auto),
							Hue => (color_handle.hue.clone_weak(), 361.0, Val::Auto),
							Saturation => (color_handle.saturation.clone_weak(), 202.0, Val::Auto),
							Lightness => (color_handle.lightness.clone_weak(), 202.0, Val::Auto),
							SatLight => {
								(color_handle.sat_light.clone_weak(), 202.0, Val::Px(202.0))
							},
						};
						let mut comp_name = comp.as_reflect().tool_name();
						comp_name.push_str(": ");
						(
							ButtonBundle {
								style: Style {
									width: Val::Px(width),
									height,
									padding: UiRect::axes(Val::Px(6.), Val::Px(2.)),
									align_items: AlignItems::FlexStart,
									justify_content: JustifyContent::FlexEnd,
									flex_shrink: 0.,
									..default()
								},
								image: handle_img.into(),
								..default()
							},
							LockableOnUI,
							comp,
							WithChild((
								TextBuild::multiple(vec![&comp_name, "0"])
									.custom_color(Color::BLACK),
								comp,
							)),
						)
					})
					.collect::<Vec<_>>(),
			),
		)),
	));

	bottom_left.with_child({
		let color_node = |color: Srgba| {
			(
				NodeBundle {
					style: Style {
						border: UiRect::all(Val::Px(1.0)),
						min_width: Val::Px(20.0),
						min_height: Val::Px(20.0),
						max_width: Val::Px(25.0),
						max_height: Val::Px(25.0),
						flex_basis: Val::Px(1.0),
						..default()
					},
					background_color: color.into(),
					border_color: Color::BEVY_DARK_GRAY.into(),
					..default()
				},
				Pickable {
					should_block_lower: false,
					is_hoverable: true,
				},
				ColorChoice,
			)
		};

		let mut chained = Vec::with_capacity(406);
		let mut basic = Srgba::iter_basic().map(|color| color_node(color)).to_vec();
		chained.append(&mut basic);
		let mut css = Srgba::iter_css().map(|color| color_node(color)).to_vec();
		chained.append(&mut css);
		let mut tailwind = Srgba::iter_tailwind()
			.map(|color| color_node(color))
			.to_vec();
		chained.append(&mut tailwind);
		(
			EffectUIBundle::column().full(),
			NotTabYet::new("Palette"),
			WithChild((
				EffectUIBundle::row().scroll().flex_start(),
				WithChildren(chained),
			)),
		)
	});

	#[derive(Component)]
	struct PathModeText;
	let mut bottom_right = editors
		.cmd
		.entity(editors.editor_part.part(&PanelBottomRight));

	bottom_right.with_child((
		EffectUIBundle::column().full(),
		NotTabYet::new("Directory"),
		WithChild((
			EffectUIBundle::column().scroll_full().flex_start(),
			DisplayObjectDirectory,
			WithChild((
				EffectUIBundle::text().button(),
				DownEffect,
				WithChild((
					TextBuild::multiple(vec![
						"Drop Path Mode: ",
						&DropPathMode::AsObject.as_reflect().tool_name(),
						" | Entity Name : Directory",
					]),
					PathModeText,
				)),
				OwnObserve::new(
					|_trigger: Trigger<RunEffect>,
					 state: Res<State<DropPathMode>>,
					 mut next_state: ResMut<NextState<DropPathMode>>,
					 mut query_text: Query<&mut Text, With<PathModeText>>| {
						let mut texted = query_text.single_mut();
						let switched_mode = match state.get() {
							DropPathMode::AsObject => DropPathMode::SaveLocation,
							DropPathMode::SaveLocation => DropPathMode::AsObject,
						};
						texted.sections[1].value = switched_mode.as_reflect().tool_name();
						next_state.set(switched_mode);
					},
				),
			)),
		)),
	));

	bottom_right.with_child((
		EffectUIBundle::column().full(),
		NotTabYet::new("Object Perm"),
		WithChild((
			EffectUIBundle::text().menu(),
			WithChild((TextBuild::multiple(vec!["Owner: ", ""]), OwnerDisplay)),
			OwnerDisplay,
		)),
		WithChild((
			EffectUIBundle::text().menu(),
			WithChild((TextBuild::single("None"), ObjectPermText)),
			OwnObserve::new(|trigger: Trigger<RunEffect>, mut cmd: Commands| {
				use NetworkTarget::*;
				cmd.spawn((
					DropDownAt::bottom(trigger.entity()),
					WithChildren([None, All, AllExcept(vec![]), Only(vec![])].map(|net| {
						(
							EffectUIBundle::text().item(),
							WithChild(TextBuild::single(&net.as_reflect().tool_name())),
							OptingPer(net),
						)
					})),
				));
			}),
		)),
		WithChild((
			EffectUIBundle::row(),
			WithChild((
				EffectUIBundle::column()
					.no_effect()
					.full()
					.node(|node| node.background_color = Color::BEVY_DARK_GRAY.into()),
				FiltedOutPer,
			)),
			WithChild((
				EffectUIBundle::column()
					.no_effect()
					.full()
					.node(|node| node.background_color = Color::BEVY_DARK_GRAY.into()),
				FiltedInPer,
			)),
		)),
	));
}

#[derive(Component)]
struct ObjectPermText;

// Time limit; Random Code Happen
fn update_option_perm(
	mut cmd: Commands,
	mut query_display_perm: Query<&mut Text, (With<ObjectPermText>, Without<OwnerDisplay>)>,
	mut query_dis_owner: Query<&mut Text, (With<OwnerDisplay>, Without<ObjectPermText>)>,
	query_filted_in: Query<(Entity, Option<&Children>), With<FiltedInPer>>,
	query_filted_out: Query<(Entity, Option<&Children>), With<FiltedOutPer>>,
	query_obj: Query<(Entity, &ObjectAccess, &ObjectOwner), With<ObjectWorld>>,
	query_access_change: Query<Entity, (Changed<ObjectAccess>, With<ObjectWorld>)>,
	query_user: Query<Ref<SelectedObject>, With<MainUser>>,
	query_user_ids: Query<(&UserId, &SharingName)>,
) {
	let filted_in = query_filted_in.single();
	let filted_out = query_filted_out.single();
	let Ok(selected_obj) = query_user.get_single() else {
		return;
	};
	let Some(Ok((obj_ent, access, owner))) =
		selected_obj.single.map(|single| query_obj.get(single))
	else {
		return;
	};
	if !selected_obj.is_changed() && !query_access_change.iter().any(|ent| ent == obj_ent) {
		return;
	}
	let mut text_perm = query_display_perm.single_mut();

	if let Some((_, name)) = query_user_ids.iter().find(|(id, _)| id.0 == owner.0) {
		let mut text_owner = query_dis_owner.single_mut();
		text_owner.sections[1].value = name.0.clone();
	};

	let filter = |vec: &Vec<ClientId>| {
		query_user_ids
			.iter()
			.fold((Vec::new(), Vec::new()), |(mut out_, mut in_), v| {
				if vec.contains(&v.0 .0) {
					in_.push(v);
				} else {
					out_.push(v);
				}
				(out_, in_)
			})
	};
	let (value, (filted_out_vec, filted_in_vec)) = match &access.0 {
		NetworkTarget::None => (NetworkTarget::None, (vec![], vec![])),
		NetworkTarget::AllExcept(vec) => (NetworkTarget::AllExcept(vec![]), filter(vec)),
		NetworkTarget::All => (NetworkTarget::All, (vec![], vec![])),
		NetworkTarget::Only(vec) => (NetworkTarget::Only(vec![]), filter(vec)),
		_ => return,
	};
	text_perm.sections[0].value = value.as_reflect().tool_name();
	if let Some(child) = filted_out.1 {
		child
			.iter()
			.for_each(|ent| cmd.entity(*ent).despawn_recursive());
	}
	if let Some(child) = filted_in.1 {
		child
			.iter()
			.for_each(|ent| cmd.entity(*ent).despawn_recursive());
	}
	filted_out_vec.iter().for_each(|(user_id, name)| {
		cmd.entity(filted_out.0).with_children(|parent| {
			let id = user_id.0.clone();
			parent.spawn((
				TextBuild::single(&name.0),
				Pickable::default(),
				On::<Pointer<Down>>::run(move |mut client: ResMut<ConnectionManager>| {
					let _ = client.send_message_to_target::<MainChannel, PerActionNet>(
						&mut PerActionNet {
							obj_ent,
							action: PerAction::Add(id),
						},
						NetworkTarget::All,
					);
				}),
			));
		});
	});
	filted_in_vec.iter().for_each(|(user_id, name)| {
		cmd.entity(filted_in.0).with_children(|parent| {
			let id = user_id.0.clone();
			parent.spawn((
				TextBuild::single(&name.0),
				Pickable::default(),
				On::<Pointer<Down>>::run(move |mut client: ResMut<ConnectionManager>| {
					let _ = client.send_message_to_target::<MainChannel, PerActionNet>(
						&mut PerActionNet {
							obj_ent,
							action: PerAction::Remove(id),
						},
						NetworkTarget::All,
					);
				}),
			));
		});
	});
}

fn opting_permission(
	mut client: ResMut<ConnectionManager>,
	mut on_up: EventReader<Pointer<PickUp>>,
	query_op_perm: Query<&OptingPer>,
	query_user: Query<Ref<SelectedObject>, With<MainUser>>,
	query_obj: Query<Entity, With<ObjectWorld>>,
) {
	let Some(on) = on_up.read().next() else {
		return;
	};
	let Ok(opting) = query_op_perm.get(on.target()) else {
		return;
	};
	let Ok(selected_obj) = query_user.get_single() else {
		return;
	};
	let Some(Ok(obj_ent)) = selected_obj.single.map(|single| query_obj.get(single)) else {
		return;
	};
	let _ = client.send_message_to_target::<MainChannel, PerActionNet>(
		&mut PerActionNet {
			obj_ent,
			action: PerAction::Change(opting.0.clone()),
		},
		NetworkTarget::All,
	);
}

#[derive(Component)]
pub struct OwnerDisplay;

#[derive(Component)]
pub struct OptingPer(NetworkTarget);

#[derive(Component)]
pub struct FiltedOutPer;

#[derive(Component)]
pub struct FiltedInPer;

// #[derive(Component)]
// pub struct OptingView;

// #[derive(Component)]
// pub struct FiltedOutView;

// #[derive(Component)]
// pub struct FiltedInView;

fn display_directory_object(
	mut cmd: Commands,
	query_user: Query<&SelectedObject, (Changed<SelectedObject>, With<MainUser>)>,
	query_directory: Query<(Entity, &Children), With<DisplayObjectDirectory>>,
	query_obj: Query<
		(
			&SharingName,
			Option<&SaveLocation>,
			Has<ObjectImage>,
			Has<ObjectPath>,
		),
		With<ObjectWorld>,
	>,
) {
	let Ok((ent_directory, child_dir)) = query_directory.get_single() else {
		return;
	};
	let Ok(selected_obj) = query_user.get_single() else {
		return;
	};
	let default_directory = source_to_docs().unwrap();

	child_dir.iter().skip(1).for_each(|ent| {
		cmd.entity(*ent).despawn_recursive();
	});
	let mut directoryer = cmd.entity(ent_directory);
	for (obj_name, op_directory, is_img, is_path) in query_obj.iter_many(selected_obj.group.iter())
	{
		let save_to = op_directory
			.map(|path| path.0.clone())
			.unwrap_or(default_directory.clone());

		let save_to = if is_img {
			save_to.join("Image")
		} else if is_path {
			save_to.join("Svg")
		} else {
			return;
		};

		let label = format!("{}: {}", obj_name.0, save_to.to_string_lossy());
		directoryer.with_child(TextBuild::single(&label));
	}
}

fn add_obj_info(
	trigger: Trigger<OnAdd, ObjectWorld>,
	obj_control_query: Query<Entity, With<ObjInfoController>>,
	query_obj: Query<(Entity, &SharingName)>,
	mut cmd: Commands,
) {
	let ent_obj = trigger.entity();
	let Ok((ent_obj, obj_name)) = query_obj.get(ent_obj) else {
		return;
	};
	let ent_controller = obj_control_query.single();
	let path_icon = Path::new("Icon");

	let visible = AttactObject::Visibility;
	let lock_move = ObjectActionNet::LockMove;
	let relationed = MergeRelation::<ObjectRelationUI>::new(ent_obj);
	let row = cmd
		.spawn((
			EffectUIBundle::row(),
			InfoAt(ent_obj),
			relationed,
			WithChild((
				EffectUIBundle::icon().button(),
				visible.target_up(ent_obj),
				relationed,
				WithChild((
					IconBuild::medium(path_icon.join(visible.as_reflect().path_img()).embed()),
					Pickable::IGNORE,
					IconVisiblity,
					relationed,
				)),
			)),
			WithChild((
				EffectUIBundle::icon().button(),
				lock_move.target_up(ent_obj),
				relationed,
				WithChild((
					IconBuild::medium(path_icon.join(lock_move.as_reflect().path_img()).embed()),
					Pickable::IGNORE,
					IconMoveLock,
					relationed,
				)),
			)),
			WithChild((
				EffectUIBundle::text().button(),
				ObjectOption,
				relationed,
				WithChild((TextBuild::single(&obj_name), relationed)),
			)),
		))
		.id();
	cmd.entity(ent_controller).add_child(row);
}

#[derive(Clone, Copy)]
pub struct MergeRelation<T: Relation>(pub Entity, PhantomData<T>);

impl<T: Relation> MergeRelation<T> {
	fn new(target: Entity) -> Self {
		Self(target, PhantomData)
	}
}

impl<T: Relation> Component for MergeRelation<T> {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_insert(|mut world, entity, _component_id| {
			let target = world.entity(entity).get::<MergeRelation<T>>().unwrap().0;
			world
				.commands()
				.entity(entity)
				.set::<T>(target)
				.remove::<MergeRelation<T>>();
		});
	}
}

#[derive(Component)]
pub struct InfoAt(pub Entity);

#[derive(Component)]
pub struct ObjectOption;

fn sort_object_z_info(
	query_object: Query<
		(),
		(
			Changed<ObjectZLayer>,
			Without<ObjectPoint>,
			With<ObjectWorld>,
		),
	>,
	query_z: Query<&ObjectZLayer>,
	mut obj_control_query: Query<&mut Children, With<ObjInfoController>>,
	query_row_info: Query<&InfoAt>,
) {
	if query_object.is_empty() {
		return;
	}
	obj_control_query.single_mut().sort_by_key(|ent| {
		let Ok(ent_obj) = query_row_info.get(*ent) else {
			return -BEGIN_OBJ_Z_INDEX as i16;
		};
		query_z.get(ent_obj.0).map(|v| -v.0).unwrap_or_default()
	});
}

fn op_menu_object(
	mut on_down: EventReader<Pointer<Down>>,
	mut cmd: Commands,
	query_option: Query<&Parent, With<ObjectOption>>,
	query_row_info: Query<&InfoAt>,
	query_object: Query<
		(Option<(&AlphaLock, &PixelLock)>, Has<ObjectPath>, &Pickable),
		With<ObjectWorld>,
	>,
	query_user: Query<(&UserId, &SelectedObject), With<MainUser>>,
) {
	let Some(first_click) = on_down.read().next() else {
		return;
	};
	if first_click.button != PointerButton::Primary {
		return;
	}
	let Ok((user_id, selected_obj)) = query_user.get_single() else {
		return;
	};
	let top_ui = first_click.target;
	let Ok(Ok(ent_obj)) = query_option
		.get(top_ui)
		.map(|parent| query_row_info.get(parent.get()).map(|at| at.0))
	else {
		return;
	};
	let Ok((op_img, is_path, pickable)) = query_object.get(ent_obj) else {
		return;
	};
	fn text_in(check: bool, reflected: &dyn Reflect, on_up: On<Pointer<PickUp>>) -> impl Bundle {
		let mut value = format!(" {}", reflected.tool_name());
		let v = if check { '/' } else { 'X' };
		value.insert(0, v);
		(
			EffectUIBundle::text().item(),
			WithChild(TextBuild::single(&value)),
			on_up,
		)
	}

	let mut with_child = {
		let pick = AttactObject::Pick;
		let lock_pick = AttactObject::LockPick;
		WithChildren(vec![
			text_in(
				selected_obj.group.contains(&ent_obj),
				pick.as_reflect(),
				pick.target_up(ent_obj),
			),
			text_in(
				!pickable.is_hoverable,
				lock_pick.as_reflect(),
				lock_pick.target_up(ent_obj),
			),
		])
	};
	if let Some((alpha_lock, pixel_lock)) = op_img {
		let id = &user_id.0;
		let alpha_lock = alpha_lock.contains(id);
		let pix_lock = pixel_lock.contains(id);
		for (has_user, action) in [
			(alpha_lock, ObjectActionNet::LockAlpha),
			(pix_lock, ObjectActionNet::LockPixel),
		] {
			with_child.0.push(text_in(
				has_user,
				action.as_reflect(),
				action.target_up(ent_obj),
			));
		}
	}

	if is_path {}

	cmd.spawn((DropDownAt::bottom(top_ui), with_child));
}

pub(super) fn add_brush(mut cmd: Commands) {
	let create_data = |data: Vec<u8>| {
		data.into_iter()
			.map(|v| [255, 255, 255, v])
			.collect::<Vec<_>>()
			.concat()
	};
	let data_type = [
		(1, (vec![255])),
		(3, (vec![0, 255, 0, 255, 255, 255, 0, 255, 0])),
		(
			7,
			(vec![
				0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255, 0, 0,
				255, 255, 255, 255, 255, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 0,
				0, 0, 0, 0, 0, 0, 0, 0,
			]),
		),
	];
	for (sized, data) in data_type {
		let size = UVec2::splat(sized);
		let data_press = if data.len() > 100 {
			DataHold::to_compress(data.as_bytes())
		} else {
			DataHold::Uncompress(data.clone())
		};
		cmd.trigger(NewBrush(
			BrushRef::CustomAlpha {
				brush: data_press,
				size,
			},
			size,
			create_data(data),
		));
	}

	let size = UVec2::splat(30);
	let display_data = {
		let mut new_img = RgbaImage::new(size.x, size.y);
		draw_filled_circle_mut(
			&mut new_img,
			(size / 2).as_ivec2().into(),
			10,
			Rgba(Srgba::WHITE.to_u8_array()),
		);
		new_img.to_vec()
	};
	cmd.trigger(NewBrush(BrushRef::Circle, size, display_data));
}

fn color_slider(
	query_panel: Query<(&ColorPanelChanger, &PickingInteraction)>,
	mut query_user: Query<(&ActionState<SettingsAction>, &mut PaintInk), With<MainUser>>,
) {
	let (action, mut paint) = query_user.single_mut();
	query_panel.iter().for_each(|(color_panel, inter)| {
		if inter == &PickingInteraction::None {
			return;
		}

		let is_pressing = (inter == &PickingInteraction::Pressed) as i32;
		let point_movement = action.axis_pair(&SettingsAction::Movement);
		let increase = action.just_pressed(&SettingsAction::Increase) as i32;
		let decrease = action.just_pressed(&SettingsAction::Decrease) as i32;
		let scroll = action.value(&SettingsAction::ScrollWheel);
		let Vec2 { x, y } = point_movement * Vec2::splat(is_pressing as f32);
		let put = x * is_pressing as f32 + scroll + increase as f32 - decrease as f32;
		if put == 0.0 && y == 0.0 {
			return;
		}

		let adjust = |ori: &mut f32| {
			*ori = (*ori + put / 255.0).clamp(0.0, 1.0);
		};
		let mut hsla: Hsla = paint.0.into();
		if hsla.hue.is_nan() || hsla.saturation.is_nan() || hsla.lightness.is_nan() {
			hsla = Hsla::default();
		}
		match color_panel {
			ColorPanelChanger::Red => adjust(&mut paint.0.red),
			ColorPanelChanger::Green => adjust(&mut paint.0.green),
			ColorPanelChanger::Blue => adjust(&mut paint.0.blue),
			ColorPanelChanger::Alpha => adjust(&mut paint.0.alpha),
			ColorPanelChanger::Hue => {
				hsla.hue = (hsla.hue + put).clamp(0.0, 359.0);
				paint.0 = hsla.into();
			},
			ColorPanelChanger::Saturation => {
				hsla.saturation = (hsla.saturation + put / 100.0).clamp(0.0, 1.0);
				paint.0 = hsla.into();
			},
			ColorPanelChanger::Lightness => {
				hsla.lightness = (hsla.lightness + put / 100.0).clamp(0.0, 1.0);
				paint.0 = hsla.into();
			},
			ColorPanelChanger::SatLight => {
				if x != 0.0 {
					hsla.saturation = (hsla.saturation + x / 100.0).clamp(0.0, 1.0);
				}
				if y != 0.0 {
					hsla.lightness = (hsla.lightness - y / 100.0).clamp(0.0, 1.0);
				}
				paint.0 = hsla.into();
			},
		};
	});
}

fn hex_color_text(
	mut query_hex: Query<&mut Text, With<HexColorText>>,
	query_user: Query<&PaintInk, (Changed<PaintInk>, With<MainUser>)>,
) {
	let (Ok(paint), Ok(mut text_hex)) = (query_user.get_single(), query_hex.get_single_mut())
	else {
		return;
	};
	text_hex.sections[1].value = paint.0.to_hex();
}

fn select_brush(
	mut on_click: EventReader<Pointer<Click>>,
	mut query_user: Query<&mut BrushRef, (With<MainUser>, Without<BrushChoice>)>,
	query_select_brush: Query<&BrushRef, With<BrushChoice>>,
) {
	let (Some(Ok(brush_ref_choice)), Ok(mut brush_ref)) = (
		on_click
			.read()
			.next()
			.map(|pointed| query_select_brush.get(pointed.target())),
		query_user.get_single_mut(),
	) else {
		return;
	};
	*brush_ref = brush_ref_choice.clone();
}

fn select_color(
	mut on_click: EventReader<Pointer<Click>>,
	mut query_user: Query<&mut PaintInk, With<MainUser>>,
	query_select_brush: Query<&BackgroundColor, With<ColorChoice>>,
) {
	for pointed in on_click.read() {
		let (Ok(picked_color), Ok(mut paint)) = (
			query_select_brush.get(pointed.target()),
			query_user.get_single_mut(),
		) else {
			continue;
		};
		if let PointerButton::Primary = pointed.button {
			paint.0 = picked_color.0.with_alpha(paint.0.alpha()).into();
		} else if let PointerButton::Secondary = pointed.button {
			paint.1 = picked_color.0.with_alpha(paint.1.alpha()).into();
		}
		break;
	}
}

fn new_brush_collection(
	trigger: Trigger<NewBrush>,
	query_brush_collector: Query<Entity, With<BrushCollector>>,
	mut cmd: Commands,
) {
	let Ok(ent_collector) = query_brush_collector.get_single() else {
		return;
	};
	let event = trigger.event();
	cmd.entity(ent_collector).with_child((
		EffectUIBundle::icon().button(),
		WithChild(IconBuild::new(event.2.clone(), event.1).extra_large()),
		BrushChoice,
		event.0.clone(),
	));
}
