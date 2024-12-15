use bevy::color::palettes::basic::BLUE;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;
use rand::Rng;

use orbitcamera::{OrbitCamera, OrbitCameraPlugin};
mod orbitcamera;
mod mesh;

const HEIGHT: f32 = 440.0;
const WIDTH: f32 = 812.0;

const MAX_LIMIT: f32 = 8.0;
const MIN_LIMIT: f32 = -8.0;

#[derive(Component)]
struct Particle {
    velocity:Vec3,
    acceleration: Vec3,
    lifetime:Timer
}

#[derive(Component, Clone, Copy)]
pub struct ParticleAlpha {
    start: f32,
    end: f32,
}

impl Particle {
    fn apply_force(&mut self, force:Vec3)->(){
        self.acceleration = self.acceleration +  force;
    }

    fn get_factor(&self)->f32{
        self.lifetime.elapsed().as_secs_f32()/self.lifetime.duration().as_secs_f32()
    }
}

#[derive(Component)]
struct ParicleEmiter {
    spawn_time:Timer,
    particle_time: f32,
    amount_per_burst: u32,
    position_variance: f32,
    particle_alpha: Option<ParticleAlpha>
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Example 4".to_string(),
                resolution: WindowResolution::new(WIDTH,  HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((OrbitCameraPlugin,
                      EguiPlugin))
        .add_systems(Startup, (spawn_camera,
                               spawn_scene,
                               mesh::spawn_limit_cube))
        .add_systems(Update, (emit_particles,
                             update_particle_lifetime,
                             moving,
                             update_particle_alpha.after(emit_particles)))
        .run();
}

fn spawn_scene(
    mut commands:Commands
){
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

    // particle emiter
    commands
        .spawn((
                   Transform::from_xyz(0.0,MAX_LIMIT,0.0),
            ParicleEmiter{
                spawn_time: Timer::from_seconds(0.05,TimerMode::Repeating),
                particle_time: 4.0,
                amount_per_burst: 20,
                position_variance: 2.0,
                particle_alpha: Some(ParticleAlpha{
                    start: 1.0,
                    end: 0.0
                })
            }));
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
    mut query: Query<(&mut Transform, &mut Particle)>
){
    for (mut transform, mut moveable) in &mut query {
        let gravity = Vec3::new(0.0,-0.5,0.0);
        moveable.apply_force(gravity);
        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_secs();
        transform.translation = transform.translation + moveable.velocity * time.delta_secs();

    }
}

fn emit_particles(
    mut commands:Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut ParicleEmiter, &Transform)>
){
    for (mut emiter, transform) in query.iter_mut(){
        emiter.spawn_time.tick(time.delta());
        if emiter.spawn_time.just_finished(){
            for _ in 0..emiter.amount_per_burst{
                let mut rng = rand::thread_rng();
                let particle = commands.spawn((
                    Mesh3d( meshes.add(Mesh::from( Cuboid::new(0.25, 0.25,0.25)))),
                    MeshMaterial3d( materials.add(StandardMaterial{
                                                        base_color:Color::Srgba(BLUE),
                                                        alpha_mode: AlphaMode::Blend,
                                                        ..default()})),
                    Transform::from_xyz(transform.translation.x + rng.gen_range(-emiter.position_variance..emiter.position_variance),
                                        transform.translation.y + rng.gen_range(-emiter.position_variance..emiter.position_variance),
                                        transform.translation.z + rng.gen_range(-emiter.position_variance..emiter.position_variance)),
                    Particle {
                        lifetime: Timer::from_seconds(emiter.particle_time,TimerMode::Once),
                        velocity: Vec3::new(1.0, 1.0, 1.0),
                        acceleration: Vec3::new(0.0, 0.0,0.0)
                    }
                )).id();

                if let Some(alpha) = emiter.particle_alpha {
                    commands.entity(particle).insert(ParticleAlpha{
                        start:alpha.start,
                        end:alpha.end
                    });
                }
            }
        }
    }
}

fn update_particle_lifetime(
    mut commands:Commands,
    mut query: Query<(Entity, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn update_particle_alpha(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Particle,&ParticleAlpha, &MeshMaterial3d<StandardMaterial>),With<ParticleAlpha>>
) {
    for (particle, alpha, material) in query.iter_mut() {
        if let Some( mat) = materials.get_mut(material) {
            mat.base_color.set_alpha(lerp(alpha.start, alpha.end, particle.get_factor()));
        }
    }
}