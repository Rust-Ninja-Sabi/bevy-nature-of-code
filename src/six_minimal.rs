use bevy::prelude::*;
use bevy::color::palettes::css::{LIGHT_GRAY,BLUE};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct Ball {
    initial_position: Vec3,
}

#[derive(Resource)]
struct ResetTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
       // .add_plugins(RapierDebugRenderPlugin::default()) for debug
        .insert_resource(ResetTimer(Timer::from_seconds(4.0,TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, auto_reset_ball)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    /* Camera */
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)
    ));

    /* Light */
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        }
    ));

    /* Ground */
    commands.spawn((
        Mesh3d::from(meshes.add(Mesh::from(Cuboid::new(100.0, 0.1, 100.0)))),
        MeshMaterial3d(materials.add(Color::Srgba(LIGHT_GRAY))),
        RigidBody::Fixed,
        Collider::cuboid(100.0, 0.1, 100.0),
    ));

    /* Ball */
    let ball_position = Vec3::new(0.0, 4.0, 0.0);
    
    commands.spawn((
        Mesh3d( meshes.add(Mesh::from(Sphere{ radius:0.5 }))),
        MeshMaterial3d(materials.add(Color::Srgba(BLUE))),
        Transform::from_translation(ball_position),
        RigidBody::Dynamic,
        Collider::ball(0.5),
        ColliderMassProperties::Density(1.2),
        Restitution::coefficient(0.9),
        Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        },
        Ball { initial_position: ball_position }
    ));

}

fn auto_reset_ball(
    mut ball_query: Query<(&Ball, &mut Transform)>,
    time:Res<Time>,
    mut reset_timer: ResMut<ResetTimer>,
) {
    if reset_timer.0.tick(time.delta()).just_finished() {
        for (ball, mut transform) in ball_query.iter_mut() {
            // reset position
            transform.translation = ball.initial_position;
        }
    }
}