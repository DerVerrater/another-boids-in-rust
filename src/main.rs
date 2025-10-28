use bevy::prelude::*;

mod birdoids;
mod debug_plugin;

use birdoids::BoidsPlugin;
use debug_plugin::BoidsDebugPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#boids-canvas".to_owned()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoidsDebugPlugin)
        .add_plugins(BoidsPlugin)
        .run();
}
