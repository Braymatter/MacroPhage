mod mainmenu;
mod settingsmenu;


use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use leafwing_input_manager::prelude::ActionState;
use iyes_loopless::prelude::*;

use crate::game::controller::PlayerAction;

use self::settingsmenu::{binding_window_system};

pub fn ui_example(mut egui_context: ResMut<EguiContext>, actions: Query<&ActionState<PlayerAction>>) {
    let actions = actions.single();

    if actions.pressed(PlayerAction::Scream) {
        egui::Window::new("AHHHH").show(egui_context.ctx_mut(), |ui| {
            ui.label("AHHHH");
        });
    }
}

enum UIState{
    MainMenu,
    Settings,
    Lobby,
    JoinLobby,
    JoiningLobby{lobby_id: String}
}

pub struct UIStateRes{
    current_state: UIState
}

pub struct UIStatePlugin;
impl Plugin for UIStatePlugin{
    fn build(&self, app: &mut App) {
        app.insert_resource(UIStateRes{current_state: UIState::MainMenu})
        .add_system(self::mainmenu::main_menu.run_if(show_main_menu))
        .add_system(self::settingsmenu::controls_window.run_if(show_settings_menu))
        .add_system(binding_window_system);
    }
}

fn show_main_menu(ui_state: Res<UIStateRes>) -> bool{
    matches!(&ui_state.current_state, UIState::MainMenu)
}

fn show_settings_menu(ui_state: Res<UIStateRes>) -> bool{
    matches!(&ui_state.current_state, UIState::Settings)
}

