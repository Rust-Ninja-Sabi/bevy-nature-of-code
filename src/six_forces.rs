use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct ThirdPersonTarget;

#[derive(Component)]
struct ThirdPersonCamera{
    ideal_offset:Vec3,
    ideal_lookat:Vec3,
    current_lookat:Vec3,
    lookat_aviabel:bool,
    follow:f32
}
impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            ideal_offset: Vec3::new(0.0,2.0,6.0),
            ideal_lookat: Vec3::new(0.0,0.0,-4.0),
            current_lookat: Vec3::new(0.0,0.0,0.0),
            lookat_aviabel: false,
            follow: 1.2
        }
    }
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Cheese;


fn main() {
    App::new()

        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "six forces".to_string(),
                resolution: WindowResolution::new(800.0,  600.0),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugins(RapierDebugRenderPlugin::default()) //for debugging
        .add_systems(Startup,setup)
        .add_systems(Update,(input_user,
                             collision,
                             move_camera))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::from_xyz(0.0,1.0,0.0).looking_at(Vec3::new(0.,0.,-4.), Vec3::Y)
    ))
        .insert(ThirdPersonCamera{..Default::default()});

    //light
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

    //walls
    let mut children_list:Vec<Entity> = Vec::new();
    let wall1 = commands
        .spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.4,0.4,12.0)))),
            MeshMaterial3d(materials.add( StandardMaterial{
                base_color: Color::srgb(0.5, 0.5, 0.5),
                double_sided: true,
                ..Default::default()
            })),
            Transform {
                translation: Vec3::new(-1.7, 0.2, 0.0),
                rotation: Quat::from_rotation_x(0.0),
                ..Default::default()
            }
        ))
        .insert(Collider::cuboid(0.4/2.0, 0.4/2.0, 12.0/2.0))
        .id();
    children_list.push(wall1);
    let wall2 = commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.4,0.4,12.0)))),
        MeshMaterial3d(materials.add( StandardMaterial{
            base_color: Color::srgb(0.5, 0.5, 0.5),
            double_sided: true,
            ..Default::default()
        })),
        Transform {
            translation: Vec3::new(1.7, 0.2, 0.0),
            rotation: Quat::from_rotation_x(0.0),
            ..Default::default()
        }
    )).insert(Collider::cuboid(0.4/2.0, 0.4/2.0, 12.0/2.0))
        .id();

    children_list.push(wall2);
    let wall3 = commands
        .spawn((
            Mesh3d( meshes.add(Mesh::from(Cuboid::new(3.8,0.4,0.4)))),
            MeshMaterial3d( materials.add( StandardMaterial{
                base_color: Color::srgb(0.5, 0.5, 0.5),
                double_sided: true,
                ..Default::default()
            })),
            Transform {
                translation: Vec3::new(0.0, 0.2, -6.0),
                rotation: Quat::from_rotation_x(0.0),
                ..Default::default()
            }
        ))
        .insert(Collider::cuboid(3.4/2.0, 0.4/2.0, 0.4/2.0))
        .id();
    children_list.push(wall3);

    //platform
    commands
        .spawn((
            Mesh3d( meshes.add(Mesh::from(Cuboid::new(3.0,0.1,12.0)))),
            MeshMaterial3d( materials.add( StandardMaterial{
                base_color: Color::srgb(1.0, 0.8, 0.6),
                double_sided: true,
                ..Default::default()
            })),
            Transform {
                translation: Vec3::new(0.0, -2.0, -11.0),
                rotation: Quat::from_rotation_x(0.0),
                ..Default::default()
            }
        ))
        .add_children (&children_list)
        .insert(RigidBody::Fixed)
        .insert(Sleeping::disabled())
        .insert(Collider::cuboid(3.0/2.0, 0.1/2.0, 12.0/2.0));

    //cheese
    let cheese_position = Vec3::new(0.0, -1.0, -9.0);

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: YELLOW.into(),
            ..Default::default()
        })),
        Transform::from_translation(cheese_position)
    ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Collider::cylinder(0.15, 0.3))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Cheese{});
    
    //ball
    commands
        .spawn((
            Mesh3d( meshes.add(Mesh::from(Sphere{
                radius:0.5
            }))),
            MeshMaterial3d( materials.add( StandardMaterial{
                base_color: Color::srgb(0.0, 0.0, 1.0),
                ..Default::default()
            })),
            Transform {
                translation: Vec3::new(0.0, -1.0, -6.0),
                rotation: Quat::from_rotation_x(0.0),
                ..Default::default()
            }
        ))
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Collider::ball(0.5))
        .insert(ExternalForce {
            ..Default::default()
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ThirdPersonTarget{})
        .insert(Ball{});

}

const SPEED:f32= 1.0;

fn input_user(
    keyboard_input:Res<ButtonInput<KeyCode>>,
    mut query_forces: Query<&mut ExternalForce>,
){

    let x = if keyboard_input.pressed(KeyCode::ArrowLeft) {
        -SPEED
    } else if keyboard_input.pressed(KeyCode::ArrowRight) {
        SPEED
    } else {
        0.0
    };

    let z = if keyboard_input.pressed(KeyCode::ArrowUp) {
        -SPEED
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        SPEED
    } else {
        0.0
    };

    if x != 0.0 || z != 0.0 {
        for mut ext_force in query_forces.iter_mut() {
            ext_force.force = Vec3::new(x,0.0, z);
        }
    }
}


fn collision(
    mut collision_events: EventReader<CollisionEvent>,
    query_ball: Query<Entity, With<Ball>>,
    query_cheese: Query<(Entity, &Transform), With<Cheese>>,
    mut commands: Commands
){
    let entity_ball = query_ball.single();
    for e in collision_events.read(){
        match e {
            CollisionEvent::Started(e1,e2,_) => {
                if e1 == &entity_ball || e2 == &entity_ball {
                    for (entity_cheese, cheese_transform) in query_cheese.iter(){
                        if e1 == &entity_cheese || e2 == &entity_cheese {
                            commands.entity(entity_cheese).despawn_recursive();
                        }
                    }
                }
            }
            CollisionEvent::Stopped(_,_,_)=> {}
        }
    }
}

fn move_camera(
    time:Res<Time>,
    mut query_camera: Query<(&mut Transform, &mut ThirdPersonCamera), Without<ThirdPersonTarget>>,
    query_target: Query<&Transform, With<ThirdPersonTarget>>
){
    let (mut camera_transform, mut thridperson) = query_camera.single_mut();
    let target_transform = query_target.single();
    let t = thridperson.follow * time.delta_secs();

    let mut offset = thridperson.ideal_offset.clone();
    offset += target_transform.translation;
    offset = camera_transform.translation.lerp(offset,t);

    let mut lookat = thridperson.ideal_lookat.clone();
    lookat+= target_transform.translation;
    if thridperson.lookat_aviabel {
        lookat = thridperson.current_lookat.lerp(lookat, t);
    } else{
        thridperson.lookat_aviabel = true;
    }

    thridperson.current_lookat = lookat;

    let transform = Transform::from_translation(offset).looking_at(lookat, Vec3::Y);
    camera_transform.translation = transform.translation;
    camera_transform.rotation = transform.rotation
}