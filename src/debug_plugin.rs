use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

pub struct BoidsDebugPlugin;

impl Plugin for BoidsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, update_cursor);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Cursor,
        MaterialMesh2dBundle {
            mesh: meshes.add(Annulus::new(9.5, 10.0)).into(),
            material: materials.add(Color::srgb(0.0, 0.0, 0.0)),
            ..default()
        },
    ));
}

#[derive(Component)]
struct Cursor;

fn update_cursor(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
) {
    // I'm trusting that only one thing has the `Cursor` component
    // It's defined here in this module, so that *should* be the case...
    let win = window.get_single().unwrap();
    let (cam, cam_transform) = camera.get_single().unwrap();
    // the cursor might not be on the window. Only adjust position when it is.
    if let Some(cursor_pos_in_window) = win.cursor_position() {
        // transform the window position into world space
        // I'm trusting that this viewport actually displays this camera
        // (or at least that the coordinates can map correctly)
        let cursor_in_world = cam
            .viewport_to_world_2d(cam_transform, cursor_pos_in_window)
            .unwrap();
        let mut cursor = cursor_query.get_single_mut().unwrap();
        cursor.translation = cursor_in_world.extend(0.0);
    }
}
