use bevy::{color::palettes::tailwind::NEUTRAL_800, prelude::*, window::WindowResized};
use bevy_cosmic_edit::FocusedWidget;
use bevy_mod_picking::prelude::*;
use leafwing_input_manager::prelude::*;
pub mod lib;
use lib::*;

use crate::trait_bevy::BuildCommonImage;

pub(super) struct CameraPlugin;
impl Plugin for CameraPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<IsUnFocusOnUI>()
			.insert_resource(ClearColor(NEUTRAL_800.into()))
			.add_plugins(InputManagerPlugin::<GlobalCamAction>::default())
			.add_systems(Startup, setup_cam)
			.add_systems(
				Update,
				(edit_global_cam.run_if(ui_unfocus), background_tile_update).chain(),
			)
			.add_systems(
				PostUpdate,
				(
					set_is_ui_active,
					change_bg_color.run_if(resource_changed::<ClearColor>),
				),
			);
	}
}

fn set_is_ui_active(
	mut is_unfocus_on_ui: ResMut<IsUnFocusOnUI>,
	mouse_button: Res<ButtonInput<MouseButton>>,
	key_button: Res<ButtonInput<KeyCode>>,
	query_ui: Query<&PickingInteraction, With<Node>>,
	focused_widget: Res<FocusedWidget>,
) {
	let moused = mouse_button.get_pressed().next().is_some()
		&& mouse_button.get_just_pressed().next().is_none();
	let keyed =
		key_button.get_pressed().next().is_some() && key_button.get_just_pressed().next().is_none();
	if moused || keyed {
		return;
	}
	let got_interact = query_ui
		.iter()
		.any(|inter| inter != &PickingInteraction::None);
	is_unfocus_on_ui.0 = !got_interact && focused_widget.is_none();
}

pub fn ui_unfocus(is_ui_unfocus: Res<IsUnFocusOnUI>) -> bool {
	is_ui_unfocus.0
}

fn setup_cam(world: &mut World) {
	let bg_color = world.resource::<ClearColor>().bg_color();
	let mut img_asset = world.resource_mut::<Assets<Image>>();
	let hand_img = img_asset.rgba8_image(bg_color, UVec2::splat(2));
	world.spawn((
		Camera2dBundle {
			projection: OrthographicProjection {
				far: CAMERA_VIEW_RANGE,
				near: -CAMERA_VIEW_RANGE,
				..default()
			},
			..default()
		},
		InputManagerBundle::with_map(GlobalCamAction::bind_default()),
		MainCamera,
	));
	world.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::Z * TILE_DISTANCE),
			texture: hand_img,
			..default()
		},
		ImageScaleMode::Tiled {
			tile_x: true,
			tile_y: true,
			stretch_value: 0.5,
		},
		BackGroundTile,
		Pickable::IGNORE,
	));
}

fn edit_global_cam(
	mut physics_cam: Query<
		(
			&mut Transform,
			&mut OrthographicProjection,
			&ActionState<GlobalCamAction>,
		),
		(With<Camera2d>, With<MainCamera>),
	>,
) {
	let (mut pos, mut ortho, actions_cam) = physics_cam.single_mut();
	let scroll = actions_cam.value(&GlobalCamAction::Scroll) * ortho.scale * 0.25;
	let max_range = Vec3::splat(MAX_VALID_RANGE);
	if actions_cam.pressed(&GlobalCamAction::Control) {
		let press_in = actions_cam.pressed(&GlobalCamAction::ZoomIn) as i8 as f32;
		let press_out = actions_cam.pressed(&GlobalCamAction::ZoomOut) as i8 as f32;
		let zoom_in = press_in
			* actions_cam
				.current_duration(&GlobalCamAction::ZoomIn)
				.as_secs_f32()
			/ 2.0;
		let zoom_out = press_out
			* actions_cam
				.current_duration(&GlobalCamAction::ZoomOut)
				.as_secs_f32()
			/ 2.0;
		let total_scale = zoom_out - zoom_in - scroll;
		if total_scale != 0.0 {
			ortho.scale = (ortho.scale + total_scale).clamp(1e-3, 100.0);
		}
	} else {
		let cal = scroll * 200.0 * ortho.scale.clamp(0.6, 2.0);
		if cal != 0.0 {
			match actions_cam.pressed(&GlobalCamAction::PanAxisX) {
				true => pos.translation.x += cal,
				false => pos.translation.y += cal,
			}
		}
	}

	if actions_cam.pressed(&GlobalCamAction::FreeMove) {
		let moved = actions_cam.axis_pair(&GlobalCamAction::Pull);
		let new_pos = pos.translation.truncate() - Vec2::new(moved.x, -moved.y) * ortho.scale * 1.5;
		pos.translation = new_pos.extend(pos.translation.z);
	}
	pos.bypass_change_detection().translation = pos.translation.clamp(-max_range, max_range);
}

fn background_tile_update(
	mut query_bg_tiled: Query<
		(&mut Transform, &mut Sprite),
		(With<BackGroundTile>, Without<Camera2d>),
	>,
	mut window_resized: EventReader<WindowResized>,
	query_cam_change: Query<(Ref<OrthographicProjection>, Ref<Transform>), With<Camera2d>>,
) {
	let (orth, tf_cam) = query_cam_change.single();
	let (mut transform, mut sprite) = query_bg_tiled.single_mut();
	let scaled = (orth.scale * 50.0).round().max(4.0);
	let scale_even = scaled - scaled % 2.0;
	if orth.is_changed() {
		transform.scale = Vec2::splat(scale_even).extend(1.0);
	}
	if orth.is_changed() || tf_cam.is_changed() || !window_resized.is_empty() {
		let pos_cam = tf_cam.translation.truncate();
		let calculated_pos = pos_cam - pos_cam % scale_even;
		transform.translation = calculated_pos.floor().extend(TILE_DISTANCE);
	}
	window_resized.read().for_each(|win| {
		let size_win = Vec2::new(win.width, win.height) / 44.0;
		sprite.custom_size = Some(size_win.ceil());
	});
}

fn change_bg_color(
	clear_color: Res<ClearColor>,
	query_bg_tiled: Query<&Handle<Image>, With<BackGroundTile>>,
	mut image_assets: ResMut<Assets<Image>>,
) {
	let img = image_assets.get_mut(query_bg_tiled.single().id()).unwrap();
	img.data = clear_color.bg_color();
}
