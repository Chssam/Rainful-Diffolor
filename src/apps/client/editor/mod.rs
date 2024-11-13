mod lib;
mod ui_trait_fn;
mod view_ui;
use lib::*;
use ui_trait_fn::*;
use view_ui::*;

use std::{collections::HashSet, num::NonZero, path::Path};

use aery::edges::RelationCommands;
use arboard::Clipboard;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use image::{EncodableLayout, Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use leafwing_input_manager::prelude::*;
use rainful_diffolor::{embed_path, source_to_docs};
use sickle_ui::{prelude::*, widgets::inputs::slider::SliderAxis};
use strum::IntoEnumIterator;

use crate::{
	tool_tip::lib::{ToolTipContent, ToolsInfo},
	trait_bevy::*,
};

use super::*;

pub(super) struct ClientEditorPlugin;
impl Plugin for ClientEditorPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<NewBrush>().add_systems(
			Update,
			(
				update_ui_content,
				display_color_update,
				update_color_image,
				color_slider,
				obj_controller_update,
				display_directory_object,
				hex_color_text,
				select_brush,
				select_color,
				opting_permission,
			)
				.run_if(in_state(RdioClientState::Online)),
		);

		// #[cfg(target_os = "android")]
		// app.add_systems(First, mobile_action);
	}
}

pub(super) fn editor_ui(
	mut editors: EditorRdio,
	main_user_query: Query<(Entity, &RGBAhsl), With<MainUser>>,
	query_main_root: Query<Entity, With<UiMainRootNode>>,
) {
	// Added 23/10/2024
	editors
		.cmd
		.ui_builder(query_main_root.single())
		.floating_panel(
			FloatingPanelConfig {
				title: Some("Object Permission".to_owned()),
				closable: false,
				folded: true,
				..default()
			},
			FloatingPanelLayout {
				size: Vec2::splat(200.),
				position: Some(Vec2::new(200., 10.)),
				droppable: false,
				..default()
			},
			|floaty| {
				floaty.tab_container(|tab_con| {
					tab_con.add_tab("Permission".to_owned(), |tab| {
						tab.scroll_view(None, |view| {
							view.style().flex_direction(FlexDirection::Row);
							view.dropdown(vec!["None", "All Except", "All", "Only"], None)
								.insert(OptingPer);
							view.row(|row| {
								row.column(|column| {
									column
										.insert(FiltedOutPer)
										.style()
										.width(Val::Percent(50.0));
								});
								row.column(|column| {
									column.insert(FiltedInPer).style().width(Val::Percent(50.0));
								});
							});
						});
					});

					// tab_con.add_tab("View".to_owned(), |tab| {
					// 	tab.scroll_view(None, |view| {
					// 		view.style().flex_direction(FlexDirection::Row);
					// 		view.dropdown(
					// 			vec![
					// 				"None",
					// 				"All Except",
					// 				"All",
					// 				"Only",
					// 			],
					// 			None,
					// 		).insert(OptingView);
					// 		view.column(|column| {
					// 			column.insert(FiltedOutView);
					// 		});
					// 		view.column(|column| {
					// 			column.insert(FiltedInView);
					// 		});
					// 	});
					// });
				});
			},
		);
	let (main_user_ent, color_handle) = main_user_query.single();
	let path_icon = Path::new("Icon");

	let mut bar_1 = editors.bar_1();
	bar_1.menu(
		MenuConfig {
			name: "File".to_owned(),
			..default()
		},
		|menu| {
			use ToolsStandAlone::*;
			for action in [
				Save,
				ExportImage,
				ExportCanvas,
				ExportSvgRelative,
				ExportSvgAbsolute,
			] {
				menu.menu_item(MenuItemConfig {
					name: action.as_reflect().tool_name(),
					..default()
				})
				.insert(instant_action(main_user_ent, action));
			}
		},
	);
	bar_1.menu(
		MenuConfig {
			name: "Edit".to_owned(),
			..default()
		},
		|menu| {
			use ToolsStandAlone::*;
			for action in [Undo, Redo, Cut, Copy, Paste, Preference] {
				menu.menu_item(MenuItemConfig {
					name: action.as_reflect().tool_name(),
					..default()
				})
				.insert(instant_action(main_user_ent, action));
			}
		},
	);
	bar_1.menu(
		MenuConfig {
			name: "View".to_owned(),
			..default()
		},
		|menu| {
			let grider = VisualGrid(false);
			menu.toggle_menu_item(ToggleMenuItemConfig {
				name: grider.as_reflect().tool_name(),
				..default()
			})
			.insert((
				ToolTipContent::new(grider),
				On::<Pointer<Click>>::run(
					|mut query_user: Query<&mut VisualGrid, With<MainUser>>| {
						let Ok(mut visual_grid) = query_user.get_single_mut() else {
							return;
						};
						visual_grid.0 = !visual_grid.0;
					},
				),
			));
		},
	);
	bar_1.menu(
		MenuConfig {
			name: "Action".to_owned(),
			..default()
		},
		|menu| {
			for action in VerifyAction::iter() {
				let mut ui_builder = menu.menu_item(MenuItemConfig {
					name: action.as_reflect().tool_name(),
					..default()
				});
				ui_builder.insert(ToolTipContent::new(action));
				if let VerifyAction::PathApplyColor = &action {
					ui_builder.insert(toggle_action(main_user_ent, action));
				} else {
					ui_builder.insert(instant_action(main_user_ent, action));
				}
			}
		},
	);

	// MARK: Second Root
	let mut bar_2 = editors.bar_2();
	bar_2.menu(
		MenuConfig {
			name: "File".to_owned(),
			..default()
		},
		|menu| {
			use ToolsStandAlone::*;
			for action in [Save, ExportImage, ExportCanvas] {
				menu.menu_item(MenuItemConfig {
					name: action.as_reflect().tool_name(),
					..default()
				})
				.insert(instant_action(main_user_ent, action));
			}
		},
	);
	bar_2.menu(
		MenuConfig {
			name: "Path".to_owned(),
			..default()
		},
		|menu| {
			for action in RenderPathAs::iter() {
				menu.menu_item(MenuItemConfig {
					name: action.as_reflect().tool_name(),
					..default()
				})
				.insert(On::<Pointer<Click>>::run(
					move |query_user: Query<&SelectedObject, With<MainUser>>,
					      mut query_path: Query<&mut RenderPathAs, With<ObjectWorld>>,
					      query_point: Query<&Parent, With<ObjectWorld>>| {
						let Ok(selected_obj) = query_user.get_single() else {
							return;
						};
						let all_path = query_point
							.iter_many(selected_obj.group.iter())
							.map(|parent| parent.get())
							.collect::<HashSet<Entity>>();
						let mut path_query = query_path.iter_many_mut(all_path.iter());
						while let Some(mut render_as) = path_query.fetch_next() {
							render_as.set_if_neq(action);
						}
					},
				));
			}
			let action = VerifyAction::ToggleClose;
			menu.menu_item(MenuItemConfig {
				name: action.as_reflect().tool_name(),
				..default()
			})
			.insert((
				ToolTipContent::new(action),
				instant_action(main_user_ent, action),
			));
		},
	);

	editors
		.left_top_panel()
		.scroll_view(Some(ScrollAxis::Vertical), |view| {
			view.row(|row| {
				row.style().flex_wrap(FlexWrap::Wrap);
				for action in EditorTools::iter() {
					row.large_icon(ImageSource::embed_path(
						&path_icon.join(action.as_reflect().path_img()),
					))
					.insert((
						instant_action(main_user_ent, action),
						ToolTipContent::new(action),
					));
				}
			});
		});

	let mut left_bottom_panel = editors.left_bottom_panel();
	left_bottom_panel.add_tab("Tool Options".to_owned(), |tab| {
		tab.scroll_view(Some(ScrollAxis::Vertical), |view| {
			view.spawn((
				TextBundle::from_section(
					"Draw Type: Normal",
					TextStyle {
						font_size: FontTypeSize::NAME,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
				Pickable::default(),
				On::<Pointer<Click>>::run(
					|event: Listener<Pointer<Click>>,
					 mut query_text: Query<&mut Text>,
					 mut query_user: Query<&mut DrawType, With<MainUser>>| {
						let Ok(mut draw_type) = query_user.get_single_mut() else {
							return;
						};
						if event.button != PointerButton::Primary {
							return;
						}
						*draw_type = match *draw_type {
							DrawType::Normal => DrawType::Behind,
							DrawType::Behind => DrawType::Normal,
							_ => unreachable!(),
						};

						let mut text = query_text.get_mut(event.target()).unwrap();
						text.sections[0].value =
							format!("Draw Type: {}", draw_type.as_reflect().tool_name());
					},
				),
			));
			view.spawn((
				TextBundle::from_section(
					"Scale: 1",
					TextStyle {
						font_size: FontTypeSize::NAME,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
				Pickable::default(),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text>,
					 mut query_user: Query<&mut BrushScale, With<MainUser>>| {
						let Ok(mut brush_scale) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.get_mut(event.target()).unwrap();
						brush_scale.add(event.delta.x as i8);
						text.sections[0].value = format!("Scale: {}", brush_scale.get());
					},
				),
			));

			view.spawn((
				TextBundle::from_section(
					"Spacing: 1",
					TextStyle {
						font_size: FontTypeSize::NAME,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
				Pickable::default(),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text>,
					 mut query_user: Query<&mut DrawingSpacing, With<MainUser>>| {
						let Ok(mut draw_spacing) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.get_mut(event.target()).unwrap();
						let calculate = (draw_spacing.get() as i16 + event.delta.x as i16)
							.clamp(1, u8::MAX as i16) as u8;
						let Some(non_zero) = NonZero::new(calculate) else {
							return;
						};
						**draw_spacing = non_zero;
						text.sections[0].value = format!("Spacing: {}", draw_spacing.get());
					},
				),
			));

			view.spawn((
				TextBundle::from_section(
					"Blur: 0",
					TextStyle {
						font_size: FontTypeSize::NAME,
						color: Srgba::BEVY_WHITE,
						..default()
					},
				),
				Pickable::default(),
				On::<Pointer<Drag>>::run(
					|event: Listener<Pointer<Drag>>,
					 mut query_text: Query<&mut Text>,
					 mut query_user: Query<&mut BlurScale, With<MainUser>>| {
						let Ok(mut blur_scale) = query_user.get_single_mut() else {
							return;
						};
						let mut text = query_text.get_mut(event.target()).unwrap();

						blur_scale.0 = (blur_scale.0 * 100.0 + event.delta.x).round() / 100.0;
						text.sections[0].value = format!("Blur: {}", blur_scale.0);
					},
				),
			));
		});
	});

	editors.left_panel().row(|row| {
		for comp in [DisplayColor::Foreground, DisplayColor::Background] {
			row.spawn((
				ButtonBundle {
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
				instant_action(main_user_ent, ToolsStandAlone::ColorSwap),
				comp,
			));
		}
	});

	editors.middle_panel().spawn((
		TextBundle::from_section(
			"Cord: -",
			TextStyle {
				font_size: FontTypeSize::NAME,
				color: Srgba::BEVY_WHITE,
				..default()
			},
		),
		EditorInfoComp,
		Pickable::IGNORE,
	));

	editors
		.right_top_panel()
		.add_tab("Brush".to_owned(), |tab| {
			tab.scroll_view(Some(ScrollAxis::Vertical), |view| {
				view.style().flex_direction(FlexDirection::Row);
				view.row(|row| {
					row.insert(BrushCollector)
						.style()
						.align_items(AlignItems::FlexStart)
						.flex_wrap(FlexWrap::Wrap)
						.flex_grow(1.0)
						.flex_shrink(0.0)
						.background_color(Color::WHITE);
				});
			});
		});

	let mut right_bottom = editors.right_bottom_panel();
	right_bottom.add_tab("Layers".to_owned(), |tab| {
		tab.slider(SliderConfig {
			label: Some("Opacity".to_owned()),
			min: 0.0,
			max: 100.0,
			initial_value: 100.0,
			show_current: true,
			axis: SliderAxis::Horizontal,
		});
		tab.row(|row| {
			row.medium_icon(ImageSource::embed_path(&path_icon.join("gimp-visible.png")))
				.insert(ToolTipContent::new("View"));
			row.medium_icon(ImageSource::embed_path(
				&path_icon.join("gimp-tool-move.png"),
			))
			.insert(ToolTipContent::new("Move Lock"));
			row.label(LabelConfig {
				label: "Object Name".to_owned(),
				..default()
			});
		});
		tab.scroll_view(None, |view| {
			view.insert(ObjInfoController)
				.style()
				.flex_direction(FlexDirection::ColumnReverse);
		});
		tab.row(|row| {
			let new_img = VerifyAction::NewImage;
			row.large_icon(ImageSource::embed_path(
				&path_icon.join(new_img.as_reflect().path_img()),
			))
			.insert(instant_action(main_user_ent, new_img));

			let main_only = |action: ObjectActionNet| {
				On::<Pointer<Click>>::run(
					move |mut client: ResMut<ClientConnectionManager>,
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
			for action in [ObjectActionNet::LayerUp, ObjectActionNet::LayerDown] {
				row.large_icon(ImageSource::embed_path(
					&path_icon.join(action.as_reflect().path_img()),
				))
				.insert(main_only(action));
			}
		});
	});

	let mut bottom_left = editors.bottom_left_panel();
	bottom_left.add_tab("Colors".to_owned(), |tab| {
		tab.scroll_view(None, |view| {
			view.spawn((
				TextBundle::from_sections([
					TextSection::new(
						"Hex: ",
						TextStyle {
							font_size: FontTypeSize::NAME,
							color: Srgba::BEVY_WHITE,
							..default()
						},
					),
					TextSection::new(
						"",
						TextStyle {
							font_size: FontTypeSize::NAME,
							color: Srgba::BEVY_WHITE,
							..default()
						},
					),
				])
				.with_no_wrap()
				.with_style(Style {
					flex_grow: 0.0,
					flex_shrink: 1.0,
					..default()
				}),
				Pickable::default(),
				HexColorText,
				ToolTipContent::new(
					"Left Click: Copy hex color code\nRight Click: Paste hex color code",
				),
				On::<Pointer<Click>>::run(
					|event: Listener<Pointer<Click>>,
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
			));
			use ColorPanelChanger::*;
			for comp in ColorPanelChanger::iter() {
				let (handle_img, width, height) = match comp {
					Red => (color_handle.red.clone_weak(), 256.0, Val::Auto),
					Green => (color_handle.green.clone_weak(), 256.0, Val::Auto),
					Blue => (color_handle.blue.clone_weak(), 256.0, Val::Auto),
					Alpha => (color_handle.alpha.clone_weak(), 256.0, Val::Auto),
					Hue => (color_handle.hue.clone_weak(), 361.0, Val::Auto),
					Saturation => (color_handle.saturation.clone_weak(), 202.0, Val::Auto),
					Lightness => (color_handle.lightness.clone_weak(), 202.0, Val::Auto),
					SatLight => (color_handle.sat_light.clone_weak(), 202.0, Val::Px(202.0)),
				};
				let mut comp_name = comp.as_reflect().tool_name();
				comp_name.push_str(": ");
				view.spawn((
					ButtonBundle {
						style: Style {
							width: Val::Px(width),
							height,
							..default()
						},
						image: handle_img.into(),
						..default()
					},
					LockableOnUI,
					comp,
				))
				.spawn((
					TextBundle::from_sections([
						TextSection::new(
							comp_name,
							TextStyle {
								font_size: FontTypeSize::NAME,
								color: Srgba::BEVY_WHITE,
								..default()
							},
						),
						TextSection::new(
							"0",
							TextStyle {
								font_size: FontTypeSize::NAME,
								color: Srgba::BEVY_WHITE,
								..default()
							},
						),
					]),
					comp,
				));
			}
		});
	});
	bottom_left.add_tab("Palettes".to_owned(), |tab| {
		tab.scroll_view(None, |view| {
			view.label(LabelConfig {
				label: "Not Ready".to_owned(),
				..default()
			});
		});
	});
	bottom_left.add_tab("Palette Editor".to_owned(), |tab| {
		tab.scroll_view(Some(ScrollAxis::Vertical), |view| {
			view.style().flex_direction(FlexDirection::Row);
			view.row(|row| {
				row.style().flex_wrap(FlexWrap::Wrap);
				let mut color_node = |color: Srgba| {
					row.spawn((
						NodeBundle {
							style: Style {
								border: UiRect::all(Val::Px(1.0)),
								min_width: Val::Px(20.0),
								min_height: Val::Px(20.0),
								flex_basis: Val::Px(1.0),
								..default()
							},
							background_color: color.into(),
							border_color: Color::BLACK.into(),
							..default()
						},
						ColorChoice,
					));
				};
				for color in Srgba::iter_basic() {
					color_node(color);
				}

				for color in Srgba::iter_css() {
					color_node(color);
				}

				for color in Srgba::iter_tailwind() {
					color_node(color);
				}
			});
		});
	});

	let mut bottom_right = editors.bottom_right_panel();
	bottom_right.add_tab("Directory".to_owned(), |tab| {
		tab.scroll_view(None, |view| {
			view.insert(DisplayObjectDirectory);
			view.row(|row| {
				#[derive(Component)]
				struct PathModeText;
				row.spawn((
					TextBundle::from_sections([
						TextSection::new(
							"Drop Path Mode: ",
							TextStyle {
								font_size: FontTypeSize::NAME,
								..default()
							},
						),
						TextSection::new(
							DropPathMode::AsObject.as_reflect().tool_name(),
							TextStyle {
								font_size: FontTypeSize::NAME,
								..default()
							},
						),
						TextSection::new(
							" | Entity Name : Directory",
							TextStyle {
								font_size: FontTypeSize::NAME,
								..default()
							},
						),
					])
					.with_style(Style {
						padding: UiRect::right(Val::Px(10.0)),
						..default()
					}),
					Pickable::default(),
					PathModeText,
					ToolTipContent::new("Drop mode: Load object <-> Save location for object"),
					On::<Pointer<Click>>::run(
						|state: Res<State<DropPathMode>>,
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
				));
			});
		});
	});
}

// Time limit; Random Code Happen
fn opting_permission(
	mut cmd: Commands,
	mut client: ResMut<ConnectionManager>,
	mut query_select: Query<&mut Dropdown, With<OptingPer>>,
	query_filted_in: Query<(Entity, Option<&Children>), With<FiltedInPer>>,
	query_filted_out: Query<(Entity, Option<&Children>), With<FiltedOutPer>>,
	query_obj: Query<(Entity, &ObjectAccess), With<ObjectWorld>>,
	query_access_change: Query<Entity, (Changed<ObjectAccess>, With<ObjectWorld>)>,
	query_user: Query<Ref<SelectedObject>, With<MainUser>>,
	query_user_ids: Query<(&UserId, &SharingName)>,
) {
	let Ok(mut drop_down) = query_select.get_single_mut() else {
		return;
	};
	let filted_in = query_filted_in.single();
	let filted_out = query_filted_out.single();
	let Ok(selected_obj) = query_user.get_single() else {
		return;
	};
	let Some(Ok((obj_ent, access))) = selected_obj.single.map(|single| query_obj.get(single))
	else {
		return;
	};
	let mut send = true;
	if selected_obj.is_changed() || query_access_change.iter().any(|ent| ent == obj_ent) {
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
			NetworkTarget::None => (0, (vec![], vec![])),
			NetworkTarget::AllExcept(vec) => (1, filter(vec)),
			NetworkTarget::All => (2, (vec![], vec![])),
			NetworkTarget::Only(vec) => (3, filter(vec)),
			_ => return,
		};
		drop_down.set_value(value);
		send = false;
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
					TextBundle::from_section(
						name.0.clone(),
						TextStyle {
							font_size: FontTypeSize::NAME,
							..default()
						},
					),
					Pickable::default(),
					On::<Pointer<Click>>::run(move |mut client: ResMut<ConnectionManager>| {
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
					TextBundle::from_section(
						name.0.clone(),
						TextStyle {
							font_size: FontTypeSize::NAME,
							..default()
						},
					),
					Pickable::default(),
					On::<Pointer<Click>>::run(move |mut client: ResMut<ConnectionManager>| {
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

	if drop_down.is_changed() && !selected_obj.is_changed() && send {
		let Some(in_drop) = drop_down.value() else {
			return;
		};
		let target = match in_drop {
			0 => NetworkTarget::None,
			1 => NetworkTarget::AllExcept(vec![]),
			2 => NetworkTarget::All,
			3 => NetworkTarget::Only(vec![]),
			_ => return,
		};
		let _ = client.send_message_to_target::<MainChannel, PerActionNet>(
			&mut PerActionNet {
				obj_ent,
				action: PerAction::Change(target),
			},
			NetworkTarget::All,
		);
	}
}

#[derive(Component)]
pub struct OwnerDisplay;

#[derive(Component)]
pub struct OptingPer;

#[derive(Component)]
pub struct FiltedOutPer;

#[derive(Component)]
pub struct FiltedInPer;

#[derive(Component)]
pub struct OptingView;

#[derive(Component)]
pub struct FiltedOutView;

#[derive(Component)]
pub struct FiltedInView;

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
	let mut directoryer = cmd.ui_builder(ent_directory);
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
		directoryer.spawn(TextBundle::from_section(
			label,
			TextStyle {
				font_size: FontTypeSize::NAME,
				..default()
			},
		));
	}
}

// #[cfg(target_os = "android")]
// fn mobile_action(
// 	i_touch: Res<Touches>,
// 	mut e_touch: EventReader<TouchInput>,
// 	mut actions_key: ResMut<ActionState<UIAction>>,
// ) {
// 	i_touch
// 		.any_just_pressed()
// 		.then(|| actions_key.press(&UIAction::Primary));
// 	e_touch.read().for_each(|e_input| {
// 		// e_input
// 		// actions_key.set_action_data(action, data)
// 	});
// }

// Toggle Menu blocked by other entity
fn obj_controller_update(
	mut cmd: Commands,
	obj_control_query: Query<Entity, With<ObjInfoController>>,
	query_obj: Query<(Entity, &SharingName, Has<ObjectImage>), Added<ObjectWorld>>,
) {
	let ent_controller = obj_control_query.single();
	let mut controller = cmd.ui_builder(ent_controller);
	query_obj.iter().for_each(|(ent_obj, obj_name, is_img)| {
		controller.row(|row| {
			row.entity_commands().set::<ObjectRelationUI>(ent_obj);

			row.checkbox(None, true)
				.insert(AttactObject::Visibility.target(ent_obj));
			row.checkbox(None, false)
				.insert(ObjectActionNet::LockMove.target(ent_obj));

			row.menu(
				MenuConfig {
					name: obj_name.0.clone(),
					..default()
				},
				|menu| {
					let pick = AttactObject::Pick;
					menu.menu_item(MenuItemConfig {
						name: pick.as_reflect().tool_name(),
						..default()
					})
					.insert(pick.target(ent_obj));

					let lock_pick = AttactObject::LockPick;
					menu.menu_item(MenuItemConfig {
						name: lock_pick.as_reflect().tool_name(),
						..default()
					})
					.insert(lock_pick.target(ent_obj));

					if is_img {
						for action in [ObjectActionNet::LockAlpha, ObjectActionNet::LockPixel] {
							menu.menu_item(MenuItemConfig {
								name: action.as_reflect().tool_name(),
								..default()
							})
							.insert(action.target(ent_obj));
						}
					}

					// let lock_pick = AttactObject::LockPick;
					// menu.toggle_menu_item(ToggleMenuItemConfig {
					// 	name: lock_pick.as_reflect().tool_name(),
					// 	..default()
					// })
					// .insert(lock_pick.target(ent_obj));

					// if is_img {
					// 	for action in [ObjectActionNet::LockAlpha, ObjectActionNet::LockPixel] {
					// 		menu.toggle_menu_item(ToggleMenuItemConfig {
					// 			name: action.as_reflect().tool_name(),
					// 			..default()
					// 		})
					// 		.insert(action.target(ent_obj));
					// 	}
					// }
				},
			);
		});
	});
}

pub(super) fn add_brush(mut cmd: Commands) {
	let create_data = |data: Vec<u8>| {
		data.into_iter()
			.map(|v| [0, 0, 0, v])
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
			Rgba(Srgba::BLACK.to_u8_array()),
		);
		new_img.to_vec()
	};
	cmd.trigger(NewBrush(BrushRef::Circle, size, display_data));
}

fn color_slider(
	query_panel: Query<(&ColorPanelChanger, &Interaction)>,
	mut query_user: Query<(&ActionState<SettingsAction>, &mut PaintInk), With<MainUser>>,
) {
	let (action, mut paint) = query_user.single_mut();
	query_panel.iter().for_each(|(color_panel, inter)| {
		if inter == &Interaction::None {
			return;
		}

		let is_pressing = (inter == &Interaction::Pressed) as i32;
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
	mut on_click: EventReader<Pointer<Down>>,
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
	mut on_click: EventReader<Pointer<Down>>,
	mut query_user: Query<&mut PaintInk, With<MainUser>>,
	query_select_brush: Query<&BackgroundColor, With<ColorChoice>>,
) {
	let Some(pointed) = on_click.read().next() else {
		return;
	};
	let (Ok(picked_color), Ok(mut paint)) = (
		query_select_brush.get(pointed.target()),
		query_user.get_single_mut(),
	) else {
		return;
	};
	if let PointerButton::Primary = pointed.button {
		paint.0 = picked_color.0.with_alpha(paint.0.alpha()).into();
	} else if let PointerButton::Secondary = pointed.button {
		paint.1 = picked_color.0.with_alpha(paint.1.alpha()).into();
	}
}

pub(super) fn new_brush_collection(
	trigger: Trigger<NewBrush>,
	query_brush_collector: Query<Entity, With<BrushCollector>>,
	mut cmd: Commands,
	mut img_asset: ResMut<Assets<Image>>,
) {
	let Ok(ent_collector) = query_brush_collector.get_single() else {
		return;
	};
	let event = trigger.event();
	cmd.ui_builder(ent_collector)
		.extra_large_icon(ImageSource::Handle(
			img_asset.rgba8_image(event.2.clone(), event.1),
		))
		.insert((BrushChoice, event.0.clone()))
		.style()
		.padding(UiRect::all(Val::Px(1.5)));
}

trait EmbedPath {
	fn embed_path(path: &Path) -> Self;
}

impl EmbedPath for ImageSource {
	fn embed_path(path: &Path) -> Self {
		Self::Path(embed_path(path))
	}
}
