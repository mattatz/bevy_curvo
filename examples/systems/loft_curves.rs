use bevy::prelude::*;
use bevy_curvo::prelude::NurbsSurfaceMesh;
use bevy_mod_raycast::prelude::*;
use bevy_normal_material::material::NormalMaterial;
use curvo::prelude::{AdaptiveTessellationOptions, NurbsSurface, Transformable};

use crate::{find_closest_curve, AppState, ProfileCurve, Setting};

pub fn update_loft_curves(
    mut next_state: ResMut<NextState<AppState>>,
    mut setting: ResMut<Setting>,
    _commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    cursor_ray: Res<CursorRay>,
    curves: Query<(Entity, &ProfileCurve, &Transform)>,
    mut gizmos: Gizmos,
) {
    if let Some(cursor_ray) = **cursor_ray {
        let (others, selected): (Vec<_>, Vec<_>) = curves
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .partition(|(e, _, _)| !setting.loft_curves_target.contains(e));
        let closest = find_closest_curve(
            cursor_ray,
            &others.iter().map(|(_, c, t)| (*c, *t)).collect::<Vec<_>>(),
            0.5,
        );
        if let Some((prof, _tr, c)) = closest {
            let tess = c.tessellate(None);
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
            if mouse_button_input.just_pressed(MouseButton::Left) {
                let found = curves
                    .iter()
                    .find(|(_e, other, _)| other.id() == prof.id())
                    .map(|(e, _, _)| e);
                if let Some(found) = found {
                    setting.loft_curves_target.push(found);
                }
            }
        }

        selected.iter().for_each(|(_, c, t)| {
            let tess = c
                .curve()
                .transformed(&t.compute_matrix().into())
                .tessellate(None);
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW_GREEN);
        });
    }

    if key_input.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Idle);
    }
}

pub fn exit_loft_curves(
    mut commands: Commands,
    mut setting: ResMut<Setting>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normal_materials: ResMut<'_, Assets<NormalMaterial>>,
    curves: Query<(Entity, &ProfileCurve, &Transform)>,
) {
    let target = setting
        .loft_curves_target
        .iter()
        .filter_map(|e| {
            curves
                .iter()
                .find(|(e2, _, _)| *e2 == *e)
                .map(|(_, c, t)| (c, t))
        })
        .collect::<Vec<_>>();
    if target.len() > 1 {
        let transformed = target
            .iter()
            .map(|(c, t)| c.curve().transformed(&t.compute_matrix().into()))
            .collect::<Vec<_>>();
        let surface = NurbsSurface::try_loft(&transformed, Some(3));
        if let Ok(lofted) = surface {
            let tess = lofted.tessellate(Some(AdaptiveTessellationOptions {
                norm_tolerance: 1e-2 * 2.5,
                ..Default::default()
            }));
            let mesh = NurbsSurfaceMesh::from(tess);
            commands
                .spawn(MaterialMeshBundle {
                    mesh: meshes.add(mesh),
                    material: normal_materials.add(NormalMaterial {
                        cull_mode: None,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert(Name::new("lofted"));
        }
    }

    setting.loft_curves_target.clear();
}
