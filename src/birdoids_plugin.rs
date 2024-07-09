use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_spatial::{
    AutomaticUpdate,
    SpatialAccess,
    TransformMode,
    kdtree::KDTree2,
};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const PLAYERBOID_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TURN_FACTOR: f32 = 1.;
const BOID_VIEW_RANGE: f32 = 50.0;
pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AutomaticUpdate::<TrackedByKdTree>::new()
                .with_frequency(Duration::from_secs_f32(0.3))
                .with_transform(TransformMode::GlobalTransform))
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (spawn_camera, spawn_boids))
            .add_systems(FixedUpdate, (
                apply_velocity,
                turn_if_edge,
                check_keyboard,
                cohesion,
            ));
    }
}

#[derive(Component)]
struct Boid;

// It's a Boid, but with an extra component so the player
// can control it from the keyboard
#[derive(Component)]
struct PlayerBoid;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

#[derive(Component)]
struct TrackedByKdTree;

#[derive(Bundle)]
struct BoidBundle {
    boid: Boid,
    velocity: Velocity,
    spatial: TrackedByKdTree,
}

impl BoidBundle {
    fn new(vel: Vec3) -> Self {
        Self {
            boid: Boid,
            velocity: Velocity(vel),
            spatial: TrackedByKdTree,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let num_boids = 1000;
    for i in 0..num_boids {
        let frac = 2.0 * std::f32::consts::PI / (num_boids as f32) * (i as f32);
        let vel = Vec3::new(frac.cos() * 10.0, frac.sin() * 10.0, 0.0);
        commands.spawn((
            BoidBundle::new(vel),
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
                ..default()
            },
        ));
    }

    commands.spawn((
        BoidBundle::new(Vec3::new(0.0, 0.0, 0.0)),
        PlayerBoid,
        MaterialMesh2dBundle {
            mesh: meshes.add(Triangle2d::default()).into(),
            material: materials.add(PLAYERBOID_COLOR),
            ..default()
        },
    ));
}

fn turn_if_edge(
    mut query: Query<(&mut Transform, &mut Velocity), With<Boid>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let (width, height) = (window.resolution.width(), window.resolution.height());
        for (transform, mut velocity) in &mut query {
            let boid_pos = transform.translation.xy();
            if boid_pos.x <= -width / 2. + 50. {
                velocity.x += TURN_FACTOR;
            } else if boid_pos.x >= width / 2. - 50. {
                velocity.x -= TURN_FACTOR;
            }

            if boid_pos.y <= -height / 2. + 50. {
                velocity.y += TURN_FACTOR;
            } else if boid_pos.y >= height / 2. - 50. {
                velocity.y -= TURN_FACTOR;
            }
        }
    } else {
        panic!("System turn_if_edge(...) got an Err(_) when getting the window properties");
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        let delta_position = **velocity * time.delta_seconds();
        transform.translation += delta_position;
    }
}

fn check_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut query: Query<&mut Velocity, With<PlayerBoid>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit::Success);
    }

    let mut pvelocity = query.single_mut();
    let mut dir = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }

    **pvelocity = **pvelocity + dir.extend(0.0);
}

fn cohesion(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut query: Query<&mut Boid, With<TrackedByKdTree>>,
    player: Query<&Transform, With<PlayerBoid>>,
) {
    let player_transform = player.get_single().unwrap();

    let neighbors = spatial_tree.within_distance(
        player_transform.translation.xy(),
        BOID_VIEW_RANGE
    );
}
