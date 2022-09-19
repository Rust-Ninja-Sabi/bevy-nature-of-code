use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;

use rand::Rng;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
mod orbitcamera;


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
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            width: WIDTH,
            height: HEIGHT,
            title:"Example 4".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_limit_cube)
        .add_system(emit_particles)
        .add_system(update_particle_lifetime)
        .add_system(update_particle_alpha.after(emit_particles))
        .add_system(moving)
        .run();
}

fn spawn_scene(
    mut commands:Commands
){
    //light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(std::f32::consts::PI),
            ..default()
        },
        ..default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.02,
    });

    // particle emiter
    commands
        .spawn_bundle(TransformBundle{
            local: Transform::from_xyz(0.0,MAX_LIMIT,0.0),
            ..default()
        })
        .insert(ParicleEmiter{
            spawn_time: Timer::from_seconds(0.05,true),
            particle_time: 4.0,
            amount_per_burst: 20,
            position_variance: 2.0,
            particle_alpha: Some(ParticleAlpha{
                start: 1.0,
                end: 0.0
            })
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
    mut query: Query<(&mut Transform, &mut Particle)>
){
    for (mut transform, mut moveable) in &mut query {
        let gravity = Vec3::new(0.0,-0.5,0.0);
        moveable.apply_force(gravity);
        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_seconds();
        transform.translation = transform.translation + moveable.velocity * time.delta_seconds();

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
                let particle = commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube{size:0.25})),
                    material: materials.add(StandardMaterial{
                                                        base_color:Color::BLUE,
                                                        alpha_mode: AlphaMode::Blend,
                                                        ..default()}),
                    transform: Transform::from_xyz(transform.translation.x + rng.gen_range(-emiter.position_variance..emiter.position_variance),
                                                   transform.translation.y + rng.gen_range(-emiter.position_variance..emiter.position_variance),
                                                   transform.translation.z + rng.gen_range(-emiter.position_variance..emiter.position_variance),),
                    ..default()
                })
                    .insert(Particle {
                        lifetime: Timer::from_seconds(emiter.particle_time,false),
                        velocity: Vec3::new(1.0, 1.0, 1.0),
                        acceleration: Vec3::new(0.0, 0.0,0.0)
                    }).id();

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
    mut query: Query<(&Particle, &ParticleAlpha, &Handle<StandardMaterial>)>
) {
    for (particle, alpha, material) in query.iter_mut() {
        if let Some( mat) = materials.get_mut(material) {
            mat.base_color.set_a(lerp(alpha.start, alpha.end, particle.get_factor()));
        }
    }
}