use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default, Deref, DerefMut, PartialEq, Debug)]
pub struct Force(pub Vec3);

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity, &mut Force)>, time: Res<Time>) {
    for (mut transform, velocity, mut acceleration) in &mut query {
        let delta_v = **acceleration * time.delta_secs();
        **acceleration = Vec3::ZERO;
        let delta_position = (**velocity + delta_v) * time.delta_secs();
        transform.translation += delta_position;
    }
}
