use bevy::prelude::{Color, Plugin};
pub mod camera;
pub struct MapManifest {
    pub map_files: Vec<String>,
}
pub struct MacroUtils {}

pub enum ColorPalette {
    ForceRed,
    ForceBlue,
    ForceYellow,
    ForceOrange,
    ForceGreen,
    ForceBlack,
    ForceWhite,
}

impl ColorPalette {
    fn value(&self) -> Color {
        match self {
            ColorPalette::ForceRed => Self::rgb(227, 72, 71),
            ColorPalette::ForceBlue => Self::rgb(150, 143, 239),
            ColorPalette::ForceYellow => Self::rgb(239, 255, 100),
            ColorPalette::ForceOrange => Self::rgb(225, 164, 3),
            ColorPalette::ForceGreen => Self::rgb(10, 155, 112),
            ColorPalette::ForceWhite => Color::WHITE,
            ColorPalette::ForceBlack => Color::BLACK,
        }
    }

    fn rgb(r: u16, g: u16, b: u16) -> Color {
        Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}

impl From<ColorPalette> for Color {
    fn from(color: ColorPalette) -> Self {
        color.value()
    }
}

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
