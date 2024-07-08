
use bevy::{math::VectorSpace, prelude::*, sprite::MaterialMesh2dBundle};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin{
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (spawn_camera, spawn_boids))
            .add_systems(FixedUpdate, (
                apply_velocity,
                check_keyboard,
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

#[derive(Bundle)]
struct BoidBundle {
    boid: Boid,
    velocity: Velocity,
}

impl BoidBundle {
    fn new(vel: Vec3) -> Self {
        Self {
            boid: Boid,
            velocity: Velocity(vel),
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
        let vel = Vec3::new(
            frac.cos() * 10.0,
            frac.sin() * 10.0,
            0.0,
        );
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
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        Boid
    ));

    commands.spawn((
        BoidBundle::new(Vec3::new(0.0, 0.0, 0.0)),
        PlayerBoid,
        MaterialMesh2dBundle {
            mesh: meshes.add(Triangle2d::default()).into(),
            material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
            ..default()
        }
    ));
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
