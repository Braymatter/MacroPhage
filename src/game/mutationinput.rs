use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;
use leafwing_input_manager::prelude::ActionState;

use crate::game::{Force, Mutation};

use super::{controller::PlayerAction, NodeId, PlayerMutationEvent, Vector};

//TODO it would be nice to have an enum here instead of a dummy mutation
//the internal values of the mutation aren't used and are just dummy values here
#[derive(Deref, DerefMut, Default)]
pub struct MutationSelection(Mutation);

pub fn mutation_selection(
    mut selected: ResMut<MutationSelection>,
    actions: Query<&ActionState<PlayerAction>>,
) {
    let actions = actions.single();

    if actions.just_pressed(PlayerAction::HotKey1) {
        //XXX these are not the values used for cost or id, this just sets the selected mutation!
        selected.0 = Mutation::TriggerRecombinator {
            target: NodeId::default(),
            cost: 0,
        };
    }
    if actions.just_pressed(PlayerAction::HotKey2) {
        //XXX these are not the values used for cost or id, this just sets the selected mutation!
        selected.0 = Mutation::AddVector {
            relation: Vector::default(),
            cost: 0,
        };
    }
    if actions.just_pressed(PlayerAction::HotKey3) {
        //XXX these are not the values used for cost or id, this just sets the selected mutation!
        selected.0 = Mutation::RemoveVector {
            relation: Vector::default(),
            cost: 0,
        };
    }
    if actions.just_pressed(PlayerAction::HotKey4) {
        //XXX these are not the values used for cost or id, this just sets the selected mutation!
        selected.0 = Mutation::ChangeReplicatorType {
            replicator: NodeId::default(),
            new_type: super::PhageType::UV,
            cost: 0,
        };
    }
}

pub fn mutation_input(
    mut writer: EventWriter<PlayerMutationEvent>,
    mut picking_events: EventReader<PickingEvent>,
    nodes: Query<&crate::game::Node>,
    selected: Res<MutationSelection>,
    //TODO use a local to track the internal state of previous clicks for multi click events
    mut previous_click: Local<Option<NodeId>>,
) {
    for event in picking_events.iter() {
        if let PickingEvent::Clicked(ent) = event {
            if let Ok(node) = nodes.get(*ent) {
                match selected.0 {
                    Mutation::TriggerRecombinator { .. } => {
                        info!(
                            "Clicked {:?} with trigger recombinator, Todo determine cost",
                            node.id
                        );
                        *previous_click = None;
                        writer.send(PlayerMutationEvent {
                            mutation: Mutation::TriggerRecombinator {
                                target: node.id,
                                cost: 10,
                            },
                            force: node.force.clone(),
                        });
                    }
                    Mutation::AddVector { .. } => {
                        if let Some(prev_id) = *previous_click {
                            info!(
                                "Clicked {:?}, {:?} with Add Vector, Todo determine cost",
                                prev_id, node.id
                            );
                            writer.send(PlayerMutationEvent {
                                mutation: Mutation::AddVector {
                                    relation: Vector::new(prev_id, node.id),
                                    cost: 10,
                                },
                                force: node.force.clone(),
                            });
                            *previous_click = None;
                        } else {
                            info!(
                                "Clicked {:?} with Add Vector, waiting for second click",
                                node.id
                            );
                            *previous_click = Some(node.id);
                        }
                    }
                    Mutation::RemoveVector { .. } => {
                        if let Some(prev_id) = *previous_click {
                            info!(
                                "Clicked {:?}, {:?} with Remove Vector, Todo determine cost",
                                prev_id, node.id
                            );
                            writer.send(PlayerMutationEvent {
                                mutation: Mutation::RemoveVector {
                                    relation: Vector::new(prev_id, node.id),
                                    cost: 10,
                                },
                                force: node.force.clone(),
                            });
                            *previous_click = None;
                        } else {
                            info!(
                                "Clicked {:?} with Remove Vector, waiting for second click",
                                node.id
                            );
                            *previous_click = Some(node.id);
                        }
                    }
                    Mutation::ChangeReplicatorType { .. } => {
                        info!(
                            "Clicked {:?} with Change Replicator Type, Todo determine cost and new type",
                            node.id
                        );
                        *previous_click = None;
                        writer.send(PlayerMutationEvent {
                            //FIXME how to select replicator type
                            mutation: Mutation::ChangeReplicatorType {
                                replicator: node.id,
                                new_type: crate::game::PhageType::UV,
                                cost: 10,
                            },
                            force: node.force.clone(),
                        });
                    }
                }
            } else {
                error!("Something not a node was clicked :(");
            }
        }
    }
}
