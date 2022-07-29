use bevy::prelude::*;

fn main() {
    App::new()
        .add_startup_system(startup_system)
        .run();
}

fn startup_system(){
    println!("Starting Application");
}