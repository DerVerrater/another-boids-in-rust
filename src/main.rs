use bevy::prelude::*;

mod birdoids_plugin;
mod breakout_plugin;
mod hello_world_plugin;

use birdoids_plugin::BoidsPlugin;
use breakout_plugin::BreakoutPlugin;
use hello_world_plugin::HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoidsPlugin)
        .run();
}
