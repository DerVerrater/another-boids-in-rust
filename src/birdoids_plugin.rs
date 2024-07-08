
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

const BACKGROUND_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin{
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (spawn_camera, spawn_boids))
            .add_systems(FixedUpdate, apply_velocity);
    }
}

#[derive(Component)]
struct Boid;

#[derive(Component, Deref)]
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
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        let delta_position = **velocity * time.delta_seconds();
        transform.translation += delta_position;
    }
}
