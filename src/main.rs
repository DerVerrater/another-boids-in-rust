use bevy::prelude::*;

mod birdoids_plugin;
mod debug_plugin;

use birdoids_plugin::BoidsPlugin;
use debug_plugin::BoidsDebugPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsDebugPlugin)
        .add_plugins(BoidsPlugin)
        .run();
}
