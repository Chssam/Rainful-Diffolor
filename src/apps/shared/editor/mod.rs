use arboard::Clipboard;
use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*,
	time::common_conditions::on_timer,
	window::{CursorGrabMode, PrimaryWindow},
};
use bevy_cosmic_edit::*;
use bevy_mod_picking::prelude::{On, *};
use cosmic_text::{Attrs, Edit as _, Family, Metrics};
use i_cant_believe_its_not_bsn::*;
use lightyear::prelude::*;
use rand::{distributions::Alphanumeric, Rng};

pub mod lib;
mod one_shot;
use lib::*;
use one_shot::*;

use super::*;
use crate::{trait_bevy::*, ui::*};

pub(super) struct EditorPlugin;
impl Plugin for EditorPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<IPAccept>()
			.init_resource::<EditorParts>()
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
					accept_public_ip.run_if(on_timer(Duration::from_secs(3))),
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
	trigger: Trigger<OnAdd, Text>,
	query_no_pick: Query<Entity, Without<Pickable>>,
	mut cmd: Commands,
) {
	let ent = trigger.entity();
	if query_no_pick.contains(ent) {
		cmd.entity(ent).insert(Pickable::IGNORE);
	}
}

fn editor_ui(mut cmd: Commands, mut font_system: ResMut<CosmicFontSystem>) {
	use EditorPosition::*;

	let mut editored = cmd.spawn((
		NodeBundle {
			style: Style {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::SpaceBetween,
				..default()
			},
			z_index: ZIndex::Global(-10),
			..default()
		},
		UiMainRootNode,
		Pickable::IGNORE,
	));

	// MARK: TOP ROOT
	editored.with_child((
		EffectUIBundle::column().no_padding().node(|edit| {
			let style = &mut edit.style;
			style.width = Val::Percent(100.0);
			style.height = Val::Auto;
			style.flex_grow = 0.0;
		}),
		WithChildren([
			(
				EffectUIBundle::row().node(|node| node.style.border = UiRect::bottom(Val::Px(2.))),
				(
					MenuBar1,
					WithChild((
						EffectUIBundle::text().menu(),
						WithChild(TextBuild::single("Apps")),
						OwnObserve::new(|trigger: Trigger<RunEffect>, mut cmd: Commands| {
							cmd.spawn((
								DropDownAt::bottom(trigger.entity()),
								WithChildren([
									(
										EffectUIBundle::text().item(),
										WithChild(TextBuild::single("Exit")),
										OwnObserve::new(
											|_trigger: Trigger<RunEffect>,
											 mut write_events: EventWriter<AppExit>| {
												write_events.send(AppExit::Success);
											},
										),
									),
									(
										EffectUIBundle::text()
											.item()
											.tip("Paste link in searcher to view Source"),
										WithChild(TextBuild::single("Github")),
										OwnObserve::new(|_trigger: Trigger<RunEffect>| {
											let mut clip_board = Clipboard::new().unwrap();
											let link = "https://github.com/Chssam/Rainful-Diffolor";
											clip_board.set_text(link).unwrap();
										}),
									),
								]),
							));
						}),
					)),
				),
			),
			(
				EffectUIBundle::row().node(|node| node.style.border = UiRect::bottom(Val::Px(2.))),
				(
					MenuBar2,
					WithChild((
						EffectUIBundle::text()
							.click()
							.tip("Just here, it does nothing"),
						WithChild(TextBuild::single("Holding")),
						OwnObserve::new(|_trigger: Trigger<RunEffect>| {
							info!("HELLO BUT NOT HELLO");
						}),
					)),
				),
			),
		]),
	));

	// MARK: MIDDLE ROOT
	editored.with_child((
		EffectUIBundle::row()
			.no_pick()
			.no_effect()
			.no_padding()
			.node(|node| {
				let style = &mut node.style;
				style.width = Val::Percent(100.);
				style.height = Val::Auto;
				style.min_height = Val::Px(80.);
				style.flex_grow = 1.;
			}),
		// MARK: PANEL LEFT
		WithChild((
			PanelLeft,
			EffectUIBundle::column().no_padding().node(|node| {
				let style = &mut node.style;
				style.width = Val::Px(100.);
				style.min_width = Val::Px(60.);
				style.max_width = Val::Percent(70.);
			}),
			WithChildren([
				(
					Maybe::new(PanelLeftTop),
					EffectUIBundle::column().node(|node| {
						let style = &mut node.style;
						style.height = Val::Percent(50.);
						style.min_height = Val::Percent(30.);
						style.max_height = Val::Percent(80.);
					}),
					Maybe::NONE,
				),
				(
					Maybe::NONE,
					EffectUIBundle::row_resize(),
					Maybe::new(On::<Pointer<Drag>>::run(
						|listener: Listener<Pointer<Drag>>,
						 editor_part: Res<EditorParts>,
						 mut query_style: Query<&mut Style>| {
							let mut style = query_style
								.get_mut(editor_part.part(&PanelLeftTop))
								.unwrap();
							let dragged = listener.delta.y;
							let (Val::Percent(min_height), Val::Percent(max_height)) =
								(style.min_height, style.max_height)
							else {
								return;
							};
							if let Val::Percent(height) = &mut style.height {
								*height = (*height + dragged).clamp(min_height, max_height);
							}
						},
					)),
				),
				(
					Maybe::new(PanelLeftBottom),
					EffectUIBundle::column().grow(),
					Maybe::NONE,
				),
			]),
		)),
		WithChild((
			EffectUIBundle::column_resize(),
			On::<Pointer<Drag>>::run(
				|listener: Listener<Pointer<Drag>>,
				 editor_part: Res<EditorParts>,
				 mut query_style: Query<&mut Style>| {
					let mut style = query_style.get_mut(editor_part.part(&PanelLeft)).unwrap();
					let dragged = listener.delta.x;
					let Val::Px(min_width) = style.min_width else {
						return;
					};
					if let Val::Px(width) = &mut style.width {
						*width = (*width + dragged).max(min_width);
					}
				},
			),
			(),
		)),
		// MARK: PANEL MIDDLE
		WithChild((
			PanelMiddle,
			EffectUIBundle::column().no_pick().no_effect().node(|node| {
				let style = &mut node.style;
				style.width = Val::Auto;
				style.height = Val::Auto;
				style.min_width = Val::Px(20.);
				style.align_items = AlignItems::Stretch;
				style.justify_content = JustifyContent::FlexEnd;
				style.flex_grow = 1.;
			}),
		)),
		WithChild((
			EffectUIBundle::column_resize(),
			On::<Pointer<Drag>>::run(
				|listener: Listener<Pointer<Drag>>,
				 editor_part: Res<EditorParts>,
				 mut query_style: Query<&mut Style>| {
					let mut style = query_style.get_mut(editor_part.part(&PanelRight)).unwrap();
					let dragged = listener.delta.x;
					let Val::Px(min_width) = style.min_width else {
						return;
					};
					if let Val::Px(width) = &mut style.width {
						*width = (*width - dragged).max(min_width);
					}
				},
			),
		)),
		// MARK: PANEL RIGHT
		WithChild((
			PanelRight,
			EffectUIBundle::column().no_padding().node(|node| {
				let style = &mut node.style;
				style.width = Val::Px(120.);
				style.min_width = Val::Px(120.);
				style.max_width = Val::Percent(80.);
			}),
			WithChildren([
				(
					Maybe::new(PanelRightTop),
					EffectUIBundle::column().node(|node| {
						let style = &mut node.style;
						style.height = Val::Percent(50.);
						style.min_height = Val::Percent(20.);
						style.max_height = Val::Percent(70.);
					}),
					Maybe::NONE,
				),
				(
					Maybe::NONE,
					EffectUIBundle::row_resize(),
					Maybe::new(On::<Pointer<Drag>>::run(
						|listener: Listener<Pointer<Drag>>,
						 editor_part: Res<EditorParts>,
						 mut query_style: Query<&mut Style>| {
							let mut style = query_style
								.get_mut(editor_part.part(&PanelRightTop))
								.unwrap();
							let dragged = listener.delta.y;
							let (Val::Percent(min_height), Val::Percent(max_height)) =
								(style.min_height, style.max_height)
							else {
								return;
							};
							if let Val::Percent(height) = &mut style.height {
								*height = (*height + dragged).clamp(min_height, max_height);
							}
						},
					)),
				),
				(
					Maybe::new(PanelRightBottom),
					EffectUIBundle::column().grow(),
					Maybe::NONE,
				),
			]),
			(),
		)),
	));

	editored.with_child((
		EffectUIBundle::row_resize(),
		On::<Pointer<Drag>>::run(
			|listener: Listener<Pointer<Drag>>,
			 editor_part: Res<EditorParts>,
			 mut query_style: Query<&mut Style>| {
				let mut style = query_style.get_mut(editor_part.part(&PanelBottom)).unwrap();
				let dragged = listener.delta.y;
				let Val::Px(min_height) = style.min_height else {
					return;
				};
				if let Val::Px(height) = &mut style.height {
					*height = (*height - dragged).max(min_height);
				}
			},
		),
	));

	// MARK: BOTTOM ROOT
	editored.with_child((
		EffectUIBundle::row().no_padding().node(|node| {
			let style = &mut node.style;
			style.min_height = Val::Px(80.);
			style.height = Val::Px(150.);
		}),
		PanelBottom,
		WithChild((
			PanelBottomLeft,
			EffectUIBundle::column().full().node(|node| {
				let style = &mut node.style;
				style.width = Val::Percent(50.);
				style.min_width = Val::Px(120.);
				style.max_width = Val::Percent(80.);
			}),
			WithChild((
				EffectUIBundle::row()
					.no_effect()
					.node(|node| node.background_color = Color::BLACK.into()),
				AsContainer,
			)),
		)),
		WithChild((
			EffectUIBundle::column_resize(),
			On::<Pointer<Drag>>::run(
				|listener: Listener<Pointer<Drag>>,
				 editor_part: Res<EditorParts>,
				 mut query_style: Query<&mut Style>| {
					let mut style = query_style
						.get_mut(editor_part.part(&PanelBottomLeft))
						.unwrap();
					let dragged = listener.delta.x;
					if let Val::Percent(width) = &mut style.width {
						*width = (*width + dragged);
					}
				},
			),
		)),
		WithChild((
			EffectUIBundle::column().full(),
			PanelBottomRight,
			WithChild((
				EffectUIBundle::row()
					.no_effect()
					.node(|node| node.background_color = Color::BLACK.into()),
				AsContainer,
			)),
			WithChild((
				EffectUIBundle::column(),
				NotTabYet::new("Connections"),
				WithChild((
					EffectUIBundle::text().menu(),
					WithChild(TextBuild::single("Options")),
					OwnObserve::new(|trigger: Trigger<RunEffect>, mut cmd: Commands| {
						cmd.spawn((
							DropDownAt::bottom(trigger.entity()),
							WithChildren([
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single("Private")),
									OwnObserve::new(select_ip_self),
								),
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single("Local")),
									OwnObserve::new(select_ip_local),
								),
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single("Public")),
									OwnObserve::new(select_ip_public),
								),
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single("Client Toggle")),
									OwnObserve::new(client_system),
								),
								(
									EffectUIBundle::text().item(),
									WithChild(TextBuild::single("Server Toggle")),
									OwnObserve::new(server_system),
								),
							]),
						));
					}),
				)),
				{
					let name = rand::thread_rng()
						.sample_iter(&Alphanumeric)
						.take(8)
						.map(char::from)
						.collect::<String>();
					input_node("Name", &name, NameInput, &mut font_system)
				},
				input_node(
					"IP",
					&CLIENT_ADDR.ip().to_string(),
					ConnectionIP,
					&mut font_system,
				),
				input_node(
					"Client Port",
					&CLIENT_ADDR.port().to_string(),
					ClientPort,
					&mut font_system,
				),
				input_node(
					"Server Port",
					&SERVER_ADDR.port().to_string(),
					ServerPort,
					&mut font_system,
				),
			)),
			WithChild((
				EffectUIBundle::column(),
				NotTabYet::new("Messages"),
				WithChild({
					let font_size = FontTypeSize::NAME;
					let y = font_size + 10.0;
					let attrs = Attrs::new()
						.family(Family::SansSerif)
						.color(Srgba::BEVY_BLACK.to_cosmic());

					(
						EffectUIBundle::row(),
						WithChild((
							EffectUIBundle::text().only_pick().no_effect().node(|node| {
								let style = &mut node.style;
								style.width = Val::Percent(100.);
								style.height = Val::Px(y);
								style.border = UiRect::all(Val::Px(2.));
								style.padding = UiRect::all(Val::Px(5.));
								node.border_radius = BorderRadius::all(Val::Px(3.));
								node.border_color = Srgba::BEVY_BLACK.into();
								node.background_color = Srgba::BEVY_WHITE.into();
							}),
							UiImage::default(),
							OwnCosmicEdit::new(
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
								(
									Placeholder::new(
										"<Chat>",
										Attrs::new()
											.family(Family::SansSerif)
											.color(Srgba::BEVY_WHITE.darker(0.3).to_cosmic()),
									),
									TypeChat,
								),
							),
						)),
					)
				}),
				WithChild((
					EffectUIBundle::column().full(),
					WithChild((EffectUIBundle::column().scroll_full(), ChatMessageHolder)),
				)),
			)),
		)),
	));
}

fn input_node<T: Bundle>(
	label: &str,
	value: &str,
	comp: T,
	font_system: &mut CosmicFontSystem,
) -> impl Bundle {
	let font_size = FontTypeSize::NAME;
	let y = font_size + 10.0;
	let attrs = Attrs::new()
		.family(Family::SansSerif)
		.color(Srgba::BEVY_BLACK.to_cosmic());

	WithChild((
		EffectUIBundle::row(),
		WithChild((EffectUIBundle::text(), TextBuild::single(&label))),
		WithChild((
			EffectUIBundle::text().only_pick().no_effect().node(|node| {
				let style = &mut node.style;
				style.width = Val::Percent(100.);
				style.height = Val::Px(y);
				style.border = UiRect::all(Val::Px(2.));
				style.padding = UiRect::all(Val::Px(5.));
				node.border_radius = BorderRadius::all(Val::Px(3.));
				node.border_color = Srgba::BEVY_BLACK.into();
				node.background_color = Srgba::BEVY_WHITE.into();
			}),
			UiImage::default(),
			OwnCosmicEdit::new(
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
			),
		)),
	))
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
		.insert(WithChild(TextBuild::single(new_msg.trim())));
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
	mut on_drag: EventReader<Pointer<DragStart>>,
	mut on_drag_leave: EventReader<Pointer<DragEnd>>,
	mut query_window: Query<&mut Window, With<PrimaryWindow>>,
	mut lock_on_ui: Local<Option<(Entity, Vec2)>>,
	query_ui: Query<(), With<LockableOnUI>>,
) {
	let Ok(mut window) = query_window.get_single_mut() else {
		return;
	};

	if on_drag_leave
		.read()
		.any(|pointed| lock_on_ui.is_some_and(|(ent, _)| ent == pointed.target()))
	{
		lock_on_ui.take();
	}

	if let Some((_, prev_pos_cursor)) = *lock_on_ui {
		let prev_pos = prev_pos_cursor.xy();
		window.cursor.grab_mode = CursorGrabMode::Confined;
		window.set_cursor_position(Some(prev_pos));
	} else {
		window.cursor.grab_mode = CursorGrabMode::None;
		*lock_on_ui = None;
	}

	if let Some(pointed) = on_drag
		.read()
		.find(|pointed| query_ui.contains(pointed.target()))
	{
		let Some(cur_pos) = window.cursor_position() else {
			return;
		};
		*lock_on_ui = Some((pointed.target(), cur_pos));
	}
}
