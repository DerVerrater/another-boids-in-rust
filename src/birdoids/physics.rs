use bevy::prelude::*;

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default, Deref, DerefMut, PartialEq, Debug)]
pub struct Force(pub Vec3);

pub fn apply_velocity(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Force)>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, mut force) in &mut query {
        // integrate forces into new velocity (assume mass of 1kg)
        let delta_v = **force * time.delta_secs();
        velocity.0 += delta_v;
        force.0 = Vec3::ZERO; // clear force value

        // integrate velocity into position
        let delta_p = **velocity * time.delta_secs();
        transform.translation += delta_p;
    }
}
