use crate::apps::shared::*;

use super::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::draw::{Fill, Stroke};
use leafwing_input_manager::{
	action_diff::{ActionDiff, ActionDiffEvent},
	prelude::*,
};
use lightyear::prelude::*;

pub(super) struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
	fn build(&self, app: &mut App) {
		use client::ComponentSyncMode::*;
		use ChannelDirection::*;

		app.add_plugins((
			VerifyActionPlugin::<PenDraw>::default(),
			VerifyActionPlugin::<ObjectBirNet>::default(),
			NetToLocalPlugin::<StrokeNet, Stroke>::default(),
			NetToLocalPlugin::<FillNet, Fill>::default(),
		));

		app.register_message::<RequestingPointRelation>(ClientToServer)
			.add_map_entities();

		app.register_message::<ConnectRelations<PointToPoint>>(ServerToClient)
			.add_map_entities();

		app.add_event::<ActionDiffEvent<VerifyAction>>();
		app.add_plugins(InputManagerPlugin::<VerifyAction>::default());
		app.register_message::<Vec<ActionDiff<VerifyAction>>>(ClientToServer);

		app.add_event::<ActionDiffEvent<ClientAction>>();
		app.add_plugins(InputManagerPlugin::<ClientAction>::default());
		app.register_message::<Vec<ActionDiff<ClientAction>>>(ClientToServer);
		app.register_message::<ToClientEntDataEvent<Vec<ActionDiff<ClientAction>>>>(ServerToClient)
			.add_map_entities();

		app.register_message::<MessageCtx>(Bidirectional);
		app.register_message::<MarkerType>(Bidirectional);
		app.register_message::<ImageNetwork>(ClientToServer);
		app.register_message::<ObjectActionToServer>(ClientToServer)
			.add_map_entities();
		app.register_message::<PerActionNet>(ClientToServer)
			.add_map_entities();

		app.register_message::<RequestImageData>(ClientToServer)
			.add_map_entities();
		app.register_message::<ApplyChange>(Bidirectional)
			.add_map_entities();
		app.register_message::<MovedPoint>(ClientToServer);
		app.register_message::<ReceiveImageData>(ServerToClient)
			.add_map_entities();

		app.add_channel::<MainChannel>(ChannelSettings {
			mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
			..default()
		});
		app.add_channel::<MessageChannel>(ChannelSettings {
			mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
			priority: 3.0,
			..default()
		});
		app.add_channel::<DisplayChannel>(ChannelSettings {
			mode: ChannelMode::UnorderedUnreliable,
			priority: 5.0,
			..default()
		});

		// Global stat
		app.register_component::<ObjectOpacity>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<SharingName>(Bidirectional)
			.add_prediction(Full);

		// User [UserDefaultBundle]
		app.register_component::<UserId>(Bidirectional)
			.add_prediction(Once);
		app.register_component::<BrushScale>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<DrawingSpacing>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<BrushRef>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<BeginSelectPoint>(Bidirectional)
			.add_prediction(Simple);
		app.register_component::<Selection>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<CursorPos>(Bidirectional)
			.add_prediction(Simple);
		app.register_component::<PaintInk>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<HardEdgeDraw>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<DrawType>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<BlurScale>(Bidirectional)
			.add_prediction(Full);

		app.register_component::<SelectedObject>(Bidirectional)
			.add_prediction(Full)
			.add_map_entities();

		// Object [ObjectBundle]
		app.register_component::<ObjectZLayer>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<ObjectPosition>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<MoveLock>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<ObjectOwner>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<ObjectAccess>(ServerToClient)
			.add_prediction(Full);

		// Image Object
		app.register_component::<ObjectImage>(ServerToClient)
			.add_prediction(Once);
		app.register_component::<PixelLock>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<AlphaLock>(ServerToClient)
			.add_prediction(Full);

		// Svg Object
		app.register_component::<ObjectPath>(ServerToClient)
			.add_prediction(Once);
		app.register_component::<ObjectPoint>(ServerToClient)
			.add_prediction(Once);
		app.register_component::<PointType>(ServerToClient)
			.add_prediction(Full);
		app.register_component::<PathClose>(ServerToClient)
			.add_prediction(Simple);
		// Text
		app.register_component::<TextValue>(ServerToClient)
			.add_prediction(Full);

		// Expermental
		app.register_component::<ResizePinPoint>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<ScaleAction>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<ResizeKind>(Bidirectional)
			.add_prediction(Full);
		app.register_component::<ScalePosition>(Bidirectional)
			.add_prediction(Full);
	}
}

#[derive(Channel)]
pub struct MainChannel;

#[derive(Channel)]
pub struct MessageChannel;

#[derive(Channel)]
pub struct DisplayChannel;
