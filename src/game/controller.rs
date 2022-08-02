use derive_more::Display;
use leafwing_input_manager::{
    Actionlike,
};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Display)]
pub enum PlayerAction {
    OpenKeyBinds,
    ToggleInspector,
    Scream,
    /// Selects the players Nexus
    SelectNexus,

    /// Selects the previous node that was selected if applicable
    SelectPrevNode,

    /// Selects the next node (after select previous has been used)
    SelectNextNode,

    /// Selects a replicator, when pressed again selects the next replicator in the list, loops back to the beginning of the list
    SelectReplicator,

    /// Opens the Qubit Dialog
    OpenQubitTradePanel,

    /// Triggers the Selected Recombinator
    TriggerRecombinator,

    OpenOptionsMenu,

    HotKey1,
    HotKey2,
    HotKey3,
    HotKey4,

    PanUp,
    PanDown,
    PanLeft,
    PanRight,
    ZoomIn,
    ZoomOut
}
