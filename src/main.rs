use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use block::{asset::BlockAssetLoader, Block, BlockPlugin};
use camera::CameraPlugin;
use chunk::ChunkPlugin;
use common::AppState;
use config::ConfigPlugin;
use player::PlayerPlugin;
use world::GameWorldPlugin;

mod block;
mod camera;
mod chunk;
mod common;
mod config;
mod player;
mod registry;
mod world;

fn main() {
    App::new()
        .init_state::<AppState>()
        .init_asset::<Block>()
        .init_asset_loader::<BlockAssetLoader>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            brightness: 0.75,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // .add_plugins((
        //     DefaultPlugins.set(RenderPlugin {
        //         render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
        //             features: WgpuFeatures::POLYGON_MODE_LINE,
        //             ..default()
        //         }),
        //         ..default()
        //     }),
        //     WireframePlugin,
        // ))
        // .insert_resource(WireframeConfig {
        //     global: true,
        //     default_color: Color::WHITE,
        // })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // here is the gravity in rapier, I searched to long on the internet, so I will save it here for future reference
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec3::ZERO,
        //     ..default()
        // })
        .add_plugins((
            BlockPlugin,
            ConfigPlugin,
            CameraPlugin,
            ChunkPlugin,
            PlayerPlugin,
            GameWorldPlugin,
        ))
        .add_systems(Startup, setup_lights)
        .add_systems(Update, cursor_grab)
        .run();
}

// rudimentary lights
fn setup_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = Sphere { radius: 0.5 };
    let mesh = meshes.add(sphere.mesh());
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 50000000.0,
            range: 1000000.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 50.0, 0.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, 50.0, 0.0),
        ..default()
    });
}

fn cursor_grab(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_window.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}
