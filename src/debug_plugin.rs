use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::birdoids::{
    Boid, center_of_boids, physics::Force, physics::Velocity, velocity_of_boids,
};

const SCANRADIUS: f32 = 50.0;
pub struct BoidsDebugPlugin;

impl Plugin for BoidsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, (update_cursor, update_scanner_mode, do_scan));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ScannerWidget::default(),
        Mesh2d(meshes.add(Annulus::new(SCANRADIUS - 1.0, SCANRADIUS))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 0.0))),
    ));
}

#[derive(Bundle)]
struct ScannerWidget {
    cursor: Cursor,
    select: SelectionMode,
    scan: ScannerMode,
}

impl Default for ScannerWidget {
    fn default() -> Self {
        Self {
            cursor: Cursor,
            select: SelectionMode::CircularArea,
            scan: ScannerMode::CenterOfMass,
        }
    }
}

#[derive(Component)]
struct Cursor;

#[derive(Component, Debug)]
enum SelectionMode {
    NearestSingle,
    CircularArea,
}

#[derive(Component, Debug)]
enum ScannerMode {
    CenterOfMass,
    Velocity,
}

fn update_cursor(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut cursor_query: Query<&mut Transform, With<Cursor>>,
) {
    // I'm trusting that only one thing has the `Cursor` component
    // It's defined here in this module, so that *should* be the case...
    let win = window.single().unwrap();
    let (cam, cam_transform) = camera.single().unwrap();
    // the cursor might not be on the window. Only adjust position when it is.
    if let Some(cursor_pos_in_window) = win.cursor_position() {
        // transform the window position into world space
        // I'm trusting that this viewport actually displays this camera
        // (or at least that the coordinates can map correctly)
        let cursor_in_world = cam
            .viewport_to_world_2d(cam_transform, cursor_pos_in_window)
            .unwrap();
        let mut cursor = cursor_query.single_mut().unwrap();
        cursor.translation = cursor_in_world.extend(0.0);
    }
}

// System to monitor for mouse and keyboard input and then update
// the scanner's modes accordingly.
fn update_scanner_mode(
    keycodes: Res<ButtonInput<KeyCode>>,
    mousebuttons: Res<ButtonInput<MouseButton>>,
    mut scanner_query: Query<(&mut SelectionMode, &mut ScannerMode), With<Cursor>>,
) {
    // I'm making another assertion that there is exactly one scanner.
    let (mut select_mode, mut scan_mode) = scanner_query.single_mut().unwrap();

    // Assign selection mode
    if keycodes.just_pressed(KeyCode::Digit1) {
        *select_mode = SelectionMode::NearestSingle;
    } else if keycodes.just_pressed(KeyCode::Digit2) {
        *select_mode = SelectionMode::CircularArea;
    }

    // Select property scanning mode
    if mousebuttons.just_pressed(MouseButton::Left) {
        *scan_mode = ScannerMode::CenterOfMass;
    } else if mousebuttons.just_pressed(MouseButton::Right) {
        *scan_mode = ScannerMode::Velocity;
    }
}

fn do_scan(
    boids_query: Query<(&Transform, &Velocity, &Force), With<Boid>>,
    scanner_query: Query<(&Transform, &SelectionMode, &ScannerMode), With<Cursor>>,
    spatial: SpatialQuery,
    /* Push info to summary somewhere */
    mut gizmos: Gizmos,
) {
    let (cursor_pos, select_mode, scan_mode) = scanner_query.single().unwrap();
    match select_mode {
        SelectionMode::NearestSingle => todo!(),
        SelectionMode::CircularArea => {
            let boids = spatial.shape_hits(
                &Collider::circle(SCANRADIUS),
                cursor_pos.translation.xy(),
                0.0,
                Dir2::Y,
                64,
                &ShapeCastConfig::from_max_distance(100.0),
                &SpatialQueryFilter::default()
            ).into_iter()
                .map(|shape_hit| {
                    let entt = shape_hit.entity;
                    let (tsfm, vel, _) = boids_query
                        .get(entt)
                        .expect("Debugger scanned a Boid missing one of it's components!");
                    (tsfm, vel)
                });
            
            match scan_mode {
                ScannerMode::CenterOfMass => {
                    if let Some(center_mass) = center_of_boids(
                        // `center_of_boids` needs an Iterator<Item = Vec2>, so we map over
                        // the output tuple to make one.
                        boids.map(|item| item.0.translation.xy()),
                    ) {
                        gizmos.circle_2d(center_mass, 1.0, bevy::color::palettes::css::RED);
                    }
                }
                ScannerMode::Velocity => {
                    if let Some(avg_velocity) = velocity_of_boids(boids.map(|item| {
                        (*item.1).xy() * 1.0
                    })) {
                        // cursor_pos.translation is already in world space, so I can skip the window -> world transform like in update_cursor()
                        gizmos.line_2d(
                            cursor_pos.translation.xy(),
                            cursor_pos.translation.xy() + avg_velocity,
                            bevy::color::palettes::css::RED,
                        );
                    }
                }
            }
        }
    }
}
