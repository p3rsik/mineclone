use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;
use block::BlockPlugin;
use camera::CameraPlugin;
use chunk::ChunkPlugin;

mod block;
mod camera;
mod chunk;

fn main() {
    App::new()
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
        .add_plugins(CameraPlugin)
        .add_plugins(BlockPlugin)
        .add_plugins(ChunkPlugin)
        // .add_systems(Startup, spawn_objects)
        .add_systems(Startup, setup_lights)
        .add_systems(Startup, spawn_stone_cube)
        .add_systems(Update, cursor_grab)
        .run();
}

fn spawn_stone_cube(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let stone_texture = asset_server.load("stone.png");
    let cube = Cuboid {
        half_size: Vec3::new(0.5, 0.5, 0.5),
    };
    let mesh_h = meshes.add(cube.mesh());
    let material_h = materials.add(StandardMaterial {
        base_color_texture: Some(stone_texture),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: mesh_h,
        material: material_h,
        transform: Transform::from_xyz(2.0, 1.0, 0.0),
        ..default()
    });
}

fn setup_lights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = Cuboid {
        half_size: Vec3::new(0.4, 0.4, 0.4),
    };
    let mesh = meshes.add(cube.mesh());
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 500000.0,
            range: 10000.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 10.0, 0.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..default()
    });
}

fn cursor_grab(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_window.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor.visible = false;
}
