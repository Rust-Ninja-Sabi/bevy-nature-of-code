//Most of the code is from
// https://github.com/iMplode-nZ/bevy-orbit-controls
// Thanks.
use bevy::prelude::*;
use bevy_egui::EguiContexts;

use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseScrollUnit::{Line, Pixel};
use bevy::input::mouse::MouseWheel;

use std::ops::RangeInclusive;

const LINE_TO_PIXEL_RATIO: f32 = 0.1;


pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App){
        app
            .add_event::<CameraEvents>()
            .add_systems( Update,(emit_motion_events,
                                  mouse_motion,
                                  emit_zoom_events,
                                  zoom,
                                  update_transform));
    }
}

#[derive(Event)]
pub enum CameraEvents {
    Orbit(Vec2),
    Pan(Vec2),
    Zoom(f32),
}

#[derive(Component)]
pub struct OrbitCamera {
    pub x: f32,
    pub y: f32,
    pub pitch_range: RangeInclusive<f32>,
    pub distance: f32,
    pub center: Vec3,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub rotate_button: MouseButton,
    pub pan_button: MouseButton,
    pub enabled: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            x: 0.0,
            y: std::f32::consts::FRAC_PI_2,
            pitch_range: 0.01..=3.13,
            distance: 5.0,
            center: Vec3::ZERO,
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            rotate_button: MouseButton::Left,
            pan_button: MouseButton::Right,
            enabled: true,
        }
    }
}

impl OrbitCamera {
    pub fn new(dist: f32, center: Vec3) -> OrbitCamera {
        OrbitCamera {
            distance: dist,
            center,
            ..Self::default()
        }
    }
}


fn emit_motion_events(
    mut events: EventWriter<CameraEvents>,
    mut motion_evr: EventReader<MouseMotion>,
    mut egui_contexts:EguiContexts,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<&OrbitCamera>
){
    let mut delta = Vec2::ZERO;
    let mut send_event = true;

    for event in motion_evr.read(){
        delta += event.delta;
    }

    let context = egui_contexts.ctx_mut();
    if context.wants_pointer_input() {
        send_event = false
    }

    for camera in query.iter_mut() {
        if camera.enabled {
            if mouse_button_input.pressed(camera.rotate_button){
                if send_event {
                    events.send(CameraEvents::Orbit(delta));
                }
            }
            if mouse_button_input.pressed(camera.pan_button){
                if send_event {
                    events.send(CameraEvents::Pan(delta));
                }
            }
        }
    }
}

fn mouse_motion(
    time: Res<Time>,
    mut events: EventReader<CameraEvents>,
    mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>
) {
    for (mut camera, transform, _) in query.iter_mut(){
        if !camera.enabled {
            continue;
        }

        for event in events.read() {
            match event {
                CameraEvents::Orbit(delta) => {
                    camera.x -= delta.x * camera.rotate_sensitivity * time.delta_secs();
                    camera.y -= delta.y * camera.rotate_sensitivity * time.delta_secs();
                    camera.y = camera
                        .y
                        .max(*camera.pitch_range.start())
                        .min(*camera.pitch_range.end())
                }
                CameraEvents::Pan(delta) => {
                    let right_dir = transform.rotation * -Vec3::X;
                    let up_dir = transform.rotation * Vec3::Y;
                    let pan_vector = (delta.x * right_dir+ delta.y * up_dir)
                        * camera.pan_sensitivity
                        * time.delta_secs();
                    camera.center += pan_vector;
                }
                _ => {}
            }
        }
    }
}

fn emit_zoom_events(
    mut events: EventWriter<CameraEvents>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&OrbitCamera>
){
    let mut total = 0.0;
    for event in mouse_wheel_events.read() {
        total += event.y
            * match event.unit {
            Line => 1.0,
            Pixel => LINE_TO_PIXEL_RATIO
        };
    }

    if total != 0.0 {
        for camera in query.iter_mut() {
            if camera.enabled {
                events.send(CameraEvents::Zoom(total));
            }
        }
    }
}

fn zoom (
    mut query: Query<&mut OrbitCamera, With<Camera>>,
    mut events: EventReader<CameraEvents>
){
    for mut camera in query.iter_mut(){
        for event in events.read() {
            if camera.enabled{
                if let CameraEvents::Zoom(distance) = event {
                    camera.distance *= camera.zoom_sensitivity.powf(*distance);
                }
            }
        }
    }
}
fn update_transform(
    mut query: Query<(&OrbitCamera, &mut Transform), (Changed<OrbitCamera>, With<Camera>)>,
) {
    for (camera, mut transform) in query.iter_mut() {
        if camera.enabled {
            let rot = Quat::from_axis_angle(Vec3::Y, camera.x)
                * Quat::from_axis_angle(-Vec3::X, camera.y);
            transform.translation = (rot * Vec3::Y) * camera.distance + camera.center;
            transform.look_at(camera.center, Vec3::Y);
        }
    }
}

