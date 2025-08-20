use bevy::prelude::*;

mod birdoids;
mod debug_plugin;

use birdoids::BoidsPlugin;
use debug_plugin::BoidsDebugPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsDebugPlugin)
        .add_plugins(BoidsPlugin)
        .run();
}
