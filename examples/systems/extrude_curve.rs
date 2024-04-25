use bevy::prelude::*;
use bevy_curvo::prelude::NurbsSurfaceMesh;
use bevy_mod_raycast::prelude::*;
use bevy_normal_material::material::NormalMaterial;
use curvo::prelude::{AdaptiveTessellationOptions, NurbsSurface, Transformable};
use nalgebra::{Translation3, Vector3};

use crate::{find_closest_curve, AppState, ExtrudeCurve, ProfileCurve};

pub fn update_extrude_curve(
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    curves: Query<(&ProfileCurve, &Transform)>,
    mut gizmos: Gizmos,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normal_materials: ResMut<'_, Assets<NormalMaterial>>,
    extrusion: Query<&ExtrudeCurve>,
) {
    if let Some(cursor_ray) = **cursor_ray {
        let n = extrusion.iter().count();
        if n == 0 {
            let closest = find_closest_curve(cursor_ray, &curves.iter().collect::<Vec<_>>(), 0.5);
            if let Some((prof, _, c)) = closest {
                let tess = c.tessellate(None);
                gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    commands.spawn(ExtrudeCurve(prof.id()));
                }
            }
        } else {
            let target = extrusion.single();
            let curve = curves
                .iter()
                .find(|(prof, _)| prof.id() == target.0)
                .unwrap();
            let transformed = curve
                .0
                .curve()
                .transformed(&curve.1.compute_matrix().into());
            let tess = transformed.tessellate(None);
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);

            if let Some(d) =
                cursor_ray.intersect_plane(Vec3::ZERO, Plane3d::new(cursor_ray.direction.into()))
            {
                let p = cursor_ray.get_point(d);
                let elevation = p.y.abs();
                let offset = transformed.transformed(&Translation3::new(0., elevation, 0.).into());
                let tess = offset.tessellate(None);
                gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    next_state.set(AppState::Idle);
                    let extruded = NurbsSurface::extrude(&transformed, Vector3::y() * elevation);
                    let tess = extruded.tessellate(Some(AdaptiveTessellationOptions {
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
                        .insert(Name::new("extrusion"));
                }
            }
        }
    }
}

pub fn exit_extrude_curve(
    mut commands: Commands,
    extrusion: Query<(Entity, &ExtrudeCurve)>,
    curves: Query<(Entity, &ProfileCurve)>,
) {
    extrusion.iter().for_each(|(e, extrude)| {
        commands.entity(e).despawn();
        let found = curves.iter().find(|(_, prof)| prof.id() == extrude.0);
        if let Some(found) = found {
            commands.entity(found.0).despawn();
        }
    });
}
