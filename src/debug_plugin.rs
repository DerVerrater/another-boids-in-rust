use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

pub struct BoidsDebugPlugin;

impl Plugin for BoidsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(FixedUpdate, (
                update_cursor,
                update_scanner_mode,
                print_gizmo_config,
            ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        ScannerWidget::default(),
        MaterialMesh2dBundle {
            mesh: meshes.add(Annulus::new(9.5, 10.0)).into(),
            material: materials.add(Color::srgb(0.0, 0.0, 0.0)),
            ..default()
        },
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

// System to monitor for mouse and keyboard input and then update
// the scanner's modes accordingly.
fn update_scanner_mode(
    keycodes: Res<ButtonInput<KeyCode>>,
    mousebuttons: Res<ButtonInput<MouseButton>>,
    mut scanner_query: Query<(&mut SelectionMode, &mut ScannerMode), With<Cursor>>, 
) {
    // I'm making another assertion that there is exactly one scanner.
    let (mut select_mode, mut scan_mode) = scanner_query.get_single_mut().unwrap();
    
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

fn print_gizmo_config(
    query: Query<(&SelectionMode, &ScannerMode), With<Cursor>>,
) {
    let (select, scan) = query.get_single().unwrap();
    println!("Selection: {select:?}, Scanning: {scan:?}");
}