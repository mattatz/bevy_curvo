use bevy::prelude::*;
use bevy_mod_picking::{selection::PickSelection, PickableBundle};

use bevy_transform_gizmo::{GizmoTransformable, RotationOriginOffset};
use curvo::prelude::Transformable;

use crate::{spawn_curve, AppState, LineMaterial, SelectedCurve};

pub fn enter_transform_curve(
    mut commands: Commands,
    curves: Query<(Entity, &SelectedCurve, &Transform)>,
) {
    curves.iter().for_each(|(e, s, _t)| {
        let pt =
            s.0.control_points_iter()
                .map(|pt| Vec3::new(pt.x, pt.y, pt.z))
                .sum::<Vec3>()
                / (s.0.control_points().len() as f32);
        commands.entity(e).log_components();
        commands
            .entity(e)
            .insert(GizmoTransformable)
            .insert(PickableBundle {
                selection: PickSelection { is_selected: true },
                ..Default::default()
            })
            .insert(RotationOriginOffset(pt));
    });
}

pub fn update_transform_curve(
    mut next_state: ResMut<NextState<AppState>>,
    _commands: Commands,
    _mouse_button_input: Res<ButtonInput<MouseButton>>,
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
                Some(*tr),
                Color::ALICE_BLUE,
            );
        }
    });
}
