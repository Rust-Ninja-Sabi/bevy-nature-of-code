use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy::render::mesh::PrimitiveTopology;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
mod orbitcamera;

use std::f32::consts::PI;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const RADIUS: f32 = 8.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

struct UiValues{
    restart: bool
}

#[derive(Component)]
struct Pendulum {
    angle: f32,
    position: Vec3,
    origin: Vec3,
    circle_velocity: f32,
    circle_acceleration: f32,
    daming: f32,
}

impl Default for Pendulum {
    fn default() -> Self {
        Pendulum {
            angle: PI/4.0,
            origin: Vec3::new(0.0, MAX_LIMIT, 0.0),
            position: Vec3::ZERO,
            circle_velocity:0.0,
            circle_acceleration:0.0,
            daming : 0.998
        }
    }
}

impl Pendulum {
    fn reset(&mut self) -> () {
            self.angle =  PI/4.0;
            self.origin =  Vec3::new(0.0, MAX_LIMIT, 0.0);
            self.position =  Vec3::ZERO;
            self.circle_velocity = 0.0;
            self.circle_acceleration = 0.0;
            self.daming = 0.998;
    }
}

#[derive(Component)]
struct PendulumLine{}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            width: WIDTH,
            height: HEIGHT,
            title:"Example 3".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(UiValues{
            restart: true
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_limit_cube)
        .add_startup_system(spawn_pendulum)
        .add_system(ui_egui)
        .add_system(moving)
        .run();
}

fn spawn_scene(
    mut commands:Commands,
){
    //light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(-PI),
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

fn spawn_pendulum(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    //line
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.2, RADIUS, 0.2))),
        material: materials.add(StandardMaterial {
            base_color: Color::LIME_GREEN,
            emissive: Color::LIME_GREEN,
            ..Default::default()
        }),
        transform: Transform {
            translation: Vec3::new(0.0, RADIUS/2.0, 0.0),
            ..default()
        },
        ..Default::default()
    })
        .insert(PendulumLine{});

    //sphere
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 1.0,
                sectors: 32,
                stacks: 32
            })),
            material: materials.add(Color::BLUE.into()),
            ..default()
        })
            .insert(Pendulum{..default()});
}

fn moving(
    time:Res<Time>,
    mut ui_values: ResMut<UiValues>,
    mut query: Query<(&mut Transform, &mut Pendulum)>,
    mut query_line: Query<&mut Transform, (With<PendulumLine>,Without<Pendulum>)>
){
    let (mut transform, mut pendulum) = query.single_mut();
    let mut line_transform = query_line.single_mut();
    let gravity = 40.0;

    if ui_values.restart {
        ui_values.restart = false;
        pendulum.reset();
    }
    pendulum.circle_acceleration = (-1.0 * gravity / RADIUS) * pendulum.angle.sin();

    pendulum.circle_velocity += pendulum.circle_acceleration * time.delta_seconds();
    pendulum.angle += pendulum.circle_velocity * time.delta_seconds();

    pendulum.circle_velocity *= pendulum.daming;

    pendulum.position = pendulum.origin -
                        Vec3::new(RADIUS*pendulum.angle.sin(),
                                  RADIUS*pendulum.angle.cos(),
                                  0.0);

    transform.translation = pendulum.position;

    line_transform.rotation = Quat::from_rotation_z(-pendulum.angle);
    line_transform.translation = pendulum.origin + Vec3::new(-pendulum.angle.sin() * RADIUS/2.0,
                                                             -pendulum.angle.cos() * RADIUS/2.0,
                                                             0.0);

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
    mut ui_values: ResMut<UiValues>,
){
    egui::Window::new("Properties").show(egui_context.ctx_mut(), |ui|{
        if ui.button("Restart").clicked() {
            ui_values.restart = true;
        }
    });
}