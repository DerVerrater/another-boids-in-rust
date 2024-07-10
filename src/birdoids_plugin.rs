use std::time::Duration;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_spatial::{
    kdtree::KDTree2, AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode
};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const PLAYERBOID_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TURN_FACTOR: f32 = 1.0;
const BOID_VIEW_RANGE: f32 = 50.0;
const COHESION_FACTOR: f32 = 1.0;
const SEPARATION_FACTOR: f32 = 1.0;
const ALIGNMENT_FACTOR: f32 = 1.0;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AutomaticUpdate::<TrackedByKdTree>::new()
                .with_frequency(Duration::from_secs_f32(0.3))
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(SpatialStructure::KDTree2))
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (spawn_camera, spawn_boids))
            .add_systems(FixedUpdate, (
                apply_velocity,
                turn_if_edge,
                check_keyboard,
                // cohesion,
                separation,
                // alignment,
                space_brakes,
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

#[derive(Component, Deref, DerefMut)]
struct Acceleration(Vec3);

#[derive(Component)]
struct TrackedByKdTree;

#[derive(Bundle)]
struct BoidBundle {
    boid: Boid,
    velocity: Velocity,
    accel: Acceleration,
    spatial: TrackedByKdTree,
}

impl BoidBundle {
    fn new(vel: Vec3) -> Self {
        Self {
            boid: Boid,
            velocity: Velocity(vel),
            accel: Acceleration(Vec3::ZERO),
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
    let num_boids = 50;
    for i in 0..num_boids {
        let frac = 2.0 * std::f32::consts::PI / (num_boids as f32) * (i as f32);
        let vel = Vec3::new(frac.cos() * 1.0, frac.sin() * 1.0, 0.0);
        commands.spawn((
            BoidBundle::new(vel),
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
                transform: Transform {
                    translation: vel * 20.0,
                    ..default()
                },
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

fn space_brakes(mut mobs: Query<&mut Acceleration, With<Boid>>) {
    for mut accel in &mut mobs {
        let braking_dir = -accel.0 * 0.01;
        accel.0 += braking_dir;
    }
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

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity, &Acceleration)>,
    time: Res<Time>
) {
    for (mut transform, velocity, acceleration) in &mut query {
        let delta_v = **acceleration * time.delta_seconds();
        let delta_position = (**velocity + delta_v) * time.delta_seconds();
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
    mut boids: Query<(&Transform, &mut Acceleration), With<Boid>>,
) {
    // for each boid
    // find neighbors
    // find center-of-mass of neighbors
    // find vector from boid to flock CoM
    // apply force
    for (transform, mut acceleration) in &mut boids {
        let neighbors = spatial_tree.within_distance(
            transform.translation.xy(),
            BOID_VIEW_RANGE
        );
        if neighbors.len() > 0 {
            let center_of_mass = neighbors.iter()
                .map(|(pos, _)| pos.extend(0.0) )
                .fold(transform.translation, |acc, neighbor| {
                    acc + neighbor
                }) / (neighbors.len()) as f32;
            
            let towards = (center_of_mass - transform.translation).normalize();
            acceleration.0 += towards * COHESION_FACTOR;
        }
    }
}

fn separation(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut boids: Query<(&Transform, &mut Acceleration), With<Boid>>,
) {
    // for each boid
    // find neighbors
    // sum force from neighbors
    // apply force
    for (boid_transform, mut boid_acceleration) in &mut boids {
        let neighbors = spatial_tree.within_distance(
            boid_transform.translation.xy(),
            BOID_VIEW_RANGE / 4.0,
        );
        let accel = neighbors.iter()
            .map(|(pos, _)| pos.extend(0.0))
            .fold(Vec3::ZERO, |accumulator, neighbor |{
                let force = separation_force(boid_transform.translation.xy(), neighbor.xy());
                accumulator + *force
            });
        boid_acceleration.0 += accel;
    }
}

// TODO: Make this an exponential so force gets stronger faster as the points approach.
fn separation_force(us: Vec2, neighbor: Vec2) -> Acceleration {
    let distance = neighbor - us;
    Acceleration(-(distance * SEPARATION_FACTOR).extend(0.0))
}

fn alignment(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut boids: Query<(&Transform, &Velocity, &mut Acceleration), With<Boid>>,
    boid_velocities: Query<&Velocity, With<Boid>>,
) {
    // for each boid
    // find neighbors
    // find average velocity vector of neighbors
    // calculate steering force
    //      perpendicular so that magnitude is constant
    // apply steering force

    for (transform, velocity, mut acceleration) in &mut boids {
        let neighbors = spatial_tree.within_distance(
            transform.translation.xy(),
            BOID_VIEW_RANGE,
        );
        // averaging divides by length. Guard against an empty set of neighbors
        // so that we don't divide by zero.
        if neighbors.len() > 0 {
            let velocity_sum = neighbors.iter()
                .map(|(_pos, entity_id)| {
                    if let Some(boid_id) = entity_id {
                        if let Ok(boid) = boid_velocities.get(*boid_id) {
                            boid.0
                        } else {
                            panic!("Err: Got a Boid during neighbor collection, but it has no Entity ID!");
                        }
                    } else {
                        panic!("Err: Found entity with component TrackedByKdTree that has no Entity ID!");
                    }
            }).sum::<Vec3>();
            let velocity_average = velocity_sum / (neighbors.len() as f32);
            let deviation = -velocity.0 + velocity_average;
            acceleration.0 += deviation * ALIGNMENT_FACTOR;
        }
    }
}
