use bevy::{log::LogPlugin, prelude::*};
use bevy_renet::RenetServerPlugin;
use macrophage::net::gamehost::GameHostPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugin(LogPlugin);
    app.add_plugin(RenetServerPlugin);
    app.add_plugin(GameHostPlugin);

    app.run();
}
