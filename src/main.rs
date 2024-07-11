use bevy::prelude::*;

mod birdoids_plugin;

use birdoids_plugin::BoidsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        .run();
}
