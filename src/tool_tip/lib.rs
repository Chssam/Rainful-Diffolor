use bevy::{
	prelude::*,
	reflect::{ReflectRef, TypeInfo},
};

#[derive(Component)]
pub(super) struct ContentDisplayerUI;

#[derive(Resource, Default)]
pub(super) struct ActiveContent(pub Option<Entity>);

#[derive(Reflect)]
pub struct ToolName(pub &'static str);

#[derive(Reflect)]
pub struct ToolTip(pub &'static str);

#[derive(Component, Deref)]
pub struct ToolTipContent(Box<dyn Reflect>);

impl ToolTipContent {
	pub fn new<T: Reflect>(comp: T) -> Self {
		Self(Box::new(comp))
	}
}

pub trait ToolsInfo {
	fn tool_tip(&self) -> &str;
	fn tool_name(&self) -> String;
}

impl ToolsInfo for dyn Reflect {
	fn tool_tip(&self) -> &str {
		match self.get_represented_type_info().unwrap() {
			TypeInfo::Struct(type_info) => type_info.get_attribute::<ToolTip>().map(|v| v.0),
			TypeInfo::TupleStruct(type_info) => type_info.get_attribute::<ToolTip>().map(|v| v.0),
			TypeInfo::Enum(type_info) => {
				let ReflectRef::Enum(enum_ref) = self.reflect_ref() else {
					unreachable!();
				};
				let enum_name = enum_ref.variant_name();
				let var_info = type_info.variant(enum_name).unwrap();
				var_info.get_attribute::<ToolTip>().map(|v| v.0)
			},
			TypeInfo::Value(_value_info) => Some(
				self.downcast_ref::<&str>()
					.cloned()
					.unwrap_or("*Unsupported Tool Tip Format"),
			),
			_ => Some("*Unsupported Tool Tip Types"),
		}
		.unwrap_or("No Tooltip")
	}

	fn tool_name(&self) -> String {
		match self.get_represented_type_info().unwrap() {
			TypeInfo::Struct(type_info) => type_info.get_attribute::<ToolName>().map(|v| v.0),
			TypeInfo::TupleStruct(type_info) => type_info.get_attribute::<ToolName>().map(|v| v.0),
			TypeInfo::Enum(type_info) => {
				let ReflectRef::Enum(enum_ref) = self.reflect_ref() else {
					unreachable!();
				};
				let enum_name = enum_ref.variant_name();
				let var_info = type_info.variant(enum_name).unwrap();
				var_info.get_attribute::<ToolName>().map(|v| v.0)
			},
			_ => Some(""),
		}
		.unwrap_or({
			let mut not_first = false;
			let b = format!("{:?}", self)
				.chars()
				.fold(String::new(), |mut named, c| {
					if c.is_uppercase() && not_first {
						named.push(' ');
					}
					not_first = true;
					named.push(c);
					named
				});
			&b.to_owned()
		})
		.to_owned()
	}
}
