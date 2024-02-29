use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{block::BLOCK_HALF_SIZE, config::GameConfig, player::Player};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraPerspective::default())
            .add_systems(Update, (change_perspective, change_camera_origin))
            .add_systems(Update, (block_selection, draw_box_aroud_object));
    }
}

// Used to determine at what entity camera is looking
#[derive(Component, Debug)]
pub struct LookingAt {
    // Entity which the player is looking at
    pub entity: Entity,
    // Intersection info
    pub intersection: RayIntersection,
    // Position of block in global coords
    pub block_pos: Vec3,
}

#[derive(Resource, Default, PartialEq)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    ThirdPerson,
    ThirdPersonInverted,
}

impl CameraPerspective {
    fn next(&mut self) {
        match self {
            Self::FirstPerson => *self = Self::ThirdPerson,
            Self::ThirdPerson => *self = Self::ThirdPersonInverted,
            Self::ThirdPersonInverted => *self = Self::FirstPerson,
        }
    }
}

#[derive(Component)]
pub struct FirstPersonCamera {
    pub sensitivity: f32,
    pub velocity: Vec3,
    pub friction: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        FirstPersonCamera {
            sensitivity: 8.0,
            velocity: Vec3::ZERO,
            friction: 0.7,
        }
    }
}

fn change_perspective(
    k_input: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut camera_perspective: ResMut<CameraPerspective>,
) {
    if k_input.just_pressed(config.key_config.camera_controls.switch_perspective) {
        camera_perspective.next();
    }
}

fn change_camera_origin(
    camera_perspective: Res<CameraPerspective>,
    mut camera_query: Query<&mut Transform, (With<FirstPersonCamera>, Without<Player>)>,
) {
    if camera_perspective.is_changed() {
        let mut camera_transform = camera_query.single_mut();
        // We cycle through camera perspectives one-by-one, so we can be sure, that when we arrive at
        // one of them, it would be from the previous perspective in order.
        // TODO implement minecraft style third-person camera
        match *camera_perspective {
            CameraPerspective::FirstPerson => {
                camera_transform.translation = Vec3::new(0.0, 2.0, 0.0);
            }
            CameraPerspective::ThirdPerson => {
                camera_transform.translation = Vec3::new(0.0, 3.0, 5.0);
            }
            // TODO figure this out
            CameraPerspective::ThirdPersonInverted => {
                camera_transform.translation = Vec3::new(0.0, 3.0, -5.0)
            }
        };
    }
}

fn block_selection(
    mut commands: Commands,
    camera_query: Query<&GlobalTransform, With<FirstPersonCamera>>,
    player_entity_query: Query<(Entity, &Transform), With<Player>>,
    looked_at_query: Query<Entity, With<LookingAt>>,
    rapier_context: Res<RapierContext>,
) {
    let camera_transform = camera_query.single();
    let (player_entity, player_transform) = player_entity_query.single();
    let ray_pos = player_transform.translation + Vec3::Y * 2.0;
    let ray_dir = camera_transform.forward().xyz();
    let max_toi = 8.0;
    let solid = true;
    let filter = QueryFilter::new().exclude_collider(player_entity);

    for looked_at in looked_at_query.iter() {
        commands.entity(looked_at).remove::<LookingAt>();
    }
    if let Some((entity, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter)
    {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            let point = intersection.point;
            entity_commands.insert(LookingAt {
                entity,
                intersection,
                block_pos: Vec3::new(point.x.floor(), point.y.floor(), point.z.floor()),
            });
        }
    };
}

fn draw_box_aroud_object(
    mut gizmos: Gizmos,
    // blocks are now children of Chunk, so Transform is local
    looked_at_query: Query<(&LookingAt, &GlobalTransform)>,
) {
    let cube = Cuboid {
        half_size: Vec3::splat(BLOCK_HALF_SIZE + 0.001),
    };

    for (looking_at, global_transform) in looked_at_query.iter() {
        let (_scale, rotation, _translation) = global_transform.to_scale_rotation_translation();
        // println!(
        //     "Looking at {}, block {}",
        //     looking_at.intersection.point, looking_at.block_pos
        // );
        gizmos.primitive_3d(
            cube,
            looking_at.block_pos + Vec3::splat(BLOCK_HALF_SIZE)
                - 2.0 * BLOCK_HALF_SIZE * looking_at.intersection.normal,
            rotation,
            Color::WHITE,
        );
    }
}
