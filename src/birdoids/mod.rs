pub mod physics;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::birdoids::physics::{Force, Velocity, apply_velocity};
use bevy_inspector_egui::{InspectorOptions, prelude::ReflectInspectorOptions};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .insert_resource(FlockingParameters::new())
            .register_type::<FlockingParameters>()
            .insert_resource(MiscParams::new())
            .register_type::<MiscParams>()
            .add_systems(Startup, (spawn_camera, spawn_boids))
            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    turn_if_edge,
                    cohesion,
                    separation,
                    alignment,
                    speed_controller,
                ),
            );
    }
}

#[derive(InspectorOptions, Reflect, Resource, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct FlockingParameters {
    view_range: f32,
    cohesion: f32,
    separation: f32,
    alignment: f32,
}

impl FlockingParameters {
    pub(crate) fn new() -> Self {
        Self {
            view_range: 15.0,
            cohesion: 1.0,
            separation: 5.0,
            alignment: 2.0,
        }
    }
}

#[derive(InspectorOptions, Reflect, Resource, Debug, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct MiscParams {
    turn_factor: f32,
    spacebrakes: f32,
    minimum_speed: f32,
    maximum_speed: f32,
}

impl MiscParams {
    pub(crate) fn new() -> Self {
        Self {
            turn_factor: 1.0,
            spacebrakes: 0.5,
            minimum_speed: 50.0,
            maximum_speed: 200.0,
        }
    }
}

#[derive(Component)]
#[require(Velocity, Force)]
pub(crate) struct Boid;

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
            Boid,
            Velocity(vel),
            Mesh2d(meshes.add(Circle::new(1.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
            Transform::from_translation(vel * 20.0),
            // RigidBody::Dynamic,
            Collider::circle(1.0),
        ));
    }
}

/// Controls the boid's minimum and maximum speed according to a low- and
/// high-threshold. Boids moving too slow are sped up, and boids moving too
/// fast are slowed down.
fn speed_controller(mut mobs: Query<(&Velocity, &mut Force), With<Boid>>, params: Res<MiscParams>) {
    for (vel, mut impulse) in &mut mobs {
        if vel.0.length() < params.minimum_speed {
            impulse.0 += vel.0 * params.spacebrakes;
        } else if vel.0.length() > params.maximum_speed {
            impulse.0 += -vel.0 * params.spacebrakes;
        }
    }
}

fn turn_if_edge(
    mut query: Query<(&mut Transform, &mut Velocity), With<Boid>>,
    window: Query<&Window>,
    params: Res<MiscParams>,
) {
    if let Ok(window) = window.single() {
        let (width, height) = (window.resolution.width(), window.resolution.height());
        for (transform, mut velocity) in &mut query {
            let boid_pos = transform.translation.xy();
            if boid_pos.x <= -width / 2. + 50. {
                velocity.x += params.turn_factor;
            } else if boid_pos.x >= width / 2. - 50. {
                velocity.x -= params.turn_factor;
            }

            if boid_pos.y <= -height / 2. + 50. {
                velocity.y += params.turn_factor;
            } else if boid_pos.y >= height / 2. - 50. {
                velocity.y -= params.turn_factor;
            }
        }
    } else {
        panic!("System turn_if_edge(...) got an Err(_) when getting the window properties");
    }
}

fn cohesion(
    spatial: SpatialQuery,
    // TODO: Ensure this is logically sound. I think it will fail the "disjoint queries" requirement.
    boid_locations: Query<&Transform, With<Boid>>,
    mut boids: Query<(Entity, &Transform, &mut Force), With<Boid>>,
    props: Res<FlockingParameters>,
) {
    // for each boid
    // find neighbors
    // find center-of-mass of neighbors
    // find vector from boid to flock CoM
    // apply force
    for (this_entt, transform, mut force) in &mut boids {
        let (len, sum) = spatial
            .shape_intersections(
                &Collider::circle(props.view_range),
                transform.translation.xy(),
                0.0,
                &SpatialQueryFilter::default(),
            )
            .iter()
            .filter_map(|&entt| {
                // extract neighbor's position
                // Skip self-comparison. A boid should not try to separate from itself.
                if this_entt == entt {
                    None
                } else {
                    let tsfm = boid_locations.get(entt).unwrap();
                    Some(tsfm.translation.xy())
                }
            })
            .enumerate()
            .fold((0, Vec2::ZERO), |(_len, com), (idx, pos)| (idx, com + pos));

        // Skip to next boid if the current one has no neighbors.
        let center_of_mass = if len > 0 {
            sum / ((len + 1) as f32)
        } else {
            continue;
        };

        let impulse = cohesive_force(center_of_mass, transform.translation.xy(), props.view_range)
            .expect("damn");

        force.0 -= impulse.0 * props.cohesion;
    }
}

fn separation(
    spatial: SpatialQuery,
    boid_locations: Query<&Transform, With<Boid>>,
    mut boids: Query<(Entity, &Transform, &mut Force), With<Boid>>,
    props: Res<FlockingParameters>,
) {
    // for each boid
    // find neighbors
    // sum force from neighbors
    // apply force
    for (this_entt, tsfm, mut force) in &mut boids {
        let impulse = spatial
            .shape_intersections(
                &Collider::circle(props.view_range / 4.0),
                tsfm.translation.xy(),
                0.0,
                &SpatialQueryFilter::default(),
            )
            .iter()
            .filter_map(|&entt| {
                // Skip self-comparison. A boid should not try to separate from itself.
                if this_entt == entt {
                    None
                } else {
                    let pos = boid_locations.get(entt).unwrap().translation.xy();
                    Some(pos.extend(0.0))
                }
            })
            .fold(Vec3::ZERO, |acc, other| {
                // let force = tsfm.translation - other;
                let force = separation_force(tsfm.translation.xy(), other.xy(), props.view_range)
                    .expect("angy");
                acc + force.0
            });
        force.0 += impulse * props.separation;
    }
}

fn alignment(
    spatial: SpatialQuery,
    mut boids: Query<(Entity, &Transform, &mut Force), With<Boid>>,
    boid_velocities: Query<&Velocity, With<Boid>>,
    props: Res<FlockingParameters>,
) {
    // for each boid
    // find neighbors
    // find average velocity vector of neighbors
    // calculate steering force
    //      perpendicular so that magnitude is constant
    // apply steering force

    for (this_entt, transform, mut force) in &mut boids {
        let neighbors = spatial.shape_intersections(
            &Collider::circle(props.view_range),
            transform.translation.xy(),
            0.0,
            &SpatialQueryFilter::default(),
        );
        // averaging divides by length. Guard against an empty set of neighbors
        let (len, sum) = neighbors
            .iter()
            .filter_map(|&entt| {
                if this_entt == entt {
                    None
                } else {
                    let vel = boid_velocities.get(entt).expect("Boid has no velocity!");
                    Some(vel.xy())
                }
            })
            .enumerate()
            .fold((0, Vec2::ZERO), |(_len, vel_acc), (idx, vel)| {
                (idx, vel_acc + vel)
            });

        // Skip to next boid if the current one has no neighbors.
        let avg = if len > 0 {
            sum / ((len + 1) as f32)
        } else {
            continue;
        };

        let boid_vel = boid_velocities.get(this_entt).unwrap();
        force.0 += (avg.extend(0.0) - boid_vel.0) * props.alignment;
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
fn cohesive_force(boid: Vec2, target: Vec2, view_range: f32) -> Option<Force> {
    let deviation = target - boid;
    /*
    Scale deviation vector by the boid's view range. The curve is made to
    operate on the range (0, 1), so that needs to be the viewing circle.
    */
    let scaled = deviation / view_range;
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
fn separation_force(boid: Vec2, target: Vec2, view_range: f32) -> Option<Force> {
    // Scale from BOID_VIEW_RANGE to unit space
    let distance_unit = (target - boid) / view_range;
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

    use super::{FlockingParameters, physics::Force};

    // forces are relative to the boid's view range, so all
    // distances need to be fractions of that

    #[test]
    fn check_cohesion_zero_zero() {
        let props = FlockingParameters::new();
        let force = cohesive_force(Vec2::ZERO, Vec2::ZERO, props.view_range);
        assert!(force.is_none());
    }

    // *********************
    // Cohesion x-axis tests
    // *********************

    #[test]
    fn check_cohesion_midpoint_x_positive() {
        // Pull right 0.5 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.5, 0.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.5 * props.view_range, 0.0),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_midpoint_x_negative() {
        // Pull left 0.5 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(-0.5, 0.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(-0.5 * props.view_range, 0.0),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_edge_x_positive() {
        // pull left 1.0 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(1.0, 0.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0 * props.view_range, 0.0),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_edge_x_negative() {
        // pull left 1.0 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(-1.0, 0.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(-1.0 * props.view_range, 0.0),
                props.view_range
            )
        );
    }

    // *********************
    // Cohesion y-axis tests
    // *********************

    #[test]
    fn check_cohesion_midpoint_y_positive() {
        // Pull up 0.5 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, 0.5, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5 * props.view_range),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_midpoint_y_negative() {
        // Pull down 0.5 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, -0.5, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, -0.5 * props.view_range),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_edge_y_positive() {
        // Pull up 1.0 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, 1.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0 * props.view_range),
                props.view_range
            )
        );
    }

    #[test]
    fn check_cohesion_edge_y_negative() {
        // pull down 0.2 units
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, -1.0, 0.0))),
            cohesive_force(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, -1.0 * props.view_range),
                props.view_range
            )
        );
    }

    // Separation 0,0 test
    #[test]
    fn check_separation_zero_zero() {
        let props = FlockingParameters::new();
        let force = separation_force(Vec2::ZERO, Vec2::ZERO, props.view_range);
        assert!(force.is_none());
    }

    // *********************
    // Separation x-axis tests
    // *********************
    #[test]
    fn check_separation_midpoint_x_positive() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.75, 0.0, 0.0))), // expected force
            separation_force(
                Vec2::new(0.5 * props.view_range, 0.0), // boid position
                Vec2::ZERO,                             // obstacle position
                props.view_range
            )
        );
    }

    #[test]
    fn check_separation_midpoint_x_negative() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(-0.75, 0.0, 0.0))), // expected force
            separation_force(
                Vec2::new(-0.5 * props.view_range, 0.0), // boid position
                Vec2::ZERO,                              // obstacle position
                props.view_range
            )
        );
    }

    #[test]
    fn check_separation_edge_x_positive() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(
                Vec2::new(1.0 * props.view_range, 0.0),
                Vec2::ZERO,
                props.view_range
            ),
        );
    }

    #[test]
    fn check_separation_edge_x_negative() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(
                Vec2::new(-1.0 * props.view_range, 0.0),
                Vec2::ZERO,
                props.view_range
            ),
        );
    }

    // *********************
    // Separation y-axis tests
    // *********************
    #[test]
    fn check_separation_midpoint_y_positive() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, 0.75, 0.0))),
            separation_force(
                Vec2::new(0.0, 0.5 * props.view_range),
                Vec2::ZERO,
                props.view_range
            )
        );
    }

    #[test]
    fn check_separation_midpoint_y_negative() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::new(0.0, -0.75, 0.0))),
            separation_force(
                Vec2::new(0.0, -0.5 * props.view_range),
                Vec2::ZERO,
                props.view_range
            )
        );
    }

    #[test]
    fn check_separation_edge_y_positive() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(
                Vec2::new(0.0, 1.0 * props.view_range),
                Vec2::ZERO,
                props.view_range
            )
        )
    }

    #[test]
    fn check_separation_edge_y_negative() {
        let props = FlockingParameters::new();
        assert_eq!(
            Some(Force(Vec3::ZERO)),
            separation_force(
                Vec2::new(0.0, -1.0 * props.view_range),
                Vec2::ZERO,
                props.view_range
            )
        )
    }
}
