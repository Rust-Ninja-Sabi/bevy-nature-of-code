use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy::time::FixedTimestep;
use bevy::render::mesh::PrimitiveTopology;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
mod orbitcamera;

use rand::Rng;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

struct UiValues{
    wind: bool,
    num_of_spheres: u8
}

#[derive(Component)]
struct Moveable {
    velocity:Vec3,
    acceleration: Vec3,
    mass: f32
}

impl Default for Moveable {
    fn default() -> Self {
        Moveable {
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0
        }
    }
}

impl Moveable {
    fn apply_force(&mut self, force:Vec3)->(){
       self.acceleration = self.acceleration +  force/self.mass;
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            width: WIDTH,
            height: HEIGHT,
            title:"Example 2.1 2.2".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(UiValues{
            wind: true,
            num_of_spheres: 1
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_limit_cube)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(spawn_sphere)
        )
        .add_system(ui_egui)
        .add_system(despawn_sphere)
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
            rotation: Quat::from_rotation_x(-std::f32::consts::PI),
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

fn spawn_sphere(
    mut commands:Commands,
    ui_values: Res<UiValues>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    for _ in 1..=ui_values.num_of_spheres {
        let mut rng = rand::thread_rng();
        let mass = rng.gen_range(1.0..2.4);
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.5 * mass,
                sectors: 32,
                stacks: 32
            })),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT),
                                           MAX_LIMIT,
                                           rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT)),
            ..default()
        })
            .insert(Moveable {
                velocity: Vec3::new(0.0, 0.0, 0.0),
                mass: mass,
                ..default()
            });
    }
}

fn despawn_sphere(
    mut commands: Commands,
    mut query: Query<(Entity,&mut Transform), With<Moveable>>,
) {
    for (e, transform) in query.iter_mut(){
        if transform.translation.y <=MIN_LIMIT-10.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn moving(
    time:Res<Time>,
    ui_values: Res<UiValues>,
    mut query: Query<(&mut Transform, &mut Moveable)>
){
    let wind = Vec3::new(8.0,0.0,0.0);

    for (mut transform, mut moveable) in &mut query {

        let gravity = Vec3::new(0.0,-10.0,0.0) * moveable.mass;
        if ui_values.wind {
            moveable.apply_force(wind);
        }
        moveable.apply_force(gravity);

        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_seconds();
        transform.translation = transform.translation + moveable.velocity * time.delta_seconds();

        moveable.acceleration *= 0.0;
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
    mut ui_values: ResMut<UiValues>,
){
    egui::Window::new("Properties").show(egui_context.ctx_mut(), |ui|{
        ui.add(egui::Slider::new(&mut (ui_values.num_of_spheres), 1..=20).text("Spheres p. Sec."));
        ui.add(egui::Checkbox::new(&mut (ui_values.wind),"wind"));
    });
}