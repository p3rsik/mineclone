use bevy::prelude::*;

pub fn setup_global_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::IDENTITY).looking_at(-Vec3::Y, Vec3::Z),
        ..default()
    });
}

// TODO implement
pub fn day_night_cycle() {}
