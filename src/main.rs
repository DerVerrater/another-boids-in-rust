use bevy::prelude::*;

mod birdoids;
mod debug_plugin;

use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{ResourceInspectorPlugin},
};
use birdoids::BoidsPlugin;
use debug_plugin::BoidsDebugPlugin;

use crate::birdoids::FlockingParameters;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#boids-canvas".to_owned()),
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BoidsDebugPlugin)
        .add_plugins(BoidsPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<FlockingParameters>::new()) // TODO: monitor only the flocking params resource (once it exists)
        .run();
}
