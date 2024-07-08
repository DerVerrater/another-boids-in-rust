use bevy::prelude::*;

mod breakout_plugin;
mod hello_world_plugin;
mod birdoids_plugin;

use breakout_plugin::BreakoutPlugin;
use hello_world_plugin::HelloPlugin;
use birdoids_plugin::BoidsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        .run();
}
