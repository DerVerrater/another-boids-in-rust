use bevy::prelude::*;

mod breakout_plugin;
mod hello_world_plugin;

use breakout_plugin::BreakoutPlugin;
use hello_world_plugin::HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BreakoutPlugin)
        .run();
}
