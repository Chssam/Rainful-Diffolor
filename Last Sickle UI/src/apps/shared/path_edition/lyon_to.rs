pub use lyon_to_raqote::*;
pub use lyon_to_zeno::*;

pub mod lyon_to_zeno {
	use bevy_prototype_lyon::prelude::*;
	use zeno::*;
	pub trait LyonToZeno {
		type Other;
		fn to_zeno(&self) -> Self::Other;
	}
	impl LyonToZeno for FillRule {
		type Other = zeno::Fill;
		fn to_zeno(&self) -> Self::Other {
			match self {
				FillRule::EvenOdd => zeno::Fill::EvenOdd,
				FillRule::NonZero => zeno::Fill::NonZero,
			}
		}
	}

	impl LyonToZeno for LineCap {
		type Other = Cap;
		fn to_zeno(&self) -> Self::Other {
			match self {
				LineCap::Butt => Cap::Butt,
				LineCap::Square => Cap::Square,
				LineCap::Round => Cap::Round,
			}
		}
	}

	impl LyonToZeno for LineJoin {
		type Other = Join;
		fn to_zeno(&self) -> Self::Other {
			match self {
				LineJoin::Miter => Join::Miter,
				LineJoin::MiterClip => Join::Miter,
				LineJoin::Round => Join::Round,
				LineJoin::Bevel => Join::Bevel,
			}
		}
	}
}

pub mod lyon_to_raqote {
	use bevy_prototype_lyon::prelude::*;
	use raqote::{LineCap as RaLineCap, LineJoin as RaLineJoin};
	pub trait LyonToRaqote {
		type Other;
		fn to_raqote(&self) -> Self::Other;
	}
	impl LyonToRaqote for LineCap {
		type Other = RaLineCap;
		fn to_raqote(&self) -> Self::Other {
			match self {
				LineCap::Butt => RaLineCap::Butt,
				LineCap::Square => RaLineCap::Square,
				LineCap::Round => RaLineCap::Round,
			}
		}
	}

	impl LyonToRaqote for LineJoin {
		type Other = RaLineJoin;
		fn to_raqote(&self) -> Self::Other {
			match self {
				LineJoin::Miter => RaLineJoin::Miter,
				LineJoin::MiterClip => RaLineJoin::Miter,
				LineJoin::Round => RaLineJoin::Round,
				LineJoin::Bevel => RaLineJoin::Bevel,
			}
		}
	}
}
