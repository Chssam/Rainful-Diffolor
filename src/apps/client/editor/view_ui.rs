use aery::prelude::*;
use editor::{ApplyDiff, DisplayColor, IconVisiblity, ObjectRelationUI};

use crate::apps::client::*;

pub(super) fn update_ui_content(
	mut query_text: ParamSet<(
		Query<&mut Text, With<EditorInfoComp>>,
		Query<(&mut Text, &ColorPanelChanger)>,
	)>,
	query_user: Query<(&CursorPos, &PaintInk), With<MainUser>>,
) {
	let Ok((position, paint)) = query_user.get_single() else {
		return;
	};
	if let Ok(mut cord_texted) = query_text.p0().get_single_mut() {
		cord_texted.sections[0].value = format!("Cord: [{:.1}, {:.1}]", position.x, position.y);
	};
	let [r, g, b, a] = paint.0.to_u8_array();
	let Hsla {
		hue,
		saturation,
		lightness,
		..
	} = paint.0.into();
	use ColorPanelChanger::*;
	query_text.p1().iter_mut().for_each(|(mut texted, panel)| {
		let num = match panel {
			Red => r.to_string(),
			Green => g.to_string(),
			Blue => b.to_string(),
			Alpha => a.to_string(),
			Hue => format!("{:.3}", hue),
			Saturation => format!("{:.3}", saturation),
			Lightness => format!("{:.3}", lightness),
			SatLight => format!("{:.3}:{:.3}", saturation, lightness),
		};
		match panel {
			Lightness | SatLight => {},
			_ => {
				let color = if lightness > 0.6 {
					Color::BLACK
				} else {
					Color::WHITE
				};

				texted.sections[0].style.color.apply_diff(color);
				texted.sections[1].style.color.apply_diff(color);
			},
		}

		texted.sections[1].value = num;
	});
}

pub(super) fn display_color_update(
	query_user: Query<&PaintInk, (With<MainUser>, Changed<PaintInk>)>,
	mut query_display_color: Query<(&mut BackgroundColor, &DisplayColor)>,
) {
	let Ok(paint) = query_user.get_single() else {
		return;
	};
	query_display_color
		.iter_mut()
		.for_each(|(mut node_color, display_color)| {
			node_color.0 = match display_color {
				DisplayColor::Foreground => paint.0,
				DisplayColor::Background => paint.1,
			}
			.with_alpha(1.0)
			.into();
		});
}

pub(super) fn update_color_image(
	mut image_assets: ResMut<Assets<Image>>,
	query_user: Query<(&PaintInk, &RGBAhsl), (With<MainUser>, Changed<PaintInk>)>,
) {
	let Ok((paint, rgba_hsl)) = query_user.get_single() else {
		return;
	};
	let RGBAhsl {
		red,
		blue,
		green,
		alpha,
		hue,
		saturation,
		lightness,
		sat_light,
	} = rgba_hsl;
	for (u, img_handle) in [
		red, green, blue, alpha, hue, saturation, lightness, sat_light,
	]
	.iter()
	.enumerate()
	{
		let img = image_assets.get_mut(img_handle.id()).unwrap();
		let data = match u {
			0 => paint.red(),
			1 => paint.green(),
			2 => paint.blue(),
			3 => paint.alpha(),
			4 => paint.hue(),
			5 => paint.saturation(),
			6 => paint.lightness(),
			7 => paint.sat_light(),
			_ => unreachable!(),
		};
		img.data = data;
	}
}

pub(super) fn update_visiblity(
	query_object: Query<(Entity, &Visibility), (Changed<Visibility>, With<ObjectWorld>)>,
	mut query_icon: Query<
		(
			(Has<IconVisiblity>, &mut Visibility),
			Relations<ObjectRelationUI>,
		),
		Without<ObjectWorld>,
	>,
) {
	if query_object.is_empty() {
		return;
	}
	query_object.iter().for_each(|(ent_obj, visiblity)| {
		query_icon
			.traverse_mut::<ObjectRelationUI>([ent_obj])
			.for_each(|ui, _| {
				if ui.0 {
					*ui.1 = *visiblity;
					info!("RUN IN 2");
				}
				info!("RUN");
			});
	});
}

// pub(super) fn update_move(
// 	query_object: Query<Entity, (Changed<InheritedVisibility>, With<ObjectWorld>)>,
// 	mut query_visit: Query<(
// 		(&mut Visibility, Has<IconVisiblity>),
// 		Relations<ObjectRelationUI>,
// 	)>,
// ) {
// 	if query_object.is_empty() {
// 		return;
// 	}
// 	info!("NO RUN");
// 	query_visit
// 		.traverse_mut::<ObjectRelationUI>(query_object.iter())
// 		.track_self()
// 		.for_each(|obj, _, ui, _| {
// 			if ui.1 {
// 				*ui.0 = obj.0.clone();
// 			}
// 			info!("RUN");
// 		});
// }
