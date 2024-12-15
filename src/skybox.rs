use bevy::prelude::*;
use std::f32::consts::PI;

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App){
        app
            .add_systems(Startup, setup_skybox);
    }
}

const SIZE:f32=640.0;

fn setup_skybox(
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
){
    let images = vec!["images/skybox_front.png",
                                 "images/skybox_left.png",
                                 "images/skybox_right.png",
                                 "images/skybox_back.png",
                                 "images/skybox_down.png",
                                 "images/skybox_up.png"];
    let distance = SIZE/2.0;
    let translations = vec![Vec3::new(0.0, 0.0, -distance),
                                       Vec3::new(distance, 0.0, 0.0),
                                       Vec3::new(-distance, 0.0, 0.0),
                                       Vec3::new(0.0, 0.0, distance),
                                       Vec3::new(0.0, -distance, 0.0),
                                       Vec3::new(0.0, distance, 0.0),];
    let rotations =vec![ Quat::from_rotation_x(0.0),
                                    Quat::from_euler(EulerRot::XYZ,0.0,-PI/2.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,0.0,PI/2.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,PI,0.0,-PI),
                                    Quat::from_euler(EulerRot::XYZ,-PI/2.0,0.0,0.0),
                                    Quat::from_euler(EulerRot::XYZ,PI/2.0,0.0,0.0)];

    for i in 0..images.len() {
        //sky
        let store_texture_handle = asset_server.load(images[i]);
        let store_aspect = 1.0;

        let store_quad_width = SIZE;
        let store_quad_handle = meshes.add(Mesh::from(Rectangle::new(
            store_quad_width,
            store_quad_width * store_aspect,
        )));

        let store_material_handle = materials.add(StandardMaterial {
            base_color_texture: Some(store_texture_handle.clone()),
            unlit: true,
            ..Default::default()
        });

        commands.spawn((
            Mesh3d(store_quad_handle.clone()),
            MeshMaterial3d(store_material_handle),
            Transform {
                translation: translations[i],
                rotation: rotations[i],
                ..Default::default()
            }
        ));
    }
}