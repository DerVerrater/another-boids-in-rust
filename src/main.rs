
use bevy::prelude::*;

mod hello_world_plugin;
// mod breakout_plugin;

use hello_world_plugin::HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}
