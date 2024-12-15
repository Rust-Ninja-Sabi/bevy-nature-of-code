use std::time::Duration;
use bevy::prelude::*;
use bevy::color::palettes::basic::BLUE;
use bevy::time::common_conditions::on_timer;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::window::WindowResolution;
use orbitcamera::{OrbitCameraPlugin, OrbitCamera};
use crate::mesh::spawn_limit_cube;

mod orbitcamera;
mod mesh;
use rand::Rng;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

#[derive(Resource)]
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

        .insert_resource(UiValues{
            wind: true,
            num_of_spheres: 1
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Example 2.1 2.2".to_string(),
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
        .add_systems(Startup, (spawn_camera,
                               spawn_scene,spawn_limit_cube)
        )
        .add_systems(
            Update,
            spawn_sphere.run_if(on_timer(Duration::from_secs(1))),
        )
        .add_systems(Update, (ui_egui,
                              despawn_sphere,
                              moving)
        )
        .run();
}

fn spawn_scene(
    mut commands:Commands,
){
    //light
    commands.spawn((
                       DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI),
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

fn spawn_sphere(
    mut commands:Commands,
    ui_values: Res<UiValues>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    for _ in 1..=ui_values.num_of_spheres {
        let mut rng = rand::thread_rng();
        let mass = rng.gen_range(1.0..2.4);
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Sphere {
                radius: 0.5 * mass,
            }))),
            MeshMaterial3d( materials.add( StandardMaterial {
                base_color: Color::Srgba(BLUE),
                ..Default::default()
            })),
            Transform::from_xyz(rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT),
                                           MAX_LIMIT,
                                           rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT)),
            Moveable {
                velocity: Vec3::new(0.0, 0.0, 0.0),
                mass: mass,
                ..default()
            }
        ));
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

        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_secs();
        transform.translation = transform.translation + moveable.velocity * time.delta_secs();

        moveable.acceleration *= 0.0;
    }
}


fn ui_egui(
    mut egui_contexts: EguiContexts,
    mut ui_values: ResMut<UiValues>,
){
    egui::Window::new("Properties").show(egui_contexts.ctx_mut(), |ui|{
        ui.add(egui::Slider::new(&mut (ui_values.num_of_spheres), 1..=20).text("Spheres p. Sec."));
        ui.add(egui::Checkbox::new(&mut (ui_values.wind),"wind"));
    });
}