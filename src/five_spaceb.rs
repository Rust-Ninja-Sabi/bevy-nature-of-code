use bevy::prelude::*;

use orbitcamera::{OrbitCameraPlugin,OrbitCamera};
use skybox::SkyboxPlugin;

mod orbitcamera;
mod skybox;

use rand::Rng;

const HEIGHT: f32 = 640.0;
const WIDTH: f32 = 960.0;

const MAX_LIMIT: f32 = 32.0;
const MIN_LIMIT: f32 = -32.0;

const NUM_MOVEABLE: u32 = 156;
struct Mover(Vec<u32>);

const MAX_SPEED:f32=32.0;
const MAX_FORCE:f32=16.0;

struct AabBox {
    min:Vec3,
    max:Vec3
}

impl AabBox {
    fn new(origin:Vec3, x_length:f32, y_length:f32, z_lenght:f32, distance:f32)->Self{
        AabBox{
            min: Vec3::new(
                origin.x-x_length/2.0-distance,
                origin.y-y_length/2.0-distance,
                origin.z-z_lenght/2.0-distance
            ),
            max:Vec3::new(
                origin.x+x_length/2.0+distance,
                origin.y+y_length/2.0+distance,
                origin.z+z_lenght/2.0+distance
            )
        }
    }
}

struct Ray{
    origin:Vec3,
    direction:Vec3
}

impl Ray {
    fn new(orign:Vec3, direction:Vec3)->Self{
        Ray {
            origin:orign,
            direction:direction
        }
    }

    fn intersect_box(&self, aabb:&AabBox)->Option<(Vec3,Vec3)>{
        let mut tmin = (aabb.min.x - self.origin.x) / self.direction.x;
        let mut tmax = (aabb.max.x - self.origin.x) / self.direction.x;

        if tmin > tmax {
            let k = tmin;
            tmin = tmax;
            tmax = k;
        }

        let mut tymin = (aabb.min.y - self.origin.y) / self.direction.y;
        let mut tymax = (aabb.max.y - self.origin.y) / self.direction.y;

        if tymin > tymax {
            let k = tymin;
            tymin = tymax;
            tymax = k;
        }

        if tmin > tymax || tymin > tmax {
            return Option::None
        };

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (aabb.min.z - self.origin.z) / self.direction.z;
        let mut tzmax = (aabb.max.z - self.origin.z) / self.direction.z;

        if tzmin > tzmax {
            let k = tzmin;
            tzmin = tzmax;
            tzmax = k;
        }

        if tmin > tzmax || tzmin > tmax {
            return Option::None;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }

        if tzmax < tmax {
            tmax = tzmax;
        }

        Option::Some((Vec3::new(tmin,tymin,tzmin),Vec3::new(tmax,tymax,tzmax)))
    }
}

#[derive(Component)]
struct SpawnLaser {
    cooldown:f32
}

#[derive(Component)]
struct Laser{
    time:f32
}

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
struct Seek {
    weight:f32,
    target:Vec3
}

#[derive(Component)]
struct Flee {}

#[derive(Component)]
struct Random {
    weight:f32
}

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

#[derive(Component)]
struct Collision {
    weight:f32,
    aabb:AabBox,
    position:Vec3
}

#[derive(Clone,PartialEq)]
enum TeamType{
    Blue,
    Pink
}

#[derive(Component)]
struct Team {
    value:TeamType
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
        .add_plugin(SkyboxPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_scene)
        .add_system(update_seek.before(moving))
        //.add_system(update_flee.before(moving))
        .add_system(update_random.before(moving))
        //.add_system(update_pursue.before(moving))
        //.add_system(update_evade.before(moving))
        //.add_system(update_arrive.before(moving))
        .add_system(update_align.before(moving))
        .add_system(update_separate.before(moving))
        .add_system(update_cohesion.before(moving))
        .add_system(update_collision.before(moving))
        .add_system(spawn_laser)
        .add_system(move_laser)
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
            illuminance: 50000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 80.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI/2.0),
            ..default()
        },
        ..default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    //cruiser
    let cruiser_position = vec![Vec3::ZERO,
                                           Vec3::new(0.0, 0.0,60.0)];
    let teams = vec![TeamType::Blue, TeamType::Pink];
    let fighters = vec!["bricks/fighter_blue.glb#Scene0","bricks/fighter_pink.glb#Scene0"];

    for i in 0..2 {
        commands.spawn_bundle(SceneBundle {
            scene: asset_server.load("bricks/cruiser.glb#Scene0"),
            transform:Transform {
                translation: cruiser_position[i].clone()-Vec3::new(-15.0,0.0,0.0),
                scale: Vec3::new(0.15,0.15,0.15),
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                ..default()
            },
            ..Default::default()
        });

        //vehicle
        for _ in 0..NUM_MOVEABLE/2 {
            let mut rng = rand::thread_rng();
            let velocity = Vec3::new(
                rng.gen_range(-4.0..4.0),
                rng.gen_range(-4.0..4.0),
                rng.gen_range(-4.0..4.0)
            );
            let entity = commands.spawn_bundle(SceneBundle {
                scene: asset_server.load(fighters[i]),
                transform: Transform {
                    translation: rnd_position(),
                    scale: Vec3::new(0.02,0.02,0.02),
                    ..default()
                },
                ..Default::default()
            })
                .insert(Moveable {
                    velocity: velocity,
                    ..default()
                })
                .insert(Align {
                    weight: 1.0
                })
                .insert(Separate {
                    weight: 4.0
                })
                .insert(Cohesion {
                    weight: 0.5
                })
                .insert(Random{
                    weight:0.1
                })
                .insert(Collision{
                    weight:7.0,
                    position: cruiser_position[i],
                    aabb: AabBox::new(cruiser_position[i],70.0,8.0,25.0,4.0)
                })
                .insert(Seek{
                    weight:5.0,
                    target: cruiser_position[i]
                })
                .insert(Team{
                    value:teams[i].clone()
                })
                .insert(SpawnLaser{
                    cooldown:rng.gen_range(0.0..=COOLDOWN)
                })
                .id();

            mover.0.push(entity.id());
        }
    }

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
    mut query: Query<(&mut Transform, &mut Moveable, &Seek)>
){
    for (transform, mut moveable, seek) in &mut query {
        let new_force = moveable.force + moveable.seek(seek.target,transform.translation) * seek.weight;
        moveable.force = new_force;
    }
}

//fn update_flee(
//    mut query: Query<(&mut Transform, &mut Moveable, &Flee),Without<Target>>,
//    query_target: Query<&Transform,With<Target>>
//){
//    let target = query_target.single();
//
//    for (transform, mut moveable, _) in &mut query {
//        let new_force = moveable.force + moveable.flee(target.translation,transform.translation);
//        moveable.force += new_force;
//    }
//}

fn update_random(
    mut query: Query<(&mut Moveable, &Random)>
){
    for (mut moveable, random) in &mut query {
        let new_force = moveable.force + moveable.random() * random.weight;
        moveable.force = new_force;
    }
}

//fn update_pursue(
//    mut query: Query<(&mut Transform, &mut Moveable, &Pursue),Without<Target>>,
//    query_target: Query<(&Transform,&Moveable),With<Target>>
//){
//    let (target_transform,target) = query_target.single();
//
//    for (transform, mut moveable, _) in &mut query {
//        let new_force = moveable.force + moveable.pursue(target_transform.translation,
//                                                         target.velocity,
//                                                         transform.translation);
//        moveable.force += new_force;
//    }
//}

//fn update_evade(
//    mut query: Query<(&mut Transform, &mut Moveable, &Evade),Without<Target>>,
//    query_target: Query<(&Transform,&Moveable),With<Target>>
//){
//    let (target_transform,target) = query_target.single();
//
//    for (transform, mut moveable, _) in &mut query {
//        let new_force = moveable.force + moveable.evade(target_transform.translation,
//                                                         target.velocity,
//                                                         transform.translation);
//        moveable.force += new_force;
//    }
//}

//fn update_arrive(
//    mut query: Query<(&mut Transform, &mut Moveable, &Arrive),Without<Target>>,
//    query_target: Query<&Transform,With<Target>>
//){
//    let target = query_target.single();
//
//    for (transform, mut moveable, _) in &mut query {
//        let new_force = moveable.force + moveable.arrive(target.translation,transform.translation);
//        moveable.force += new_force;
//    }
//}

fn update_align(
    mover: Res<Mover>,
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Align, &Team)>
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
            let first_team:TeamType;
            {
                let en = Entity::from_raw(mover_id);
                let (_,_,transform_1,_,team_1) = query.get_mut(en).unwrap();
                first_position = transform_1.translation.clone();
                first_team = team_1.value.clone()
            }
            let second_position:Vec3;
            let second_velocity:Vec3;
            let second_team:TeamType;
            {
                let en2 = Entity::from_raw(other_mover_id);
                let (_,moveable_2,transform_2,_,team_2)= query.get_mut(en2).unwrap();
                second_position = transform_2.translation.clone();
                second_velocity = moveable_2.velocity.clone();
                second_team = team_2.value.clone();
            }
            if first_position.distance(second_position) < neighbor_distance && first_team == second_team {
                count += 1;
                sum += second_velocity;
            }
        }
        if count > 0{
            let en = Entity::from_raw(mover_id);
            let (_, mut moveable, _, align,_) = query.get_mut(en).unwrap();
            sum = sum * (1.0/ count as f32);
            sum = sum.normalize() * moveable.maximum_speed;
            let new_force = moveable.force + align.weight * (sum-moveable.velocity).clamp_length(0.0, moveable.maximum_force);
            moveable.force = new_force;
        }
    }
}

const DESIRED_SEPARATION:f32 = 8.0;

fn update_separate(
    mover: Res<Mover>,
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Separate)>
) {
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
            if dist < DESIRED_SEPARATION {

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
    mut query: Query<(Entity, &mut Moveable, &Transform, &mut Cohesion, &Team)>
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
            let first_team:TeamType;
            {
                let en = Entity::from_raw(mover_id);
                let (_,_,transform_1,_,team_1) = query.get_mut(en).unwrap();
                first_position = transform_1.translation.clone();
                first_team = team_1.value.clone();
            }
            let second_position:Vec3;
            let second_team:TeamType;
            {
                let en2 = Entity::from_raw(other_mover_id);
                let (_,_,transform_2,_,team_2)= query.get_mut(en2).unwrap();
                second_position = transform_2.translation.clone();
                second_team = team_2.value.clone();
            }
            let dist = first_position.distance(second_position);
            if dist < neighbor_distance && first_team == second_team {
                count += 1;
                sum += second_position;
            }
        }
        if count > 0{
            let en = Entity::from_raw(mover_id);
            let (_, mut moveable, transform, cohesion,_) = query.get_mut(en).unwrap();
            sum = sum * (1.0/ count as f32);

            let new_force = moveable.force + moveable.seek(sum,transform.translation) * cohesion.weight;
            moveable.force = new_force;
        }
    }
}

fn update_collision(
    mut query: Query<(&mut Moveable, &Transform, &mut Collision)>
) {

    for (mut moveable,transform,collision) in query.iter_mut() {
        let ray = Ray::new(transform.translation, transform.forward());

        match ray.intersect_box(&collision.aabb) {
            Some((result_1,result_2))=>{

                let dist_1 = result_1.distance(transform.translation);
                let dist_2 = result_1.distance(transform.translation);

                let result = if dist_1 < dist_2{
                    result_1
                }else {
                    result_2
                };

                let dir_to_center = (collision.position.clone() - transform.translation).normalize();
                let dir_to_collision = (result.clone() - transform.translation).normalize();
                let steering_direction = (dir_to_collision - dir_to_center).normalize();

                let new_force = moveable.force + steering_direction * collision.weight * moveable.maximum_force;
                moveable.force = new_force;
            }
            _=>{}
        }

    }
}

const EDGE_LIMIT:bool=false;

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


const COOLDOWN:f32=4.0;
const LASER_TIME:f32=0.2;

fn spawn_laser(
    mut commands: Commands,
    time:Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Transform,&mut SpawnLaser)>
)
{
    for (transform, mut spawnlaser) in query.iter_mut(){

        if spawnlaser.cooldown <= 0.0 {

            spawnlaser.cooldown = COOLDOWN;

                let z_length = 3.9;
                let position = transform.translation.clone() + transform.forward() * z_length/2.0;

                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, z_length))),
                    material: materials.add(StandardMaterial {
                        base_color: Color::LIME_GREEN,
                        emissive: Color::LIME_GREEN,
                        ..Default::default()
                    }),
                    transform: Transform {
                        translation: position,
                        rotation: transform.rotation.clone(),
                        scale: Vec3::new(1.0, 1.0, 1.0),
                        ..default()
                    },
                    ..Default::default()
                })
                    .insert(Name::new("Laser"))
                    .insert(Laser{time:LASER_TIME});
            } else {
            spawnlaser.cooldown -= time.delta_seconds();
        }
    }
}

const SPEED_LASER: f32 = 15.0;

fn move_laser(
    mut commands: Commands,
    time:Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Laser)>,
){
    for (entity, mut transform, mut laser) in query.iter_mut() {
            let translation_change = transform.forward() * SPEED_LASER * time.delta_seconds();
            transform.translation += translation_change;
        laser.time -= time.delta_seconds();
        if laser.time < 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}