use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn enter_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = true;
}

pub fn exit_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = false;
}
