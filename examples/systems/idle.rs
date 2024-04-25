use bevy::prelude::*;
use bevy_mod_raycast::CursorRay;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::{find_closest_curve, AppState, ProfileCurve, SelectedCurve};

pub fn enter_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = true;
}

pub fn update_idle(
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    curves: Query<(Entity, &ProfileCurve, &Transform)>,
    mut gizmos: Gizmos,
) {
    if let Some(cursor_ray) = **cursor_ray {
        let closest = find_closest_curve(
            cursor_ray,
            &curves.iter().map(|(_, p, t)| (p, t)).collect::<Vec<_>>(),
            0.5,
        );
        if let Some((prof, _tr, transformed)) = closest {
            let tess = transformed.tessellate(Some(1e-4));
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
            if mouse_button_input.just_pressed(MouseButton::Left) {
                let found = curves.iter().find(|c| c.1.id() == prof.id()).map(|c| c.0);
                if let Some(found) = found {
                    commands.entity(found).insert(SelectedCurve);
                    next_state.set(AppState::Select);
                }
            }
        }
    }
}

pub fn exit_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = false;
}
