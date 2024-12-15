use bevy::prelude::*;
use bevy::color::palettes::basic::BLUE;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::window::WindowResolution;
use orbitcamera::{OrbitCameraPlugin, OrbitCamera};
use crate::mesh::spawn_limit_cube;

mod mesh;
mod orbitcamera;


const HEIGHT: f32 = 440.0;
const WIDTH: f32 = 812.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

#[derive(Resource)]
struct UiAcceleration{
    value: f32
}

#[derive(Component)]
struct Moveable {
    velocity:Vec3,
    acceleration: Vec3
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(UiAcceleration{value:0.0})
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Example 1.2 1.8".to_string(),
                resolution: WindowResolution::new(WIDTH,  HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
                         EguiPlugin,
                         OrbitCameraPlugin
                     ))
        .add_systems(Startup, (spawn_camera, spawn_scene, spawn_limit_cube))
        .add_systems(Update, (ui_egui,moving))
        .run();
}

fn spawn_scene(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Sphere{
            radius:0.5
        }))),
        MeshMaterial3d(materials.add( StandardMaterial {
            base_color: Color::Srgba(BLUE),
            ..Default::default()
            })),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ))
        .insert(Moveable{
            velocity: Vec3::new(1.0, 1.0, 1.0),
            acceleration: Vec3::new(0.0, 0.0,0.0)
        });

    //light
    commands.spawn((DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(std::f32::consts::PI),
            ..default()
        }
    ));
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
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

fn moving(
    time:Res<Time>,
    ui_acceleration: Res<UiAcceleration>,
    mut query: Query<(&mut Transform, &mut Moveable)>
){
    for (mut transform, mut moveable) in &mut query {
        moveable.acceleration = moveable.velocity * 0.01 *  ui_acceleration.value;
        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_secs();
        transform.translation = transform.translation + moveable.velocity * time.delta_secs();

        if transform.translation.x  > MAX_LIMIT || transform.translation.x < MIN_LIMIT {
            moveable.velocity.x = moveable.velocity.x * -1.0;
        }

        if transform.translation.y  > MAX_LIMIT || transform.translation.y < MIN_LIMIT {
            moveable.velocity.y = moveable.velocity.y * -1.0;
        }

        if transform.translation.z  > MAX_LIMIT || transform.translation.z < MIN_LIMIT {
            moveable.velocity.z = moveable.velocity.z * -1.0;
        }
    }
}



fn ui_egui(
    mut egui_contexts: EguiContexts,
    mut ui_acceleration: ResMut<UiAcceleration>
){
    egui::Window::new("Properties").show(egui_contexts.ctx_mut(), |ui|{
        ui.add(egui::Slider::new(&mut (ui_acceleration.value), -50.0..=50.0).text("Acceleration"));
    });
}