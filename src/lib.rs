use bevy::prelude::*;
use bevy::utils::HashMap;

pub enum Message {
	Infrastructure(InfrastructureMessage),
	GamePlay
}

pub enum NetworkingState {
	Host(HostState),
	Client(ClientState),
}

pub struct ClientId;

pub struct HostState {
	clients: Vec<ClientId>
}

pub struct ClientState;

pub enum InfrastructureMessage {
	Ping,
	HostMatch,
	LobbyReserved(String),
	JoinMatch(String),
}

pub enum Mutation{
	TriggerRecombinator{ target: u32, cost: u32 },
	AddVector{relation: NodeRelation, cost: u32},
	RemoveVector{relation: NodeRelation, cost: u32},
	ChangeReplicatorType{replicator: u32, new_type: PhageType, cost: u32}
}

pub struct NodeRelation (u32, u32);

pub enum PhageType{
    UV,
    Electro,
    Sonic,
    Any
}

/// Defines the team and occupying phage type
pub struct Occupant(Force, PhageType);

/// Defines a relationship between two cells
pub struct Vector(u32, u32);

/// Defines a Force
pub struct Force(u32);
pub struct CellId(u32);

pub enum RecombinatorEffect {
	DestroyPhageWithinRange(u32),
	GiveOccupierQubits{ amt : u32},
	GiveOccupierAdvantage,
	DestroyOccupierIfType { phage_type: PhageType },
	PullPhageForCombat(NodeRelation),
}

pub enum RecombinatorTriggers {
	NumberOfTransmissionPhases{phases: u32},
	PhageEntered,
	PhageExited,
	CombatOccured,
	Qubits {qty: u32},
	VectorAdded {dest: CellId},
	VectorRemoved {dest: CellId},
	OpposingNeighbors
}

#[derive(Component)]
pub struct Recombinator{
    trigger: RecombinatorTriggers, 
    effect: RecombinatorEffect
}

#[derive(Component)]
pub struct Cell{
	occupant: Option<Occupant>,
	id: CellId,
	vectors: Vec<CellId>,
	position: Vec3
}

pub enum CellType{
	Transit,
	Recombinator{recombinator: Recombinator},
	Generator{qubits_per_phase: u32}
}

pub struct Replicator{

}

pub struct Nexus{

}
//Map should be a resource not component?
#[derive(Component)]
pub struct Map {
	cells: HashMap<CellId, Cell>,
	nexuses: HashMap<CellId, Nexus>,
	replicator: HashMap<CellId, Replicator>,
	num_players: u32,
	name: String,
}

