use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;

use crate::game::{Force, Mutation};

use super::PlayerMutationEvent;

pub fn mutation_input(
    mut writer: EventWriter<PlayerMutationEvent>,
    mut picking_events: EventReader<PickingEvent>,
    nodes: Query<&crate::game::Node>,
    //TODO use a local to track the internal state of previous clicks for multi click events
) {
    for event in picking_events.iter() {
        if let PickingEvent::Clicked(ent) = event {
            if let Ok(node) = nodes.get(*ent) {
                info!("Clicked {:?}, Todo determine force and cost", node.id);
                writer.send(PlayerMutationEvent {
                    mutation: Mutation::TriggerRecombinator {
                        target: node.id,
                        cost: 10,
                    },
                    force: Force(0),
                });
            } else {
                error!("Something not a node was clicked :(");
            }
        }
    }
}
