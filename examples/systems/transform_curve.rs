use bevy::prelude::*;
use bevy_mod_picking::{selection::PickSelection, PickableBundle};

use bevy_transform_gizmo::{GizmoTransformable, RotationOriginOffset};
use curvo::prelude::Transformable;

use crate::{AppState, LineMaterial, ProfileCurve, SelectedCurve};

pub fn enter_transform_curve(
    mut commands: Commands,
    curves: Query<(Entity, &ProfileCurve, &Transform), With<SelectedCurve>>,
) {
    curves.iter().for_each(|(e, prof, _t)| {
        let pt = prof
            .curve()
            .control_points_iter()
            .map(|pt| Vec3::new(pt.x, pt.y, pt.z))
            .sum::<Vec3>()
            / (prof.curve().control_points().len() as f32);
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
    curves: Query<(&ProfileCurve, &Transform, &PickSelection), With<SelectedCurve>>,
    mut gizmos: Gizmos,
) {
    curves.iter().for_each(|(curve, tr, pick)| {
        let mat = tr.compute_matrix();
        let curve = curve.curve().transformed(&mat.into());
        let tess = curve.tessellate(Some(1e-4));
        gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
        if !pick.is_selected {
            next_state.set(AppState::Idle);
        }
    });
}

pub fn exit_transform_curve(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _line_materials: ResMut<Assets<LineMaterial>>,
    curve: Query<Entity, With<SelectedCurve>>,
) {
    curve.iter().for_each(|e| {
        commands
            .entity(e)
            .remove::<SelectedCurve>()
            .remove::<GizmoTransformable>()
            .remove::<PickSelection>()
            .remove::<RotationOriginOffset>();
    });
}
