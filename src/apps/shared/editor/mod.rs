use arboard::Clipboard;
use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*,
	window::{CursorGrabMode, PrimaryWindow},
};
use bevy_cosmic_edit::*;
use bevy_mod_picking::prelude::{On, *};
use cosmic_text::{Attrs, Edit as _, Family, Metrics};
use i_cant_believe_its_not_bsn::WithChild;
use lightyear::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use sickle_ui::{prelude::*, widgets::layout::sized_zone::SizedZoneResizeHandleContainer};

pub mod lib;
mod one_shot;
use lib::*;
use one_shot::*;

use super::*;
use crate::{tool_tip::lib::ToolTipContent, trait_bevy::*};

pub(super) struct EditorPlugin;
impl Plugin for EditorPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<EditorParts>()
			.init_resource::<ConnectTokenRequestTask>()
			.init_state::<RdioClientState>()
			.enable_state_scoped_entities::<RdioClientState>()
			.init_state::<RdioServerState>()
			.enable_state_scoped_entities::<RdioServerState>()
			.observe(disable_pick_component)
			.observe(receive_display_msg)
			.add_systems(PreStartup, editor_ui)
			.add_systems(
				Update,
				(
					handle_local_message,
					change_active_editor_ui,
					deselect_editor_on_esc,
					toggle_editor.run_if(input_just_pressed(KeyCode::F8)),
				),
			)
			.add_systems(PostUpdate, lock_cursor);
	}
}

fn toggle_editor(mut query_main_node: Query<&mut Visibility, With<UiMainRootNode>>) {
	let Ok(mut visibility) = query_main_node.get_single_mut() else {
		return;
	};
	*visibility = match *visibility {
		Visibility::Hidden => Visibility::Inherited,
		_ => Visibility::Hidden,
	};
}

fn change_active_editor_ui(
	mut on_click: EventReader<Pointer<Down>>,
	mut commands: Commands,
	interaction_query: Query<(Option<&CosmicSource>, Has<CosmicBuffer>), Without<ReadOnly>>,
) {
	let not_empty = !on_click.is_empty();
	let no_process = on_click.read().next().is_some_and(|pointed| {
		if pointed.button != PointerButton::Primary {
			return true;
		}
		let targeted = pointed.target();
		let Ok((op_source, is_cosmic)) = interaction_query.get(targeted) else {
			return true;
		};
		!is_cosmic
			.then_some(targeted)
			.or(op_source.map(|source| source.0))
			.is_some_and(|ent_cosmic| {
				commands.insert_resource(FocusedWidget(Some(ent_cosmic)));
				true
			})
	});
	if not_empty && no_process {
		commands.insert_resource(FocusedWidget(None));
	}
}

fn disable_pick_component(
	trigger: Trigger<
		OnAdd,
		(
			Text,
			SizedZoneResizeHandleContainer,
			ResizeHandle,
			ResizeHandles,
		),
	>,
	query_no_pick: Query<Entity, Without<Pickable>>,
	mut cmd: Commands,
) {
	let ent = trigger.entity();
	if query_no_pick.contains(ent) {
		cmd.entity(ent).insert(Pickable::IGNORE);
	}
}

fn editor_ui(
	mut cmd: Commands,
	mut editor_part: ResMut<EditorParts>,
	mut font_system: ResMut<CosmicFontSystem>,
) {
	use EditorPosition::*;
	cmd.ui_builder(UiRoot).container(
		(
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					flex_direction: FlexDirection::Column,
					justify_content: JustifyContent::SpaceBetween,
					..default()
				},
				..default()
			},
			UiMainRootNode,
			Pickable::IGNORE,
		),
		|container| {
			// MARK: First Root
			container.column(|column| {
				column
					.style()
					.height(Val::Auto)
					.width(Val::Percent(100.0))
					.flex_grow(0.0)
					.background_color(Srgba::BEVY_BLACK);

				column.menu_bar(|menu_bar| {
					editor_part.insert(MenuBar1, menu_bar.id());
					menu_bar.menu(
						MenuConfig {
							name: "Apps".to_owned(),
							..default()
						},
						|menu| {
							menu.menu_item(MenuItemConfig {
								name: "Exit".to_owned(),
								..default()
							})
							.insert(On::<Pointer<Click>>::run(
								|mut write_events: EventWriter<AppExit>| {
									write_events.send(AppExit::Success);
								},
							));

							menu.menu_item(MenuItemConfig {
								name: "Github".to_owned(),
								..default()
							})
							.insert((
								ToolTipContent::new("Paste link in searcher to view Source"),
								On::<Pointer<Click>>::run(|| {
									let mut clip_board = Clipboard::new().unwrap();
									let link = "https://github.com/Chssam/Rainful-Diffolor";
									clip_board.set_text(link).unwrap();
								}),
							));

							menu.menu_item(MenuItemConfig {
								name: "About".to_owned(),
								..default()
							})
							.insert(ToolTipContent::new("This button does nothing"));
						},
					);
				});
			});

			// MARK: Second Root
			container.column(|column| {
				column
					.style()
					.height(Val::Auto)
					.width(Val::Percent(100.0))
					.flex_grow(0.0)
					.background_color(Srgba::BEVY_BLACK);

				column.menu_bar(|menu_bar| {
					editor_part.insert(MenuBar2, menu_bar.id());
				});
			});

			let dock_zone_size = 100.0;
			let dock_size = dock_zone_size / 1.5;
			let do_size = dock_size / 1.5;

			// MARK: Middle ROOT
			container.sized_zone(
				SizedZoneConfig {
					size: dock_zone_size * 4.0,
					min_size: dock_zone_size,
				},
				|sized| {
					sized
						.insert(Pickable::IGNORE)
						.style()
						.flex_direction(FlexDirection::Row)
						.height(Val::Auto)
						.width(Val::Percent(100.0))
						.background_color(Srgba::NONE.into())
						.lock_attribute(LockableStyleAttribute::BackgroundColor);

					// MARK: LEFT
					sized.sized_zone(
						SizedZoneConfig {
							size: dock_size,
							min_size: dock_size,
						},
						|sized| {
							editor_part.insert(PanelLeft, sized.id());
							sized.style().overflow(Overflow::clip());
							sized.sized_zone(
								SizedZoneConfig {
									size: do_size,
									min_size: do_size,
								},
								|sized| {
									editor_part.insert(PanelLeftTop, sized.id());
								},
							);
							sized.docking_zone(
								SizedZoneConfig {
									size: do_size,
									min_size: do_size,
								},
								false,
								|dock| {
									editor_part.insert(PanelLeftBottom, dock.id());
								},
							);
						},
					);

					// MARK: Active World View Zone (MIDDLE)
					sized.sized_zone(
						SizedZoneConfig {
							size: 100.0,
							min_size: 10.0,
						},
						|sized| {
							editor_part.insert(PanelMiddle, sized.id());
							sized
								.insert(Pickable::IGNORE)
								.style()
								.overflow(Overflow::clip())
								.justify_content(JustifyContent::FlexEnd)
								.background_color(Color::NONE)
								.lock_attribute(LockableStyleAttribute::BackgroundColor);
						},
					);

					// MARK: Option (RIGHT)
					sized.sized_zone(
						SizedZoneConfig {
							size: dock_size,
							min_size: dock_size,
						},
						|sized| {
							editor_part.insert(PanelRight, sized.id());
							sized.style().overflow(Overflow::clip());

							sized.docking_zone(
								SizedZoneConfig {
									size: do_size,
									min_size: do_size,
								},
								false,
								|dock| {
									editor_part.insert(PanelRightTop, dock.id());
								},
							);
							sized.docking_zone(
								SizedZoneConfig {
									size: do_size,
									min_size: do_size,
								},
								false,
								|dock| {
									editor_part.insert(PanelRightBottom, dock.id());
								},
							);
						},
					);
				},
			);

			// MARK: Bottom Panel
			container.sized_zone(
				SizedZoneConfig {
					size: dock_zone_size,
					min_size: dock_zone_size,
				},
				|sized| {
					editor_part.insert(PanelBottom, sized.id());
					sized
						.style()
						.width(Val::Percent(100.0))
						.background_color(Srgba::BEVY_BLACK);

					sized.docking_zone(
						SizedZoneConfig {
							size: do_size,
							min_size: do_size,
						},
						false,
						|dock| {
							editor_part.insert(PanelBottomLeft, dock.id());
						},
					);
					sized.docking_zone(
						SizedZoneConfig {
							size: do_size,
							min_size: do_size,
						},
						false,
						|dock| {
							editor_part.insert(PanelBottomRight, dock.id());

							dock.add_tab("Connection".to_owned(), |tab| {
								let ip = CLIENT_ADDR.ip();
								let client_port = CLIENT_ADDR.port();
								let server_port = SERVER_ADDR.port();
								tab.menu(
									MenuConfig {
										name: "Options".to_owned(),
										..default()
									},
									|menu| {
										menu.menu_item(MenuItemConfig {
											name: "Private".to_owned(),
											..default()
										})
										.insert(On::<Pointer<Click>>::run(select_ip_self));
										menu.menu_item(MenuItemConfig {
											name: "Local".to_owned(),
											..default()
										})
										.insert(On::<Pointer<Click>>::run(select_ip_local));
										menu.menu_item(MenuItemConfig {
											name: "Client Toggle".to_owned(),
											..default()
										})
										.insert(On::<Pointer<Click>>::run(client_system));
										menu.menu_item(MenuItemConfig {
											name: "Server Toggle".to_owned(),
											..default()
										})
										.insert(On::<Pointer<Click>>::run(server_system));
									},
								);
								let name = rand::thread_rng()
									.sample_iter(&Alphanumeric)
									.take(8)
									.map(char::from)
									.collect();
								input_node(
									tab,
									"Name".to_owned(),
									name,
									NameInput,
									&mut font_system,
								);
								input_node(
									tab,
									"IP".to_owned(),
									ip.to_string(),
									ConnectionIP,
									&mut font_system,
								);
								input_node(
									tab,
									"Client Port".to_owned(),
									client_port.to_string(),
									ClientPort,
									&mut font_system,
								);
								input_node(
									tab,
									"Server Port".to_owned(),
									server_port.to_string(),
									ServerPort,
									&mut font_system,
								);
							});

							dock.add_tab("Messages".to_owned(), |tab| {
								tab.style().overflow(Overflow::clip());
								let font_size = FontTypeSize::NAME;
								let y = font_size + 4.0;
								let attrs = Attrs::new()
									.family(Family::SansSerif)
									.color(Srgba::BEVY_BLACK.to_cosmic());
								let target = tab
									.spawn((
										CosmicEditBundle {
											buffer: CosmicBuffer::new(
												&mut font_system,
												Metrics::new(font_size, font_size),
											)
											.with_text(&mut font_system, "", attrs),
											fill_color: CosmicBackgroundColor(Color::NONE),
											sprite_bundle: SpriteBundle {
												visibility: Visibility::Hidden,
												..default()
											},
											max_chars: MaxChars(255),
											max_lines: MaxLines(1),
											text_position: CosmicTextAlign::Left { padding: 2 },
											x_offset: XOffset {
												left: -5.0,
												width: 1.0,
											},
											..default()
										},
										Placeholder::new(
											"<Chat>",
											Attrs::new()
												.family(Family::SansSerif)
												.color(Srgba::BEVY_WHITE.darker(0.3).to_cosmic()),
										),
										TypeChat,
									))
									.id();
								tab.spawn((
									ButtonBundle {
										style: Style {
											width: Val::Percent(100.0),
											height: Val::Px(y + 10.0),
											border: UiRect::all(Val::Px(2.5)),
											padding: UiRect::all(Val::Px(5.0)),
											overflow: Overflow::clip(),
											..default()
										},
										border_color: Srgba::BEVY_BLACK.into(),
										background_color: Srgba::BEVY_WHITE.into(),
										..default()
									},
									CosmicSource(target),
								));

								tab.scroll_view(Some(ScrollAxis::Vertical), |view| {
									view.insert(ChatMessageHolder)
										.style()
										.overflow(Overflow::clip())
										.border(UiRect::all(Val::Px(2.5)))
										.border_color(Srgba::BEVY_BLACK);

									view.spawn(TextBundle::from_section(
										"Beginning Of Message".to_owned(),
										TextStyle {
											font_size: FontTypeSize::NAME,
											color: Srgba::BEVY_WHITE,
											..default()
										},
									));
								});
							});
						},
					);
				},
			);
		},
	);
}

fn input_node<T: Component>(
	tab: &mut UiBuilder<Entity>,
	label: String,
	value: String,
	comp: T,
	font_system: &mut CosmicFontSystem,
) {
	tab.row(|row| {
		let font_size = FontTypeSize::NAME;
		let y = font_size + 4.0;
		let attrs = Attrs::new()
			.family(Family::SansSerif)
			.color(Srgba::BEVY_BLACK.to_cosmic());
		row.label(LabelConfig { label, ..default() })
			.style()
			.padding(UiRect::horizontal(Val::Px(4.0)));
		let target = row
			.spawn((
				CosmicEditBundle {
					buffer: CosmicBuffer::new(font_system, Metrics::new(font_size, font_size))
						.with_text(font_system, &value, attrs),
					fill_color: CosmicBackgroundColor(Color::NONE),
					sprite_bundle: SpriteBundle {
						visibility: Visibility::Hidden,
						..default()
					},
					max_chars: MaxChars(40),
					max_lines: MaxLines(1),
					text_position: CosmicTextAlign::Left { padding: 2 },
					x_offset: XOffset {
						left: -5.0,
						width: 1.0,
					},
					..default()
				},
				comp,
			))
			.id();
		row.spawn((
			ButtonBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Px(y + 8.0),
					border: UiRect::all(Val::Px(2.5)),
					padding: UiRect::all(Val::Px(5.0)),
					overflow: Overflow::clip(),
					..default()
				},
				border_color: Srgba::BEVY_BLACK.into(),
				background_color: Srgba::BEVY_WHITE.into(),
				..default()
			},
			CosmicSource(target),
		));
	});
}

fn receive_display_msg(
	trigger: Trigger<DisplayMsgEvent>,
	mut cmd: Commands,
	query_msg_holder: Query<(Entity, Option<&Children>), With<ChatMessageHolder>>,
) {
	let Ok((ent_holder, op_child)) = query_msg_holder.get_single() else {
		return;
	};
	let new_msg = trigger.event();

	cmd.entity(ent_holder)
		.insert(WithChild(TextBundle::from_section(
			new_msg.trim().to_owned(),
			TextStyle {
				font_size: FontTypeSize::NAME,
				color: Srgba::BEVY_WHITE,
				..default()
			},
		)));
	if let Some(child) = op_child {
		if child.len() > 100 {
			let despawn_first = child.first().unwrap();
			cmd.entity(*despawn_first).despawn_recursive();
		}
	}
}

fn handle_local_message(
	mut cmd: Commands,
	mut client: ResMut<ClientConnectionManager>,
	mut server: ResMut<ServerConnectionManager>,
	mut query_cosmic_editor: Query<&mut CosmicEditor>,
	identity: NetworkIdentity,
	kb_i: Res<ButtonInput<KeyCode>>,
	focused_widget: Res<FocusedWidget>,
	query_type_chat: Query<(Entity, &CosmicBuffer), With<TypeChat>>,
) {
	let Ok((ent_typer, cos_buff)) = query_type_chat.get_single() else {
		return;
	};
	if !focused_widget
		.0
		.is_some_and(|ent_widget| ent_widget == ent_typer)
		|| !kb_i.just_pressed(KeyCode::Enter)
	{
		return;
	}
	let Ok(mut cosmic_editor) = query_cosmic_editor.get_single_mut() else {
		return;
	};
	cosmic_editor.with_buffer_mut(|buf| {
		let Some(buffed) = buf.lines.get_mut(0) else {
			return;
		};

		let msg_copy = buffed.text().to_owned();
		let msg = msg_copy.trim();

		let single_line = &cos_buff.0.lines[0];
		buffed.set_text("", single_line.ending(), single_line.attrs_list().clone());
		buffed.reset();

		if let Some((first, rest)) = msg.split_once(char::is_whitespace) {
			if first.starts_with(">echo") {
				cmd.trigger(DisplayMsgEvent(rest.trim().to_owned()));
				return;
			}
		};

		if identity.is_client() {
			client
				.send_message::<MessageChannel, MessageCtx>(&mut MessageCtx(msg.to_owned()))
				.unwrap_or_else(|e| {
					error!("Fail to send message: {:?}", e);
				});
		} else if identity.is_server() {
			let server_msg = format!("[SERVER] {}", msg);
			cmd.trigger(DisplayMsgEvent(server_msg.clone()));
			server
				.send_message_to_target::<MessageChannel, MessageCtx>(
					&mut MessageCtx(server_msg),
					NetworkTarget::All,
				)
				.unwrap_or_else(|e| {
					error!("Fail to send message: {:?}", e);
				});
		}
	});
}

fn lock_cursor(
	mut lock_on_ui: Local<Option<(Entity, Vec2)>>,
	mut query_window: Query<&mut Window, With<PrimaryWindow>>,
	query_ui: Query<(Entity, Ref<Interaction>), With<LockableOnUI>>,
) {
	let Ok(mut window) = query_window.get_single_mut() else {
		return;
	};
	if let Some((ent, prev_pos_cursor)) = *lock_on_ui {
		let (_, inter) = query_ui.get(ent).unwrap();
		if *inter == Interaction::Pressed {
			let prev_pos = prev_pos_cursor.xy();
			window.cursor.grab_mode = CursorGrabMode::Confined;
			window.set_cursor_position(Some(prev_pos));
			return;
		} else {
			window.cursor.grab_mode = CursorGrabMode::None;
			*lock_on_ui = None;
		}
	}
	query_ui.iter().for_each(|(ent, inter)| {
		if *inter != Interaction::Pressed && !inter.is_changed() {
			return;
		}
		let Some(cur_pos) = window.cursor_position() else {
			return;
		};
		*lock_on_ui = Some((ent, cur_pos));
	});
}
