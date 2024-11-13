use bevy::{
	ecs::component::{ComponentHooks, StorageType},
	prelude::*,
};

use super::*;

pub(super) fn scroll_ui(
	mut query_scroll: Query<(&mut Style, &Node, &Parent), With<Scrollable>>,
	node_list: Query<&Node>,
	hover_map: Res<HoverMap>,
	actions_ui: Res<ActionState<UIAction>>,
) {
	let is_holding = actions_ui.pressed(&UIAction::Primary) as i32 as f32;
	let motion = actions_ui.axis_pair(&UIAction::Pull) * 20. / 8. * is_holding;
	let mut scroll = actions_ui.axis_pair(&UIAction::Scroll) * 20.;
	if actions_ui.pressed(&UIAction::SwapScroll) {
		std::mem::swap(&mut scroll.x, &mut scroll.y);
	}
	let total_scroll = scroll + motion;
	if total_scroll == Vec2::ZERO {
		return;
	}

	let mapped = hover_map
		.iter()
		.map(|(_, map)| map.keys().cloned().collect::<Vec<_>>())
		.collect::<Vec<_>>()
		.concat();
	let mut hovered = query_scroll.iter_many_mut(mapped.iter());
	while let Some((mut style, uinode, parenty)) = hovered.fetch_next() {
		let fit_node = node_list.get(parenty.get()).unwrap().size();
		let Vec2 { x, y } = (uinode.size() - fit_node).max(Vec2::ZERO);

		if let Val::Px(left) = &mut style.left {
			*left = (*left + total_scroll.x).clamp(-x, 0.);
		}
		if let Val::Px(right) = &mut style.right {
			*right = (*right - total_scroll.x).clamp(0., x);
		}
		if let Val::Px(top) = &mut style.top {
			*top = (*top + total_scroll.y).clamp(-y, 0.);
		}
		if let Val::Px(bottom) = &mut style.bottom {
			*bottom = (*bottom - total_scroll.y).clamp(0., y);
		}
	}
}

#[derive(Default)]
pub struct Scrollable;

impl Component for Scrollable {
	const STORAGE_TYPE: StorageType = StorageType::Table;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_add(|mut world, entity, _component_id| {
			let mut ent_mut = world.entity_mut(entity);
			if let Some(mut z_index) = ent_mut.get_mut::<ZIndex>() {
				*z_index = UILayer::SCROLL_VIEW;
			}
		});
	}
}
