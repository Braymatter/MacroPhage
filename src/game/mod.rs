use crate::util::{
    modelloading::{spawn_model, NodeTenentAssets},
    ColorPalette,
};

use self::controller::PlayerAction;
use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

pub mod controller;
pub mod gamerunner;
pub mod map;
pub mod mutationinput;

#[derive(Component, Serialize, Deserialize, Clone)]
pub enum Mutation {
    TriggerRecombinator {
        target: NodeId,
        cost: u32,
    },
    AddVector {
        relation: Vector,
        cost: u32,
    },
    RemoveVector {
        relation: Vector,
        cost: u32,
    },
    ChangeReplicatorType {
        replicator: NodeId,
        new_type: PhageType,
        cost: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PhageType {
    UV,
    Electro,
    Sonic,
    Any,
}

#[derive(
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Copy,
    Clone,
    Deref,
    DerefMut,
    PartialOrd,
    Ord,
    Debug,
)]
pub struct NodeId(u32);

#[derive(Serialize, Deserialize)]
pub enum RecombinatorEffect {
    DestroyPhageWithinRange(u32),
    GiveOccupierQubits { amt: u32 },
    GiveOccupierAdvantage,
    DestroyOccupierIfType { phage_type: PhageType },
    PullPhageForCombat(Vector),
}

#[derive(Serialize, Deserialize)]
pub enum RecombinatorTriggers {
    NumberOfTransmissionPhases { phases: u32 },
    PhageEntered,
    PhageExited,
    CombatOccured,
    Qubits { qty: u32 },
    VectorAdded { dest: NodeId },
    VectorRemoved { dest: NodeId },
    OpposingNeighbors,
}

#[derive(Component, Serialize, Deserialize)]
pub struct Recombinator {
    trigger: RecombinatorTriggers,
    effect: RecombinatorEffect,
}

/// Describes a discrete location on the map that can be connected to other locations
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: NodeId,
    pub position: Vec3,
    pub force: Force,
    pub tenant: NodeTenant,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum NodeTenant {
    Cell { cell: Cell },
    Replicator { replicator: Replicator },
    Nexus { nexus: Nexus },
    Generator { generator: Generator },
}
pub enum CellType {
    Transit,
    Recombinator { recombinator: Recombinator },
    Generator { qubits_per_phase: u32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cell {
    occupant: Option<Occupant>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Replicator {
    output: PhageType,

    ///How many intervals/transmission phases this replicator takes to produce a phage
    speed: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Nexus {}

#[derive(Serialize, Deserialize, Clone)]
pub struct Generator {
    amt: u32,
    speed: u32,
}

#[derive(Serialize, Deserialize)]
pub struct VectorId(u32);

/// Defines the team and occupying phage type
#[derive(Serialize, Deserialize, Clone)]
pub struct Occupant(Force, PhageType);

/// Defines a relationship between two cells
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Vector(pub NodeId, pub NodeId);

impl Vector {
    pub fn new(id_1: NodeId, id_2: NodeId) -> Vector {
        if id_1 < id_2 {
            Vector(id_1, id_2)
        } else {
            Vector(id_2, id_1)
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.1 > self.0
    }
}

/// Defines a Force
#[derive(Serialize, Deserialize, Clone)]
pub struct Force(u32);

impl Force {
    pub fn color(&self) -> Color {
        match self.0 {
            0 => ColorPalette::ForceBlue.into(),
            1 => ColorPalette::ForceGreen.into(),
            2 => ColorPalette::ForceOrange.into(),
            3 => ColorPalette::ForceRed.into(),
            4 => ColorPalette::ForceYellow.into(),
            5 => ColorPalette::ForceBlack.into(),
            _ => ColorPalette::ForceWhite.into(),
        }
    }
}

pub enum GameMove {
    MovePhage {
        vector: Vector,
    },
    Mutate {
        mutation: Mutation,
    },
    GiveQubits {
        source: Force,
        dest: Force,
        qty: u32,
    },
}

/// Data Only representation of a Game Map, Game acts as a pure state-machine
#[derive(Component, Serialize, Deserialize)]
pub struct GameState {
    pub nodes: HashMap<NodeId, Node>,
    //connections: HashMap<NodeId, Vec<VectorId>>,
    pub vectors: Vec<Vector>,
    pub num_players: u32,
    pub name: String,
    pub next_free_id: NodeId,
}

impl GameState {
    pub fn create_node(&mut self, force: Force, position: Vec3) -> NodeId {
        self.nodes.insert(
            self.next_free_id,
            Node {
                id: self.next_free_id,
                position,
                force,
                tenant: NodeTenant::Cell {
                    cell: Cell { occupant: None },
                },
            },
        );

        let to_return = self.next_free_id;
        self.next_free_id = NodeId(self.next_free_id.0 + 1);
        to_return
    }

    pub fn add_vector(&mut self, vector: Vector) -> Result<(), PlayerActionError> {
        if self.vector_exists(vector) {
            return Err(PlayerActionError::VectorExists);
        }

        if !self.nodes.contains_key(&vector.0) {
            return Err(PlayerActionError::NodeIdDoesNotExist(vector.0));
        }

        if !self.nodes.contains_key(&vector.1) {
            return Err(PlayerActionError::NodeIdDoesNotExist(vector.1));
        }

        //Swap in place so the lowest node id is first
        if vector.0 > vector.1 {
            self.vectors.push(Vector(vector.1, vector.0));
        } else {
            self.vectors.push(vector);
        }

        Ok(())
    }

    pub fn vector_exists(&self, vector: Vector) -> bool {
        if vector.0 > vector.1 {
            self.vectors.contains(&Vector(vector.1, vector.0))
        } else {
            self.vectors.contains(&vector)
        }
    }

    pub fn remove_vector(&mut self, to_remove: Vector) -> Result<(), PlayerActionError> {
        if self.vector_exists(to_remove) {
            // https://stackoverflow.com/questions/26243025/remove-an-element-from-a-vector

            match self.vectors.iter().position(|item| to_remove == *item) {
                Some(index) => {
                    self.vectors.remove(index); // No indexes on this!
                    return Ok(());
                }
                None => return Err(PlayerActionError::VectorDoesNotExist { vector: to_remove }),
            }
        }

        Err(PlayerActionError::VectorDoesNotExist { vector: to_remove })
    }

    pub fn get_all_neighbors(&self, id: NodeId) -> Vec<NodeId> {
        let mut to_return = Vec::default();
        for vector in self.vectors.iter() {
            if vector.0 == id {
                to_return.push(vector.1);
            }
            if vector.1 == id {
                to_return.push(vector.0);
            }
        }
        to_return
    }

    pub fn spawn_node(
        &self,
        node: &Node,
        commands: &mut Commands,
        node_assets: &Res<NodeTenentAssets>,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Entity {
        match &node.tenant {
            NodeTenant::Cell { cell: _ } => {
                spawn_model(commands, node_assets.cell.clone(), meshes, node)
            }
            NodeTenant::Replicator { replicator: _ } => {
                spawn_model(commands, node_assets.replicator.clone(), meshes, node)
            }
            NodeTenant::Nexus { nexus: _ } => {
                spawn_model(commands, node_assets.nexus.clone(), meshes, node)
            }
            //FIXME this needs to change when the model exists
            NodeTenant::Generator { generator: _ } => {
                spawn_model(commands, node_assets.cell.clone(), meshes, node)
            }
        }
    }
}

pub struct LevelManagerRes {
    pub current_level: Option<String>,
}

#[derive(Debug)]
pub enum PlayerActionError {
    VectorExists,
    VectorDoesNotExist { vector: Vector },
    BadVectorFormat,
    NodeIdDoesNotExist(NodeId),
}

#[derive(Clone)]
pub struct PlayerMutationEvent {
    pub mutation: Mutation,
    pub force: Force,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MutationFailed {
    pub mutation: Mutation,
}

pub fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(InputManagerBundle::<PlayerAction> {
            //TODO: Can we export this bundle from controller.rs?
            input_map: InputMap::new([
                (KeyCode::Space, PlayerAction::Scream),
                (KeyCode::Escape, PlayerAction::OpenKeyBinds),
                (KeyCode::Grave, PlayerAction::ToggleInspector),
                (KeyCode::W, PlayerAction::PanUp),
                (KeyCode::S, PlayerAction::PanDown),
                (KeyCode::A, PlayerAction::PanLeft),
                (KeyCode::D, PlayerAction::PanRight),
                (KeyCode::Key1, PlayerAction::HotKey1),
                (KeyCode::Key2, PlayerAction::HotKey2),
                (KeyCode::Key3, PlayerAction::HotKey3),
                (KeyCode::Key4, PlayerAction::HotKey4),
                (KeyCode::PageUp, PlayerAction::ZoomIn),
                (KeyCode::PageDown, PlayerAction::ZoomOut),
                (KeyCode::Left, PlayerAction::PanLeft),
                (KeyCode::Right, PlayerAction::PanRight),
                (KeyCode::Up, PlayerAction::PanUp),
                (KeyCode::Down, PlayerAction::PanDown),
            ]),
            ..default()
        })
        .insert(Name::new("Player"));
}
