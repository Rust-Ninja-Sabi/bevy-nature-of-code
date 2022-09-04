use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy::render::mesh::PrimitiveTopology;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
mod orbitcamera;


const HEIGHT: f32 = 440.0;
const WIDTH: f32 = 812.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

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
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            width: WIDTH,
            height: HEIGHT,
            title:"Example 1.2 1.8".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(UiAcceleration{value:0.0})
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_limit_cube)
        .add_system(ui_egui)
        .add_system(moving)
        .run();
}

fn spawn_scene(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere{
            radius:0.5,
            sectors:32,
            stacks:32
        })),
        material: materials.add(Color::BLUE.into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    })
        .insert(Moveable{
            velocity: Vec3::new(1.0, 1.0, 1.0),
            acceleration: Vec3::new(0.0, 0.0,0.0)
        });

    //light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });
}

fn spawn_camera(
    mut commands:Commands
){
    commands.spawn_bundle(Camera3dBundle{
        ..default()
    })
        .insert(OrbitCamera{
            distance : 28.0,
            ..default()
        });
}

fn moving(
    time:Res<Time>,
    ui_acceleration: Res<UiAcceleration>,
    mut query: Query<(&mut Transform, &mut Moveable)>
){
    for (mut transform, mut moveable) in &mut query {
        moveable.acceleration = moveable.velocity * 0.01 *  ui_acceleration.value;
        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_seconds();
        transform.translation = transform.translation + moveable.velocity * time.delta_seconds();

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

fn spawn_limit_cube(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    let min_limit = MIN_LIMIT;
    let max_limit= MAX_LIMIT;

    let ind = vec![vec![0, 1, 2, 3, 0],
                                  vec![4, 5, 6, 7, 4],
                                  vec![0, 4],vec![1, 5],vec![2, 6],vec![3, 7]];

    let mut indices:Vec<u32> = Vec::new();
    indices.push(0);
    indices.push(1);

    let mut positions = Vec::new();
    for l in vec![max_limit, min_limit] {
        positions.push([max_limit, l, max_limit]);
        positions.push([max_limit, l, min_limit]);
        positions.push([min_limit, l, min_limit]);
        positions.push([min_limit, l, max_limit]);
    }

    let mut normals = Vec::new();
    for _ in 0..8 {
        normals.push([0.0, 1.0, 0.0]);
    }

    let mut indices:Vec<u32> = Vec::new();

    for k in ind {
        let mut j = 0;
        let mut first = true;

        for i in k {
            if !first {
                indices.push(j);
                indices.push(i);
            } else {
                first = false
            };
            j = i;
        }
    }
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::LIME_GREEN.into()),
            ..Default::default()
        });
}

fn ui_egui(
    mut egui_context: ResMut<EguiContext>,
    mut ui_acceleration: ResMut<UiAcceleration>
){
    egui::Window::new("Properties").show(egui_context.ctx_mut(), |ui|{
        ui.add(egui::Slider::new(&mut (ui_acceleration.value), -50.0..=50.0).text("Acceleration"));
    });
}