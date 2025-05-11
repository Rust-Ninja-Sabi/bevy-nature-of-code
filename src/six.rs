use std::time::Duration;
use bevy::prelude::*;
use bevy::color::palettes::basic::BLUE;
use bevy::color::palettes::css::LIGHT_GRAY;
use bevy::time::common_conditions::on_timer;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;
use orbitcamera::{OrbitCameraPlugin, OrbitCamera};
use crate::mesh::spawn_limit_cube;

mod orbitcamera;
mod mesh;
use rand::Rng;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

const BOARD_WIDTH: f32 = 8.0;
const BOARD_HEIGHT: f32 = 16.0;

const PIN_RADIUS: f32 = 0.2;
const PIN_HEIGHT:f32 = 0.6;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Board;

const BALL_RADIUS: f32 = 0.25;

#[derive(Resource)]
struct UiValues{
    inclination: usize,
    num_of_spheres: usize,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))

        .insert_resource(UiValues{
            inclination: 12,
            num_of_spheres: 1
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Example Chapter 6".to_string(),
                resolution: WindowResolution::new(WIDTH,  HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins((
            EguiPlugin,
            //RapierDebugRenderPlugin::default(),
            OrbitCameraPlugin
        ))
        .add_systems(Startup, (
            spawn_camera,
            spawn_scene,
            spawn_board,
            spawn_limit_cube
        ))
        .add_systems(
            Update,
            spawn_ball.run_if(on_timer(Duration::from_secs(1))),
        )
        .add_systems(Update, (
            ui_egui,
            rotate_board,
            despawn_ball
        ))
        .run();
}

fn spawn_scene(
    mut commands:Commands
){
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

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
}

fn spawn_board(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>)
{
    let cuboid = Cuboid::new(BOARD_WIDTH, 0.25, BOARD_HEIGHT);

    let board = commands.spawn((
        Mesh3d::from(meshes.add(Mesh::from(cuboid))),
        MeshMaterial3d(materials.add(Color::Srgba(LIGHT_GRAY))),
        RigidBody::Fixed,
        Collider::cuboid(cuboid.half_size.x, cuboid.half_size.y, cuboid.half_size.z),
        Board
    )).id();

    let pin_mesh = meshes.add(Mesh::from(Cylinder {
        radius: PIN_RADIUS,
        half_height: PIN_HEIGHT/2.0
    }));

    let pin_material = materials.add(Color::rgb(0.8, 0.1, 0.1));

    let spacing = 1.6;
    let rows = 4;
    let cols = 5;

    let start_z =  -BOARD_HEIGHT/2.0 + 1.5; // Rechte Seite
    let start_x = -((rows as f32 - 1.0) / 2.0) * spacing - 1.0;

    for row in 0..rows {
        for col in 0..cols {
            let row_offset = if row % 2 == 1 { spacing / 2.0 } else { 0.0 };
            let x = start_x + col as f32 * spacing + row_offset;
            let z = start_z + row as f32 * spacing;
            let transform = Transform::from_xyz(x, PIN_HEIGHT / 2.0, z);

            commands.entity(board).with_children(|parent| {
                parent.spawn((
                    Mesh3d(pin_mesh.clone()),
                    MeshMaterial3d(pin_material.clone()),
                    transform,
                    RigidBody::Fixed,
                    Collider::cylinder(PIN_HEIGHT/2.0, PIN_RADIUS),
                    ColliderMassProperties::Density(2.0)
                ));
            });
        }
    }
}

fn spawn_camera(
    mut commands:Commands
){
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        OrbitCamera{
            distance : 28.0,
            ..default()
        }
    ));
}

fn spawn_ball(
    mut commands:Commands,
    ui_values: Res<UiValues>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    for _ in 1..=ui_values.num_of_spheres {
        let mut rng = rand::rng();
        
        commands.spawn((
            Mesh3d( meshes.add(Mesh::from(Sphere{ radius:BALL_RADIUS }))),
            MeshMaterial3d( materials.add( StandardMaterial {
                base_color: Color::Srgba(BLUE),
                ..Default::default()
            })),
            Transform::from_xyz(rng.random_range((-BOARD_WIDTH/2.0)..(BOARD_WIDTH/2.0)),
                                MAX_LIMIT,
                                rng.random_range((2.0 .. (BOARD_HEIGHT/2.0)))),
            Ball,
            RigidBody::Dynamic,
            Collider::ball(BALL_RADIUS),
            Restitution::coefficient(0.6)
        ));
    }
}

fn despawn_ball(
    mut commands: Commands,
    mut query: Query<(Entity,&mut Transform), With<Ball>>
) {
    for (e, transform) in query.iter_mut(){
        if transform.translation.y <=MIN_LIMIT-10.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn rotate_board(
    mut query: Query<&mut Transform, With<Board>>,
    ui_values: ResMut<UiValues>
){
    for mut transform in query.iter_mut(){
        transform.rotation = Quat::from_rotation_x(-(ui_values.inclination as f32).to_radians());
    }
}


fn ui_egui(
    mut egui_contexts: EguiContexts,
    mut ui_values: ResMut<UiValues>,
){
    egui::Window::new("Properties").show(egui_contexts.ctx_mut(), |ui|{
        ui.add(egui::Slider::new(&mut (ui_values.num_of_spheres), 1..=20).text("Spheres p. Sec."));
        ui.add(egui::Slider::new(&mut (ui_values.inclination), 0..=45).text("Inclination in Degrees"));
    });
}