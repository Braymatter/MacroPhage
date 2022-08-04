use bevy::prelude::*;
use iyes_loopless::prelude::*;

use super::{Force, NodeId, PhageType, Recombinator};

/// The order in which game state is processed during the transmission phase
pub enum TransmissionSubPhase {
    /// Distribute any qubits from enriched nodes to the occupying player
    DistributeQubits,

    /// Activate recombinators, this should happen in the same order for
    /// every client. Activating based on map position is probably the
    /// safest way to ensure this for now
    ReCombinate,

    /// Move any phage that are moving into unoccupied or uncontested cells
    /// A cell is considered contested when phage from opposing Forces
    /// are attempting to occupy a cell at the same time. If two allied cells
    /// attempt to move into the same cell the cell that is farthest from the
    /// nexus will move while the other cell will stay put (Or go to next best?)
    Movement,

    /// When two Phage on opposing forces are attempting to occupy the same cell
    /// Resolve via combat with the victor occupying the cell
    Combat,
}

/// Events related to updates to game state that occur during the transmission
/// phase. Intended to be consumed by other systems to display visual cues
pub enum TransmissionEvents {
    /// A phage moved from one cell to another
    PhageMoved { from: NodeId, to: NodeId },

    /// Combat occured between two phages competing for a cell
    CombatOccured {
        cell: NodeId,
        victor: (Force, PhageType),
        lose: (Force, PhageType),
    },

    /// A recombinator triggered
    RecombinatorTriggered {
        cell: NodeId,
        recombinator: Recombinator,
    },

    /// Qubits were distributed to a player from either a Recombinator or a node
    QubitsDistributed {
        recipient: Force,
        qty: u32,
        source: NodeId,
    },
}

/// This event is received and used to prompt for the next phase of the game
/// Driving the game via event allows easy-swapping of behavior related to how
/// the game is advanced. For example on a timer or turn based
pub struct AdvanceGamePhaseEvent {}

/// A way for the GameRunner to notify other systems about events related to the game as a whole
pub enum GameRunnerEvent {
    GameBegun,
    GameCompleted { victor: Force },
    PhaseEntered { phase: GamePhase },
    PhaseExited { phase: GamePhase },
}

/// Procedure for running gameplay
pub enum GamePhase {
    /// Players can submit mutations to the board which take effect immediately
    /// One mutation can be submitted per tile, per mutation phase, per player.
    /// In general mutations can only be applied to cells the player is occupying
    MutationPhase,

    /// Brief interval after the mutation phase for network messages to catch up
    /// and sync client state as applicable
    InterstitialPhase,

    ///This is where the meat of the 'gameplay' happens. The phage board will
    /// update based on its state. There are no side effects and the result of
    /// this phase should be a pure function, where state-in = state-out; every
    /// time. No calls to Rand!
    TransmissionPhase,
}

/// This plugin is responsible for updating the game state
pub struct MacroPhageGamerunnerPlugin {}
pub struct GameRunnerRes {
    pub run_game: bool,
    pub game_phase: GamePhase,
}

impl Default for GameRunnerRes {
    fn default() -> Self {
        GameRunnerRes {
            run_game: false,
            game_phase: GamePhase::MutationPhase,
        }
    }
}

impl Plugin for MacroPhageGamerunnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_game);
        app.add_event::<AdvanceGamePhaseEvent>();
        app.add_event::<TransmissionEvents>();
        app.add_event::<GameRunnerEvent>();
        app.insert_resource(GameRunnerRes::default());
        app.add_system(run_game.run_if(should_run_game));
    }
}

fn init_game() {
    info!("Initializing Game Plugin");
}

fn should_run_game(runner: Res<GameRunnerRes>) -> bool {
    runner.run_game
}

fn run_game(
    mut phase_events: EventReader<AdvanceGamePhaseEvent>,
    mut runner: ResMut<GameRunnerRes>,
    mut runner_events: EventWriter<GameRunnerEvent>,
) {
    if let Some(_adv_phase) = phase_events.iter().last() {
        runner.game_phase = match runner.game_phase {
            GamePhase::MutationPhase => GamePhase::InterstitialPhase,
            GamePhase::InterstitialPhase => GamePhase::TransmissionPhase,
            GamePhase::TransmissionPhase => GamePhase::MutationPhase,
        };
    }
}
