use bevy::{prelude::*, utils::HashMap};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Infrastructure(InfrastructureMessage),
    GamePlay,
}

#[derive(Serialize, Deserialize)]
pub enum NetworkingState {
    Host(HostState),
    Client(ClientState),
}

#[derive(Serialize, Deserialize)]
pub struct ClientId;

#[derive(Serialize, Deserialize)]
pub struct HostState {
    clients: Vec<ClientId>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientState;

#[derive(Serialize, Deserialize)]
pub enum InfrastructureMessage {
    Ping,
    HostMatch,
    LobbyReserved(String),
    JoinMatch(String),
}

#[derive(Serialize, Deserialize)]
pub enum Mutation {
    TriggerRecombinator {
        target: u32,
        cost: u32,
    },
    AddVector {
        relation: NodeRelation,
        cost: u32,
    },
    RemoveVector {
        relation: NodeRelation,
        cost: u32,
    },
    ChangeReplicatorType {
        replicator: u32,
        new_type: PhageType,
        cost: u32,
    },
}

#[derive(Serialize, Deserialize)]
pub struct NodeRelation(u32, u32);

#[derive(Serialize, Deserialize)]
pub enum PhageType {
    UV,
    Electro,
    Sonic,
    Any,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, Deref, DerefMut, PartialOrd, Ord, Debug)]
pub struct NodeId(u32);

#[derive(Serialize, Deserialize)]
pub enum RecombinatorEffect {
    DestroyPhageWithinRange(u32),
    GiveOccupierQubits { amt: u32 },
    GiveOccupierAdvantage,
    DestroyOccupierIfType { phage_type: PhageType },
    PullPhageForCombat(NodeRelation),
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
#[derive(Component, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub position: Vec3,
    pub tenant: NodeTenant,
}

#[derive(Serialize, Deserialize)]
pub enum NodeTenant {
    Cell { cell: Cell },
    Replicator { replicator: Replicator },
    Nexus { nexus: Nexus },
}
pub enum CellType {
    Transit,
    Recombinator { recombinator: Recombinator },
    Generator { qubits_per_phase: u32 },
}

#[derive(Serialize, Deserialize)]
pub struct Cell {
    occupant: Option<Occupant>,
}

#[derive(Serialize, Deserialize)]
pub struct Replicator {
    output: PhageType,

    ///How many intervals/transmission phases this replicator takes to produce a phage
    speed: u32,
    force: Force,
}

#[derive(Serialize, Deserialize)]
pub struct Nexus {
    force: Force,
    vectors: Vec<NodeId>,
}

#[derive(Serialize, Deserialize)]
pub struct VectorId(u32);

/// Defines the team and occupying phage type
#[derive(Serialize, Deserialize)]
pub struct Occupant(Force, PhageType);

/// Defines a relationship between two cells
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Serialize, Deserialize)]
pub struct Force(u32);

//Map should be a resource not component?
/// Data Only representation of a Game Map
#[derive(Component, Serialize, Deserialize)]
pub struct GameMap {
    pub nodes: HashMap<NodeId, Node>,
    //connections: HashMap<NodeId, Vec<VectorId>>,
    pub vectors: Vec<Vector>,
    pub num_players: u32,
    pub name: String,
	pub next_free_id: NodeId
}

#[derive(Debug)]
pub enum PlayerActionError {
	VectorExists,
	VectorDoesNotExist{vector: Vector},
	BadVectorFormat,
	NodeIdDoesNotExist (NodeId)
}

impl GameMap {
	pub fn create_node(&mut self, position: Vec3) -> NodeId {
		self.nodes.insert(self.next_free_id, Node {
			id: self.next_free_id,
			position,
			tenant: NodeTenant::Cell { cell: Cell{occupant: None} }
		});

		let to_return = self.next_free_id;
		self.next_free_id = NodeId(self.next_free_id.0 + 1);
		to_return
	}



	pub fn add_vector(&mut self, vector: Vector) -> Result<(), PlayerActionError>  {
		if self.vector_exists(vector) {
			return Err(PlayerActionError::VectorExists);
		}

		if !self.nodes.contains_key(&vector.0) {
			return Err(PlayerActionError::NodeIdDoesNotExist(vector.0));
		}

		if !self.nodes.contains_key(&vector.1) {
			return Err(PlayerActionError::NodeIdDoesNotExist(vector.1));
		}

		self.vectors.push(vector);
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
			
			match self.vectors.iter().position(|item| {to_remove == *item}) {
				Some(index) => {
					self.vectors.remove(index); // No indexes on this!
					return Ok(());
				},
				None => return Err(PlayerActionError::VectorDoesNotExist { vector: to_remove })
			}
		}

		Err(PlayerActionError::VectorDoesNotExist{vector: to_remove})
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
}

#[cfg(test)]
mod tests {
	use super::*;
	
    #[test]
    pub fn create_test_map() {
        use std::fs::File;
        use std::io::prelude::*;

        let nodes = HashMap::new();
        let vectors = vec![];
        let num_players = 5;

        let name: String = "Hello Map".to_string();
        let mut map = GameMap {
            nodes,
			next_free_id: NodeId(0),
            vectors,
            name,
            num_players,
        };

		let node_1 = map.create_node(Vec3::ZERO);
		let node_2 = map.create_node(Vec3::new(1., 2., 3.));
		let node_3 = map.create_node(Vec3::new(-2., -3., 4.));
		let node_4 = map.create_node(Vec3::new(-20., -30., 40.));
		map.add_vector(Vector::new(node_4, node_1)).unwrap();
		
		map.add_vector(Vector::new(node_1, node_2)).unwrap();
		map.remove_vector(Vector::new(node_1, node_2)).unwrap();
		map.add_vector(Vector::new(node_3, node_1)).unwrap();
		map.remove_vector(Vector::new(node_1, node_3)).unwrap();

		map.add_vector(Vector::new(node_3, node_2)).unwrap();
		map.add_vector(Vector::new(node_3, node_1)).unwrap();
		map.add_vector(Vector::new(node_1, node_2)).unwrap();
		map.remove_vector(Vector::new(node_3, node_2)).unwrap();

		assert!(map.vectors.len()==3);

        let map_json = serde_json::to_string(&map).unwrap();
        let mut input = File::create("assets/test_map.json").unwrap();
        // https://doc.rust-lang.org/std/fs/struct.File.html
        input.write_all(map_json.as_bytes()).unwrap();

        println!("File Created Luv u -- ur pc");
    }
}

pub fn spawn_test_map(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let path = "assets/test_map.json";

    use std::fs::File;
    let file = File::open(path).unwrap();
    use std::io::BufReader;
    let reader = BufReader::new(file);

    let map: GameMap = serde_json::from_reader(reader).unwrap();

    let map_ent = commands
        .spawn_bundle(TransformBundle::default())
        .insert(Name::new("Map"))
        .id();
    let mut node_ents = Vec::default();
    for node in map.nodes.values() {
        node_ents.push(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 1.0,
                        subdivisions: 5,
                    })),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(node.position),
                    ..default()
                })
                .insert(Name::new("Node"))
                .id(),
        );
    }
    let mut vector_ents = Vec::default();
    for vector in map.vectors.iter() {
        let node_0 = map.nodes.get(&vector.0).unwrap();
        let node_1 = map.nodes.get(&vector.1).unwrap();

        let pos = (node_0.position + node_1.position) / 2.0;
        let mut transform = Transform::from_translation(pos).looking_at(node_1.position, Vec3::Y);
        transform.scale.z = Vec3::distance(node_0.position, node_1.position) * 2.0;

        vector_ents.push(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: 0.5,
                    })),
                    material: materials.add(Color::rgb(0.8, 0.1, 0.1).into()),
                    transform,
                    ..default()
                })
                .insert(Name::new("Vector"))
                .id(),
        );
    }
    commands.entity(map_ent).push_children(&node_ents);
    commands.entity(map_ent).push_children(&vector_ents);
}

// Sphere = Cell
// TetraHedron = Replicator
// Pyramid = ReCombinator
// 3d Line = Vector
// Cylinder = Nexus
// Torus = Generator