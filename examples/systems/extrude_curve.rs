use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_curvo::prelude::NurbsSurfaceMesh;
use bevy_mod_raycast::prelude::*;
use bevy_normal_material::material::NormalMaterial;
use curvo::prelude::{AdaptiveTessellationOptions, NurbsSurface, Transformable};
use nalgebra::{Point3, Translation3, Vector3};

use crate::{AppState, ExtrudeCurve, ProfileCurve};

pub fn update_extrude_curve(
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_ray: Res<CursorRay>,
    curves: Query<&ProfileCurve>,
    mut gizmos: Gizmos,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normal_materials: ResMut<'_, Assets<NormalMaterial>>,
    extrusion: Query<&ExtrudeCurve>,
) {
    if let Some(cursor_ray) = **cursor_ray {
        let n = extrusion.iter().count();
        if n == 0 {
            if let Some(d) = cursor_ray.intersect_plane(Vec3::ZERO, Plane3d::default()) {
                let p = cursor_ray.get_point(d);
                let closest = curves
                    .iter()
                    .map(|c| {
                        let pt = Point3::from(p);
                        let closest = c.0.closest_point(&pt);
                        let distance = (closest - pt).norm();
                        (distance, c)
                    })
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
                if let Some((_, closest)) = closest {
                    let tess = closest.0.tessellate(None);
                    gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        commands.spawn(ExtrudeCurve(closest.0.clone()));
                    }
                }
            }
        } else {
            let target = extrusion.single();
            let tess = target.0.tessellate(None);
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);

            if let Some(d) =
                cursor_ray.intersect_plane(Vec3::ZERO, Plane3d::new(cursor_ray.direction.into()))
            {
                let p = cursor_ray.get_point(d);
                let elevation = p.y.abs();
                let offset = target
                    .0
                    .transformed(&Translation3::new(0., elevation, 0.).into());
                let tess = offset.tessellate(None);
                gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::YELLOW);
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    next_state.set(AppState::Idle);
                    let extruded = NurbsSurface::extrude(&target.0, Vector3::y() * elevation);
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

pub fn exit_extrude_curve(mut commands: Commands, extrusion: Query<Entity, With<ExtrudeCurve>>) {
    extrusion.iter().for_each(|e| {
        commands.entity(e).despawn();
    });
}
