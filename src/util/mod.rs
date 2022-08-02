use bevy::prelude::Plugin;
pub mod camera;
pub struct MapManifest {
    pub map_files: Vec<String>,
}
pub struct MacroUtils {}

impl Plugin for MacroUtils {
    fn build(&self, app: &mut bevy::prelude::App) {
        let maps_dir =
            std::fs::read_dir("./assets/maps/").expect("Could not read files in ./assets/maps/");
        let mut map_files: Vec<String> = vec![];

        for map_file in maps_dir {
            map_files.push(
                map_file
                    .expect("Weird thing when iterating over map directory entries")
                    .file_name()
                    .into_string()
                    .expect("Could not convert OS String to string"),
            );
        }

        app.insert_resource(MapManifest { map_files });
    }
}

impl MacroUtils {}
