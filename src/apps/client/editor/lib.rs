use aery::prelude::*;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use sickle_ui::prelude::*;

use super::*;

#[derive(Relation)]
#[aery(Total, Symmetric, Poly)]
pub struct ObjectRelationUI;

#[derive(Component)]
pub struct LargeIcon;

#[derive(Component)]
pub struct BrushCollector;

#[derive(Event)]
pub struct NewBrush(pub BrushRef, pub UVec2, pub Vec<u8>);

impl LargeIcon {
	fn bundle(size: f32) -> impl Bundle {
		let size = Val::Px(size);
		let all_side = UiRect::all(Val::Px(2.0));
		ImageBundle {
			style: Style {
				width: size,
				height: size,
				border: all_side,
				margin: all_side,
				..default()
			},
			..default()
		}
	}
}

pub trait UiMoreIconExt {
	fn medium_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity>;
	fn large_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity>;
	fn extra_large_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity>;
}

impl UiMoreIconExt for UiBuilder<'_, Entity> {
	fn medium_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity> {
		let mut icon = self.spawn((Name::new("Medium Icon"), LargeIcon::bundle(20.0), LargeIcon));
		icon.style().image(image_source);
		icon
	}
	fn large_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity> {
		let mut icon = self.spawn((Name::new("Large Icon"), LargeIcon::bundle(28.0), LargeIcon));
		icon.style().image(image_source);
		icon
	}
	fn extra_large_icon(&mut self, image_source: ImageSource) -> UiBuilder<Entity> {
		let mut icon = self.spawn((
			Name::new("Extra Large Icon"),
			LargeIcon::bundle(36.0),
			LargeIcon,
		));
		icon.style().image(image_source);
		icon
	}
}

#[derive(Reflect)]
pub(super) enum AttactObject {
	Pick,
	LockPick,
	Visibility,
}

impl AttactObject {
	pub fn target(&self, ent_obj: Entity) -> On<Pointer<Click>> {
		match self {
			AttactObject::Pick => On::<Pointer<Click>>::run(
				move |mut query_user: Query<&mut SelectedObject, With<MainUser>>| {
					let Ok(mut selected_obj) = query_user.get_single_mut() else {
						return;
					};
					if selected_obj.group.contains(&ent_obj) {
						selected_obj.deselect_single(ent_obj);
						return;
					}
					selected_obj.select_single(ent_obj);
				},
			),
			AttactObject::LockPick => {
				On::<Pointer<Click>>::run(move |mut query_object: Query<&mut Pickable>| {
					let Ok(mut pick_lock) = query_object.get_mut(ent_obj) else {
						return;
					};
					pick_lock.is_hoverable = !pick_lock.is_hoverable;
				})
			},
			AttactObject::Visibility => {
				On::<Pointer<Click>>::run(move |mut query_object: Query<&mut Visibility>| {
					let Ok(mut visibility) = query_object.get_mut(ent_obj) else {
						return;
					};
					*visibility = match *visibility {
						Visibility::Hidden => Visibility::Inherited,
						_ => Visibility::Hidden,
					}
				})
			},
		}
	}
}

#[derive(Component)]
pub(super) struct HexColorText;

#[derive(Component)]
pub(super) struct ColorChoice;

#[derive(Component)]
pub(super) struct DisplayObjectDirectory;

#[derive(Component)]
pub enum DisplayColor {
	Foreground,
	Background,
}
