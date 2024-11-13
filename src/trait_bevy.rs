use bevy::{
	color::palettes::*,
	ecs::{
		component::{ComponentHooks, StorageType},
		system::EntityCommands,
	},
	prelude::*,
	reflect::{ReflectRef, TypeInfo},
	render::{
		render_resource::{Extent3d, TextureDimension, TextureFormat},
		texture::ImageSampler,
	},
};
use i_cant_believe_its_not_bsn::*;

pub struct FontTypeSize;

impl FontTypeSize {
	pub const NAME: f32 = 16.0;
	pub const DESCRIPTION: f32 = 14.0;
}

/// Theme based on [Bevy](https://bevyengine.org/) logo
pub trait BevyColorTheme {
	const BEVY_WHITE: Color = Color::srgb(236.0 / 255.0, 236.0 / 255.0, 236.0 / 255.0);
	const BEVY_GRAY: Color = Color::srgb(178.0 / 255.0, 178.0 / 255.0, 178.0 / 255.0);
	const BEVY_DARK_GRAY: Color = Color::srgb(120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0);
	const BEVY_BLACK: Color = Color::srgb(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0);
}
impl BevyColorTheme for Srgba {}
impl BevyColorTheme for Color {}

/// [Ayu Theme](https://github.com/ayu-theme/ayu-colors)
pub trait AyuColorTheme {
	const AYU_GRAY: Color = Color::srgb(108.0 / 255.0, 115.0 / 255.0, 128.0 / 255.0);
	const AYU_LIME: Color = Color::srgb(149.0 / 255.0, 230.0 / 255.0, 203.0 / 255.0);
	const AYU_BLACK: Color = Color::srgb(13.0 / 255.0, 16.0 / 255.0, 23.0 / 255.0);
}

impl AyuColorTheme for Srgba {}
impl AyuColorTheme for Color {}

pub trait BuildCommonImage {
	fn rgba8_image(&mut self, data: Vec<u8>, size: UVec2) -> Handle<Image>;
}

impl BuildCommonImage for Assets<Image> {
	fn rgba8_image(&mut self, data: Vec<u8>, size: UVec2) -> Handle<Image> {
		let mut new_img = Image::new(
			Extent3d {
				width: size.x,
				height: size.y,
				..default()
			},
			TextureDimension::D2,
			data,
			TextureFormat::Rgba8UnormSrgb,
			default(),
		);
		new_img.sampler = ImageSampler::nearest();
		self.add(new_img)
	}
}

pub trait ApplyDiff {
	fn apply_diff(&mut self, apply: Self);
}

impl ApplyDiff for Color {
	fn apply_diff(&mut self, apply: Self) {
		if *self != apply {
			*self = apply;
		}
	}
}

#[derive(Reflect)]
pub struct ToolPath(pub &'static str);

pub trait ImageIconPath {
	fn path_img(&self) -> &str;
}

impl ImageIconPath for dyn Reflect {
	fn path_img(&self) -> &str {
		match self.get_represented_type_info().unwrap() {
			TypeInfo::Struct(type_info) => type_info.get_attribute::<ToolPath>().map(|v| v.0),
			TypeInfo::TupleStruct(type_info) => type_info.get_attribute::<ToolPath>().map(|v| v.0),
			TypeInfo::Enum(type_info) => {
				let ReflectRef::Enum(enum_ref) = self.reflect_ref() else {
					unreachable!();
				};
				let enum_name = enum_ref.variant_name();
				let var_info = type_info.variant(enum_name).unwrap();
				var_info.get_attribute::<ToolPath>().map(|v| v.0)
			},
			_ => None,
		}
		.unwrap_or("Marks.png")
	}
}

pub trait ImproveEntityCommands {
	fn with_child<B: Bundle>(&mut self, bundle: B) -> &mut Self;
	fn with_childs<B: Bundle, I: IntoIterator<Item = B> + Send + Sync + 'static>(
		&mut self,
		bundle: I,
	) -> &mut Self;
}

impl<'a> ImproveEntityCommands for EntityCommands<'a> {
	fn with_child<B: Bundle>(&mut self, bundle: B) -> &mut Self {
		self.insert(WithChild(bundle))
	}
	fn with_childs<B: Bundle, I: IntoIterator<Item = B> + Send + Sync + 'static>(
		&mut self,
		bundle: I,
	) -> &mut Self {
		self.insert(WithChildren(bundle))
	}
}

#[derive(Clone, Default)]
pub enum DirectImage {
	#[default]
	None,
	Path(String),
	Image {
		data: Vec<u8>,
		size: UVec2,
	},
}

impl Component for DirectImage {
	const STORAGE_TYPE: StorageType = StorageType::SparseSet;
	fn register_component_hooks(_hooks: &mut ComponentHooks) {
		_hooks.on_insert(|mut world, entity, _component_id| {
			let handle = match std::mem::take(
				&mut *world.entity_mut(entity).get_mut::<DirectImage>().unwrap(),
			) {
				DirectImage::None => return,
				DirectImage::Path(get_path) => world.resource::<AssetServer>().load(get_path),
				DirectImage::Image { data, size } => {
					let mut new_img = Image::new(
						Extent3d {
							width: size.x,
							height: size.y,
							..default()
						},
						TextureDimension::D2,
						data,
						TextureFormat::Rgba8UnormSrgb,
						default(),
					);
					new_img.sampler = ImageSampler::nearest();
					world.resource_mut::<Assets<Image>>().add(new_img)
				},
			};
			world
				.commands()
				.entity(entity)
				.insert(UiImage::new(handle))
				.remove::<DirectImage>();
		});
	}
}

pub trait IterAll {
	fn iter_basic() -> [Self; 16]
	where
		Self: Sized;
	fn iter_css() -> [Self; 147]
	where
		Self: Sized;
	fn iter_tailwind() -> [Self; 242]
	where
		Self: Sized;
}

impl IterAll for Srgba {
	fn iter_basic() -> [Self; 16] {
		use basic::*;
		[
			AQUA, BLACK, BLUE, FUCHSIA, GRAY, GREEN, LIME, MAROON, NAVY, OLIVE, PURPLE, RED,
			SILVER, TEAL, WHITE, YELLOW,
		]
	}

	fn iter_css() -> [Self; 147] {
		use css::*;
		[
			ALICE_BLUE,
			ANTIQUE_WHITE,
			AQUA,
			AQUAMARINE,
			AZURE,
			BEIGE,
			BISQUE,
			BLACK,
			BLANCHED_ALMOND,
			BLUE,
			BLUE_VIOLET,
			BROWN,
			BURLYWOOD,
			CADET_BLUE,
			CHARTREUSE,
			CHOCOLATE,
			CORAL,
			CORNFLOWER_BLUE,
			CORNSILK,
			CRIMSON,
			DARK_BLUE,
			DARK_CYAN,
			DARK_GOLDENROD,
			DARK_GRAY,
			DARK_GREEN,
			DARK_GREY,
			DARK_KHAKI,
			DARK_MAGENTA,
			DARK_OLIVEGREEN,
			DARK_ORANGE,
			DARK_ORCHID,
			DARK_RED,
			DARK_SALMON,
			DARK_SEA_GREEN,
			DARK_SLATE_BLUE,
			DARK_SLATE_GRAY,
			DARK_SLATE_GREY,
			DARK_TURQUOISE,
			DARK_VIOLET,
			DEEP_PINK,
			DEEP_SKY_BLUE,
			DIM_GRAY,
			DIM_GREY,
			DODGER_BLUE,
			FIRE_BRICK,
			FLORAL_WHITE,
			FOREST_GREEN,
			FUCHSIA,
			GAINSBORO,
			GHOST_WHITE,
			GOLD,
			GOLDENROD,
			GRAY,
			GREEN,
			GREEN_YELLOW,
			GREY,
			HONEYDEW,
			HOT_PINK,
			INDIAN_RED,
			INDIGO,
			IVORY,
			KHAKI,
			LAVENDER,
			LAVENDER_BLUSH,
			LAWN_GREEN,
			LEMON_CHIFFON,
			LIGHT_BLUE,
			LIGHT_CORAL,
			LIGHT_CYAN,
			LIGHT_GOLDENROD_YELLOW,
			LIGHT_GRAY,
			LIGHT_GREEN,
			LIGHT_GREY,
			LIGHT_PINK,
			LIGHT_SALMON,
			LIGHT_SEA_GREEN,
			LIGHT_SKY_BLUE,
			LIGHT_SLATE_GRAY,
			LIGHT_SLATE_GREY,
			LIGHT_STEEL_BLUE,
			LIGHT_YELLOW,
			LIME,
			LIMEGREEN,
			LINEN,
			MAGENTA,
			MAROON,
			MEDIUM_AQUAMARINE,
			MEDIUM_BLUE,
			MEDIUM_ORCHID,
			MEDIUM_PURPLE,
			MEDIUM_SEA_GREEN,
			MEDIUM_SLATE_BLUE,
			MEDIUM_SPRING_GREEN,
			MEDIUM_TURQUOISE,
			MEDIUM_VIOLET_RED,
			MIDNIGHT_BLUE,
			MINT_CREAM,
			MISTY_ROSE,
			MOCCASIN,
			NAVAJO_WHITE,
			NAVY,
			OLD_LACE,
			OLIVE,
			OLIVE_DRAB,
			ORANGE,
			ORANGE_RED,
			ORCHID,
			PALE_GOLDENROD,
			PALE_GREEN,
			PALE_TURQUOISE,
			PALE_VIOLETRED,
			PAPAYA_WHIP,
			PEACHPUFF,
			PERU,
			PINK,
			PLUM,
			POWDER_BLUE,
			PURPLE,
			REBECCA_PURPLE,
			RED,
			ROSY_BROWN,
			ROYAL_BLUE,
			SADDLE_BROWN,
			SALMON,
			SANDY_BROWN,
			SEASHELL,
			SEA_GREEN,
			SIENNA,
			SILVER,
			SKY_BLUE,
			SLATE_BLUE,
			SLATE_GRAY,
			SLATE_GREY,
			SNOW,
			SPRING_GREEN,
			STEEL_BLUE,
			TAN,
			TEAL,
			THISTLE,
			TOMATO,
			TURQUOISE,
			VIOLET,
			WHEAT,
			WHITE,
			WHITE_SMOKE,
			YELLOW,
			YELLOW_GREEN,
		]
	}

	fn iter_tailwind() -> [Self; 242] {
		use tailwind::*;
		[
			AMBER_50,
			AMBER_100,
			AMBER_200,
			AMBER_300,
			AMBER_400,
			AMBER_500,
			AMBER_600,
			AMBER_700,
			AMBER_800,
			AMBER_900,
			AMBER_950,
			BLUE_50,
			BLUE_100,
			BLUE_200,
			BLUE_300,
			BLUE_400,
			BLUE_500,
			BLUE_600,
			BLUE_700,
			BLUE_800,
			BLUE_900,
			BLUE_950,
			CYAN_50,
			CYAN_100,
			CYAN_200,
			CYAN_300,
			CYAN_400,
			CYAN_500,
			CYAN_600,
			CYAN_700,
			CYAN_800,
			CYAN_900,
			CYAN_950,
			EMERALD_50,
			EMERALD_100,
			EMERALD_200,
			EMERALD_300,
			EMERALD_400,
			EMERALD_500,
			EMERALD_600,
			EMERALD_700,
			EMERALD_800,
			EMERALD_900,
			EMERALD_950,
			FUCHSIA_50,
			FUCHSIA_100,
			FUCHSIA_200,
			FUCHSIA_300,
			FUCHSIA_400,
			FUCHSIA_500,
			FUCHSIA_600,
			FUCHSIA_700,
			FUCHSIA_800,
			FUCHSIA_900,
			FUCHSIA_950,
			GRAY_50,
			GRAY_100,
			GRAY_200,
			GRAY_300,
			GRAY_400,
			GRAY_500,
			GRAY_600,
			GRAY_700,
			GRAY_800,
			GRAY_900,
			GRAY_950,
			GREEN_50,
			GREEN_100,
			GREEN_200,
			GREEN_300,
			GREEN_400,
			GREEN_500,
			GREEN_600,
			GREEN_700,
			GREEN_800,
			GREEN_900,
			GREEN_950,
			INDIGO_50,
			INDIGO_100,
			INDIGO_200,
			INDIGO_300,
			INDIGO_400,
			INDIGO_500,
			INDIGO_600,
			INDIGO_700,
			INDIGO_800,
			INDIGO_900,
			INDIGO_950,
			LIME_50,
			LIME_100,
			LIME_200,
			LIME_300,
			LIME_400,
			LIME_500,
			LIME_600,
			LIME_700,
			LIME_800,
			LIME_900,
			LIME_950,
			NEUTRAL_50,
			NEUTRAL_100,
			NEUTRAL_200,
			NEUTRAL_300,
			NEUTRAL_400,
			NEUTRAL_500,
			NEUTRAL_600,
			NEUTRAL_700,
			NEUTRAL_800,
			NEUTRAL_900,
			NEUTRAL_950,
			ORANGE_50,
			ORANGE_100,
			ORANGE_200,
			ORANGE_300,
			ORANGE_400,
			ORANGE_500,
			ORANGE_600,
			ORANGE_700,
			ORANGE_800,
			ORANGE_900,
			ORANGE_950,
			PINK_50,
			PINK_100,
			PINK_200,
			PINK_300,
			PINK_400,
			PINK_500,
			PINK_600,
			PINK_700,
			PINK_800,
			PINK_900,
			PINK_950,
			PURPLE_50,
			PURPLE_100,
			PURPLE_200,
			PURPLE_300,
			PURPLE_400,
			PURPLE_500,
			PURPLE_600,
			PURPLE_700,
			PURPLE_800,
			PURPLE_900,
			PURPLE_950,
			RED_50,
			RED_100,
			RED_200,
			RED_300,
			RED_400,
			RED_500,
			RED_600,
			RED_700,
			RED_800,
			RED_900,
			RED_950,
			ROSE_50,
			ROSE_100,
			ROSE_200,
			ROSE_300,
			ROSE_400,
			ROSE_500,
			ROSE_600,
			ROSE_700,
			ROSE_800,
			ROSE_900,
			ROSE_950,
			SKY_50,
			SKY_100,
			SKY_200,
			SKY_300,
			SKY_400,
			SKY_500,
			SKY_600,
			SKY_700,
			SKY_800,
			SKY_900,
			SKY_950,
			SLATE_50,
			SLATE_100,
			SLATE_200,
			SLATE_300,
			SLATE_400,
			SLATE_500,
			SLATE_600,
			SLATE_700,
			SLATE_800,
			SLATE_900,
			SLATE_950,
			STONE_50,
			STONE_100,
			STONE_200,
			STONE_300,
			STONE_400,
			STONE_500,
			STONE_600,
			STONE_700,
			STONE_800,
			STONE_900,
			STONE_950,
			TEAL_50,
			TEAL_100,
			TEAL_200,
			TEAL_300,
			TEAL_400,
			TEAL_500,
			TEAL_600,
			TEAL_700,
			TEAL_800,
			TEAL_900,
			TEAL_950,
			VIOLET_50,
			VIOLET_100,
			VIOLET_200,
			VIOLET_300,
			VIOLET_400,
			VIOLET_500,
			VIOLET_600,
			VIOLET_700,
			VIOLET_800,
			VIOLET_900,
			VIOLET_950,
			YELLOW_50,
			YELLOW_100,
			YELLOW_200,
			YELLOW_300,
			YELLOW_400,
			YELLOW_500,
			YELLOW_600,
			YELLOW_700,
			YELLOW_800,
			YELLOW_900,
			YELLOW_950,
			ZINC_50,
			ZINC_100,
			ZINC_200,
			ZINC_300,
			ZINC_400,
			ZINC_500,
			ZINC_600,
			ZINC_700,
			ZINC_800,
			ZINC_900,
			ZINC_950,
		]
	}
}
