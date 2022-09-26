use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
mod orbitcamera;

use rand::Rng;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const MAX_LIMIT: f32 = 32.0;
const MIN_LIMIT: f32 = -32.0;

const NUM_MOVEABLE: u32 = 1024;
struct Mover(Vec<u32>);

const MAX_SPEED:f32=32.0;
const MAX_FORCE:f32=16.0;

#[derive(Component)]
struct Moveable {
    force:Vec3,
    velocity:Vec3,
    acceleration: Vec3,
    maximum_speed: f32,
    maximum_force: f32,
    mass: f32
}

impl Default for Moveable {
    fn default() -> Self {
        Moveable {
            force: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            maximum_speed: MAX_SPEED,
            maximum_force: MAX_FORCE,
            mass: 1.0
        }
    }
}

impl Moveable {
    fn apply_force(&mut self, force:Vec3)->(){
       self.acceleration = self.acceleration +  force/self.mass;
    }

    fn seek(&mut self, target:Vec3, position:Vec3)->Vec3{
        //steering force = desired velocity - velocity
        let desired_velocity = (target - position).normalize()*self.maximum_speed;
        let steering_force = desired_velocity - self.velocity;
        steering_force.clamp_length(0.0,self.maximum_force)
    }

    fn flee(&mut self, target:Vec3, position:Vec3)->Vec3{
        -self.seek(target, position)
    }

    fn random(&mut self)->Vec3{
        let mut rng = rand::thread_rng();

        let steering_force = Vec3::new(
            rng.gen_range(10.0*MIN_LIMIT..10.0*MAX_LIMIT),
            rng.gen_range(10.0*MIN_LIMIT..10.0*MAX_LIMIT),
            rng.gen_range(10.0*MIN_LIMIT..10.0*MAX_LIMIT)
        );
        steering_force.clamp_length(0.0,self.maximum_force)
    }

    fn pursue(&mut self, target:Vec3, target_velocity:Vec3, position:Vec3)->Vec3 {
        let target_position = target.clone() + target_velocity;
        self.seek(target_position,position)
    }

    fn evade(&mut self, target:Vec3, target_velocity:Vec3, position:Vec3)->Vec3 {
        -self.pursue(target,target_velocity,position)
    }

    fn arrive(&mut self, target:Vec3, position:Vec3)->Vec3{
        let slow_down = 8.0;
        //steering force = desired velocity - velocity
        let desired_velocity = (target - position).normalize()*self.maximum_speed;
        let mut steering_force = desired_velocity - self.velocity;
        if steering_force.length()<slow_down{
            steering_force = steering_force.normalize()*(steering_force.length()/slow_down)
        }
        steering_force.clamp_length(0.0,self.maximum_force)
    }
}

#[derive(Component)]
struct Target {}

#[derive(Component)]
struct Seek {}

#[derive(Component)]
struct Flee {}

#[derive(Component)]
struct Random {}

#[derive(Component)]
struct Pursue {}

#[derive(Component)]
struct Evade {}

#[derive(Component)]
struct Arrive {}

#[derive(Component)]
struct Align {
    weight:f32
}

#[derive(Component)]
struct Separate {
    weight:f32
}

#[derive(Component)]
struct Cohesion {
    weight:f32
}


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor{
            width: WIDTH,
            height: HEIGHT,
            title:"Example 5 Group".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(Mover(Vec::new()))
        .add_plugins(DefaultPlugins)
        .add_plugin(OrbitCameraPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_startup_system(spawn_limit_cube)
        .add_system(update_seek.before(moving))
        .add_system(update_flee.before(moving))
        .add_system(update_random.before(moving))
        .add_system(update_pursue.before(moving))
        .add_system(update_evade.before(moving))
        .add_system(update_arrive.before(moving))
        .add_system(update_align.before(moving))
        .add_system(update_separate.before(moving))
        .add_system(update_cohesion.before(moving))
        .add_system(moving)
        .run();
}

fn spawn_scene(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut mover: ResMut<Mover>
){
    //light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, MAX_LIMIT, 0.0),
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
    //vehicle
    for _ in 0..NUM_MOVEABLE{
        let mut rng = rand::thread_rng();
        let velocity = Vec3::new(
            rng.gen_range(-4.0..4.0),
            rng.gen_range(-4.0..4.0),
            rng.gen_range(-4.0..4.0)
        );
        let entity = commands.spawn_bundle(SceneBundle {
            scene: asset_server.load("models/cone_blue.glb#Scene0"),
            transform: Transform {
                translation: rnd_position(),
                ..default()
            },
            ..Default::default()
        })
            .insert(Moveable {
                velocity: velocity,
                ..default() })
            .insert(Align {
                weight: 1.0
            })
            .insert(Separate{
                weight: 1.0
            })
            .insert(Cohesion{
                weight: 1.0
            })
            .id();

        mover.0.push(entity.id());

    }
    //target
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 0.5,
            sectors: 32,
            stacks: 32
        })),
        material: materials.add(Color::YELLOW.into()),
        transform: Transform::from_translation(Vec3::ZERO),
        ..default()
    })
        .insert(Moveable{..default()})
        .insert(Random{})
        .insert(Target{});
}

fn rnd_position()->Vec3 {
    let mut rng = rand::thread_rng();
    Vec3::new(
        rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT),
        rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT),
        rng.gen_range(MIN_LIMIT + 1.0..MAX_LIMIT)
    )
}

fn spawn_camera(
    mut commands:Commands
){
    commands.spawn_bundle(Camera3dBundle{
        ..default()
    })
        .insert(OrbitCamera{
            distance : 56.0,
            ..default()
        });
}

fn update_seek(
    mut query: Query<(&mut Transform, &mut Moveable, &Seek),Without<Target>>,
    query_target: Query<&Transform,With<Target>>
){
    let target = query_target.single();

    for (transform, mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.seek(target.translation,transform.translation);
        moveable.force += new_force;
    }
}

fn update_flee(
    mut query: Query<(&mut Transform, &mut Moveable, &Flee),Without<Target>>,
    query_target: Query<&Transform,With<Target>>
){
    let target = query_target.single();

    for (transform, mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.flee(target.translation,transform.translation);
        moveable.force += new_force;
    }
}

fn update_random(
    mut query: Query<(&mut Moveable, &Random)>
){
    for (mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.random();
        moveable.force += new_force;
    }
}

fn update_pursue(
    mut query: Query<(&mut Transform, &mut Moveable, &Pursue),Without<Target>>,
    query_target: Query<(&Transform,&Moveable),With<Target>>
){
    let (target_transform,target) = query_target.single();

    for (transform, mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.pursue(target_transform.translation,
                                                         target.velocity,
                                                         transform.translation);
        moveable.force += new_force;
    }
}

fn update_evade(
    mut query: Query<(&mut Transform, &mut Moveable, &Evade),Without<Target>>,
    query_target: Query<(&Transform,&Moveable),With<Target>>
){
    let (target_transform,target) = query_target.single();

    for (transform, mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.evade(target_transform.translation,
                                                         target.velocity,
                                                         transform.translation);
        moveable.force += new_force;
    }
}

fn update_arrive(
    mut query: Query<(&mut Transform, &mut Moveable, &Arrive),Without<Target>>,
    query_target: Query<&Transform,With<Target>>
){
    let target = query_target.single();

    for (transform, mut moveable, _) in &mut query {
        let new_force = moveable.force + moveable.arrive(target.translation,transform.translation);
        moveable.force += new_force;
    }
}

fn update_align(
    mover: Res<Mover>,
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Align)>
) {
    let neighbor_distance = 8.0;

    for i in 0..NUM_MOVEABLE {

        let mover_id = mover.0[i as usize];
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for j in 0..NUM_MOVEABLE {

            let other_mover_id = mover.0[j as usize];

            if mover_id == other_mover_id {
                continue;
            }

            let first_position:Vec3;
            {
                let en = Entity::from_raw(mover_id);
                let (_,_,transform_1,_) = query.get_mut(en).unwrap();
                first_position = transform_1.translation.clone();
            }
            let second_position:Vec3;
            let second_velocity:Vec3;
            {
                let en2 = Entity::from_raw(other_mover_id);
                let (_,moveable_2,transform_2,_)= query.get_mut(en2).unwrap();
                second_position = transform_2.translation.clone();
                second_velocity = moveable_2.velocity.clone();
            }
            if first_position.distance(second_position) < neighbor_distance {
                count += 1;
                sum += second_velocity;
            }
        }
        if count > 0{
            let en = Entity::from_raw(mover_id);
            let (_, mut moveable, _, align) = query.get_mut(en).unwrap();
            sum = sum * (1.0/ count as f32);
            sum = sum.normalize() * moveable.maximum_speed;
            let new_force = moveable.force + align.weight * (sum-moveable.velocity).clamp_length(0.0, moveable.maximum_force);
            moveable.force = new_force;
        }
    }
}

fn update_separate(
    mover: Res<Mover>,
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Separate)>
) {
    let desired_separation  = 2.0;

    for i in 0..NUM_MOVEABLE {

        let mover_id = mover.0[i as usize];
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for j in 0..NUM_MOVEABLE {

            let other_mover_id = mover.0[j as usize];

            if mover_id == other_mover_id {
                continue;
            }

            let first_position:Vec3;
            {
                let en = Entity::from_raw(mover_id);
                let (_,_,transform_1,_) = query.get_mut(en).unwrap();
                first_position = transform_1.translation.clone();
            }
            let second_position:Vec3;
            {
                let en2 = Entity::from_raw(other_mover_id);
                let (_,_,transform_2,_)= query.get_mut(en2).unwrap();
                second_position = transform_2.translation.clone();
            }
            let dist = first_position.distance(second_position);
            if dist < desired_separation {

                let mut diff = first_position - second_position;
                diff = diff.normalize();
                diff = diff * 1.0/dist;
                count += 1;
                sum += diff;
            }
        }
        if count > 0{
            let en = Entity::from_raw(mover_id);
            let (_, mut moveable, _, separate) = query.get_mut(en).unwrap();
            sum = sum * (1.0/ count as f32);
            sum = sum.normalize() * moveable.maximum_speed;
            let new_force = moveable.force + separate.weight * (sum-moveable.velocity).clamp_length(0.0, moveable.maximum_force);
            moveable.force = new_force;
        }
    }
}

fn update_cohesion(
    mover: Res<Mover>,
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Cohesion)>
) {
    let neighbor_distance = 8.0;

    for i in 0..NUM_MOVEABLE {

        let mover_id = mover.0[i as usize];
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for j in 0..NUM_MOVEABLE {

            let other_mover_id = mover.0[j as usize];

            if mover_id == other_mover_id {
                continue;
            }

            let first_position:Vec3;
            {
                let en = Entity::from_raw(mover_id);
                let (_,_,transform_1,_) = query.get_mut(en).unwrap();
                first_position = transform_1.translation.clone();
            }
            let second_position:Vec3;
            {
                let en2 = Entity::from_raw(other_mover_id);
                let (_,_,transform_2,_)= query.get_mut(en2).unwrap();
                second_position = transform_2.translation.clone();
            }
            let dist = first_position.distance(second_position);
            if dist < neighbor_distance {
                count += 1;
                sum += second_position;
            }
        }
        if count > 0{
            let en = Entity::from_raw(mover_id);
            let (_, mut moveable, transform, cohesion) = query.get_mut(en).unwrap();
            sum = sum * (1.0/ count as f32);

            let new_force = moveable.force + moveable.seek(sum,transform.translation) * cohesion.weight;
            moveable.force = new_force;
        }
    }
}

const EDGE_LIMIT:bool=true;

fn moving(
    time:Res<Time>,
    mut query: Query<(&mut Transform, &mut Moveable)>,
){

    for (mut transform, mut moveable) in &mut query {
        let force = moveable.force.clone();
        moveable.apply_force(force);
        moveable.force *= 0.0;

        moveable.velocity = moveable.velocity + moveable.acceleration * time.delta_seconds();

        if EDGE_LIMIT {
            if transform.translation.x > MAX_LIMIT || transform.translation.x < MIN_LIMIT {
                moveable.velocity.x = moveable.velocity.x * -1.0;
            }

            if transform.translation.y > MAX_LIMIT || transform.translation.y < MIN_LIMIT {
                moveable.velocity.y = moveable.velocity.y * -1.0;
            }

            if transform.translation.z > MAX_LIMIT || transform.translation.z < MIN_LIMIT {
                moveable.velocity.z = moveable.velocity.z * -1.0;
            }
        }

        transform.translation = transform.translation + moveable.velocity * time.delta_seconds();

        moveable.acceleration *= 0.0;

        let t = transform.translation.clone();
        transform.look_at(moveable.velocity+t,Vec3::Y);

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