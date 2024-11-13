use editor::DisplayColor;

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
	query_text.p1().iter_mut().for_each(|(mut texted, panel)| {
		let num = match panel {
			ColorPanelChanger::Red => r.to_string(),
			ColorPanelChanger::Green => g.to_string(),
			ColorPanelChanger::Blue => b.to_string(),
			ColorPanelChanger::Alpha => a.to_string(),
			ColorPanelChanger::Hue => hue.to_string(),
			ColorPanelChanger::Saturation => saturation.to_string(),
			ColorPanelChanger::Lightness => lightness.to_string(),
			ColorPanelChanger::SatLight => format!("{}:{}", saturation, lightness),
		};
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
