
use bevy::prelude::*;

mod hello_world_plugin;
mod breakout_plugin;

use hello_world_plugin::HelloPlugin;
use breakout_plugin::BreakoutPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BreakoutPlugin)
        .run();
}
