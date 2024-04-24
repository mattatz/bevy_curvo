use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_mod_picking::{focus::PickingInteraction, selection::PickSelection, PickableBundle};
use bevy_mod_raycast::CursorRay;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_transform_gizmo::{GizmoTransformable, TransformGizmoBundle};
use curvo::prelude::Transformable;
use nalgebra::Point3;

use crate::{spawn_curve, spawn_interp_curve, AppState, LineMaterial, ProfileCurve, SelectedCurve};

pub fn enter_transform_curve(mut commands: Commands, mut curves: Query<(Entity, &SelectedCurve)>) {
    curves.iter().for_each(|(e, _)| {
        commands.entity(e).log_components();
        commands
            .entity(e)
            .insert(GizmoTransformable)
            .insert(PickableBundle {
                selection: PickSelection { is_selected: true },
                ..Default::default()
            });
    });
}

pub fn update_transform_curve(
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    curves: Query<(&SelectedCurve, &Transform, &PickSelection)>,
    mut gizmos: Gizmos,
) {
    curves.iter().for_each(|(curve, tr, pick)| {
        let mat = tr.compute_matrix();
        let curve = curve.0.transformed(&mat.into());
        let tess = curve.tessellate(Some(1e-4));
        gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
        if !pick.is_selected {
            next_state.set(AppState::Idle);
        }
    });
}

pub fn exit_transform_curve(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    curve: Query<(Entity, &SelectedCurve, &Transform)>,
) {
    curve.iter().for_each(|(e, curve, tr)| {
        commands.entity(e).despawn();

        let mat = tr.compute_matrix();
        if Mat4::IDENTITY != mat {
            spawn_curve(
                curve.0.clone(),
                &mut commands,
                &mut meshes,
                &mut line_materials,
                Some(tr.clone()),
                Color::ALICE_BLUE,
            );
        }
    });
}
