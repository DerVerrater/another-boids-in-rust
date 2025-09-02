pub mod physics;

use bevy::prelude::*;
use bevy_spatial::{
    kdtree::KDTree2, AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode,
};

use crate::birdoids::physics::{apply_velocity, Force, Velocity};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
const PLAYERBOID_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const TURN_FACTOR: f32 = 1.0;
const BOID_VIEW_RANGE: f32 = 50.0;
const COHESION_FACTOR: f32 = 1.0;
const SEPARATION_FACTOR: f32 = 1.0;
const ALIGNMENT_FACTOR: f32 = 10.0;
const SPACEBRAKES_COEFFICIENT: f32 = 0.01;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AutomaticUpdate::<TrackedByKdTree>::new()
                // .with_frequency(Duration::from_secs_f32(0.3))
                .with_transform(TransformMode::GlobalTransform)
                .with_spatial_ds(SpatialStructure::KDTree2),
        )
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, (spawn_camera, spawn_boids))
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                turn_if_edge,
                check_keyboard,
                cohesion,
                separation,
                alignment,
                // space_brakes,
            ),
        );
    }
}

#[derive(Component)]
pub(crate) struct Boid;

// It's a Boid, but with an extra component so the player
// can control it from the keyboard
#[derive(Component)]
struct PlayerBoid;

#[derive(Component)]
pub struct TrackedByKdTree;

#[derive(Bundle)]
struct BoidBundle {
    boid: Boid,
    velocity: Velocity,
    accel: Force,
    spatial: TrackedByKdTree,
}

impl BoidBundle {
    fn new(vel: Vec3) -> Self {
        Self {
            boid: Boid,
            velocity: Velocity(vel),
            accel: Force(Vec3::ZERO),
            spatial: TrackedByKdTree,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let num_boids = 1000;
    for i in 0..num_boids {
        let frac = 2.0 * std::f32::consts::PI / (num_boids as f32) * (i as f32);
        let vel = Vec3::new(frac.cos() * 1.0, frac.sin() * 1.0, 0.0) * 10.0;
        commands.spawn((
            BoidBundle::new(vel),
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
            Transform::from_translation(vel * 20.0),
        ));
    }

    commands.spawn((
        BoidBundle::new(Vec3::new(0.0, 0.0, 0.0)),
        PlayerBoid,
        Mesh2d(meshes.add(Triangle2d::default())),
        MeshMaterial2d(materials.add(PLAYERBOID_COLOR)),
    ));
}

fn space_brakes(mut mobs: Query<&mut Force, With<Boid>>) {
    for mut accel in &mut mobs {
        let braking_dir = -accel.0 * SPACEBRAKES_COEFFICIENT;
        accel.0 += braking_dir;
    }
}

fn turn_if_edge(
    mut query: Query<(&mut Transform, &mut Velocity), With<Boid>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.single() {
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

fn check_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut query: Query<&mut Velocity, With<PlayerBoid>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit::Success);
    }

    let mut pvelocity = query
        .single_mut()
        .expect("[birdoids_plugin::check_keyboard()] ->> There seems to be more than one player... How did that happen?");
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

    **pvelocity += dir.extend(0.0);
}

fn cohesion(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut boids: Query<(&Transform, &mut Force), With<Boid>>,
) {
    // for each boid
    // find neighbors
    // find center-of-mass of neighbors
    // find vector from boid to flock CoM
    // apply force
    for (transform, mut acceleration) in &mut boids {
        let neighbors = spatial_tree.within_distance(transform.translation.xy(), BOID_VIEW_RANGE);
        if let Some(center_mass) = center_of_boids(neighbors.iter().map(|boid| boid.0)) {
            let force = cohesive_force(center_mass, transform.translation.xy()).unwrap_or_default();
            acceleration.0 += *force * COHESION_FACTOR;
        }
    }
}

fn separation(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut boids: Query<(&Transform, &mut Force), With<Boid>>,
) {
    // for each boid
    // find neighbors
    // sum force from neighbors
    // apply force
    for (boid_transform, mut boid_acceleration) in &mut boids {
        let neighbors =
            spatial_tree.within_distance(boid_transform.translation.xy(), BOID_VIEW_RANGE / 4.0);
        let accel = neighbors.iter().map(|(pos, _)| pos.extend(0.0)).fold(
            Vec3::ZERO,
            |accumulator, neighbor| {
                let force = separation_force(boid_transform.translation.xy(), neighbor.xy())
                    .unwrap_or_default();
                accumulator + *force * SEPARATION_FACTOR
            },
        );
        boid_acceleration.0 += accel;
    }
}

fn alignment(
    spatial_tree: Res<KDTree2<TrackedByKdTree>>,
    mut boids: Query<(&Transform, &mut Force), With<Boid>>,
    boid_velocities: Query<&Velocity, With<Boid>>,
) {
    // for each boid
    // find neighbors
    // find average velocity vector of neighbors
    // calculate steering force
    //      perpendicular so that magnitude is constant
    // apply steering force

    for (transform, mut force) in &mut boids {
        let neighbors = spatial_tree.within_distance(transform.translation.xy(), BOID_VIEW_RANGE);
        // averaging divides by length. Guard against an empty set of neighbors
        let (len, sum) = neighbors
            .iter()
            // Extract the velocities by `get()`ing from another query param.
            .map(|(_pos, maybe_entt)| {
                let entt = maybe_entt
                    .expect("Neighbor boid has no Entity ID. I don't know what this means");
                let vel = boid_velocities.get(entt).expect("Boid has no velocity!");
                vel.xy()
            })
            .enumerate()
            .fold((0, Vec2::ZERO), |(_len, vel_acc), (idx, vel)| {
                (idx, vel_acc + vel)
            });

        // Skip to next boid if the current one has no neighbors.
        let avg = if len > 0 {
            sum / (len as f32)
        } else {
            continue;
        };

        force.0 = (avg.extend(0.0) - force.0) * ALIGNMENT_FACTOR;
    }
}

pub(crate) fn center_of_boids(points: impl Iterator<Item = Vec2>) -> Option<Vec2> {
    average_of_vec2s(points)
}

pub(crate) fn velocity_of_boids(points: impl Iterator<Item = Vec2>) -> Option<Vec2> {
    average_of_vec2s(points)
}

fn average_of_vec2s(points: impl Iterator<Item = Vec2>) -> Option<Vec2> {
    let (len, sum) = points
        .enumerate()
        .fold((0, Vec2::ZERO), |(_len, sum), (idx, point)| {
            // replace length with most recent index
            // add running sum & new point for new running sum
            (idx, sum + point)
        });
    let avg = sum / ((len + 1) as f32);

    Some(avg)
}

// f(x) = 4((x-0.5)^3 + 0.125)
fn cohesive_force(boid: Vec2, target: Vec2) -> Option<Force> {
    let deviation = target - boid;
    /*
    Scale deviation vector by the boid's view range. The curve is made to
    operate on the range (0, 1), so that needs to be the viewing circle.
    */
    let scaled = deviation / BOID_VIEW_RANGE;
    let mag = scaled.length();
    if mag > 0.0 {
        let cube: f32 = (mag - 0.5).powf(3.0);
        let offset = cube + 0.125;
        let mul = offset * 4.0;
        // It's necessary to re-normalize the scaled vector here.
        // This is because it needs to be a unit vector before getting a new
        // magnitude assigned.
        let force_vec = mul * scaled.normalize();
        Some(Force(force_vec.extend(0.0)))
    } else {
        None
    }
}

// f(x) = x^2 - 1
fn separation_force(boid: Vec2, target: Vec2) -> Option<Force> {
    // Scale from BOID_VIEW_RANGE to unit space
    let distance_unit = (target - boid) / BOID_VIEW_RANGE;
    let mag = distance_unit.length();
    if mag > 0.0 {
        let force_mag = mag.powf(2.0) - 1.0;
        let force = force_mag * distance_unit.normalize();
        Some(Force(force.extend(0.0)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::birdoids::{cohesive_force, separation_force};

    use super::{physics::Force, BOID_VIEW_RANGE};

    // forces are relative to the boid's view range, so all
    // distances need to be fractions of that

    #[test]
    fn check_cohesion_zero_zero() {
        let force = cohesive_force(Vec2::ZERO, Vec2::ZERO);
        assert!(force.is_none());
    }

    // *********************
    // Cohesion x-axis tests
    // *********************

    #[test]
    fn check_cohesion_midpoint_x_positive() {
        // Pull right 0.5 units
        assert_eq!(
            Some(Force(Vec3::new(0.5, 0.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(0.5 * BOID_VIEW_RANGE, 0.0),)
        );
    }

    #[test]
    fn check_cohesion_midpoint_x_negative() {
        // Pull left 0.5 units
        assert_eq!(
            Some(Force(Vec3::new(-0.5, 0.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(-0.5 * BOID_VIEW_RANGE, 0.0),)
        );
    }

    #[test]
    fn check_cohesion_edge_x_positive() {
        // pull left 1.0 units
        assert_eq!(
            Some(Force(Vec3::new(1.0, 0.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(1.0 * BOID_VIEW_RANGE, 0.0),)
        );
    }

    #[test]
    fn check_cohesion_edge_x_negative() {
        // pull left 1.0 units
        assert_eq!(
            Some(Force(Vec3::new(-1.0, 0.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(-1.0 * BOID_VIEW_RANGE, 0.0),)
        );
    }

    // *********************
    // Cohesion y-axis tests
    // *********************

    #[test]
    fn check_cohesion_midpoint_y_positive() {
        // Pull up 0.5 units
        assert_eq!(
            Some(Force(Vec3::new(0.0, 0.5, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.5 * BOID_VIEW_RANGE),)
        );
    }

    #[test]
    fn check_cohesion_midpoint_y_negative() {
        // Pull down 0.5 units
        assert_eq!(
            Some(Force(Vec3::new(0.0, -0.5, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(0.0, -0.5 * BOID_VIEW_RANGE),)
        );
    }

    #[test]
    fn check_cohesion_edge_y_positive() {
        // Pull up 1.0 units
        assert_eq!(
            Some(Force(Vec3::new(0.0, 1.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0 * BOID_VIEW_RANGE))
        );
    }

    #[test]
    fn check_cohesion_edge_y_negative() {
        // pull down 0.2 units
        assert_eq!(
            Some(Force(Vec3::new(0.0, -1.0, 0.0))),
            cohesive_force(Vec2::new(0.0, 0.0), Vec2::new(0.0, -1.0 * BOID_VIEW_RANGE),)
        );
    }

    // Separation 0,0 test
    #[test]
    fn check_separation_zero_zero() {
        let force = separation_force(Vec2::ZERO, Vec2::ZERO);
        assert!(force.is_none());
    }

    // *********************
    // Separation x-axis tests
    // *********************
    #[test]
    fn check_separation_midpoint_x_positive() {
        assert_eq!(
            Some(Force(Vec3::new(0.75, 0.0, 0.0))), // expected force
            separation_force(
                Vec2::new(0.5 * BOID_VIEW_RANGE, 0.0), // boid position
                Vec2::ZERO,                            // obstacle position
            )
        );
    }

    #[test]
    fn check_separation_midpoint_x_negative() {
        assert_eq!(
            Some(Force(Vec3::new(-0.75, 0.0, 0.0))), // expected force
            separation_force(
                Vec2::new(-0.5 * BOID_VIEW_RANGE, 0.0), // boid position
                Vec2::ZERO,                             // obstacle position
            )
        );
    }

    #[test]
    fn check_separation_edge_x_positive() {
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(Vec2::new(1.0 * BOID_VIEW_RANGE, 0.0), Vec2::ZERO,),
        );
    }

    #[test]
    fn check_separation_edge_x_negative() {
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(Vec2::new(-1.0 * BOID_VIEW_RANGE, 0.0), Vec2::ZERO,),
        );
    }

    // *********************
    // Separation y-axis tests
    // *********************
    #[test]
    fn check_separation_midpoint_y_positive() {
        assert_eq!(
            Some(Force(Vec3::new(0.0, 0.75, 0.0))),
            separation_force(Vec2::new(0.0, 0.5 * BOID_VIEW_RANGE), Vec2::ZERO,)
        );
    }

    #[test]
    fn check_separation_midpoint_y_negative() {
        assert_eq!(
            Some(Force(Vec3::new(0.0, -0.75, 0.0))),
            separation_force(Vec2::new(0.0, -0.5 * BOID_VIEW_RANGE), Vec2::ZERO,)
        );
    }

    #[test]
    fn check_separation_edge_y_positive() {
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(Vec2::new(0.0, 1.0 * BOID_VIEW_RANGE), Vec2::ZERO,)
        )
    }

    #[test]
    fn check_separation_edge_y_negative() {
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(Vec2::new(0.0, -1.0 * BOID_VIEW_RANGE), Vec2::ZERO,)
        )
    }
}
