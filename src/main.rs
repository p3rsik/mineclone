use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use block::BlockPlugin;
use camera::CameraPlugin;
use chunk::ChunkPlugin;
use common::AppState;
use config::ConfigPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;
use world::GameWorldPlugin;

mod block;
mod camera;
mod chunk;
mod common;
mod config;
mod player;
mod registry;
mod ui;
mod world;

fn main() {
    App::new()
        .init_state::<AppState>()
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
            UiPlugin,
            BlockPlugin,
            ConfigPlugin,
            CameraPlugin,
            ChunkPlugin,
            PlayerPlugin,
            GameWorldPlugin,
        ))
        .run();
}
