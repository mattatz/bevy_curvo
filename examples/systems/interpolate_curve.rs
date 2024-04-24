use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_mod_raycast::prelude::*;
use bevy_normal_material::material::NormalMaterial;
use curvo::prelude::NurbsCurve3D;
use nalgebra::Point3;

use crate::{AppState, InterpolateCurve, LineMaterial, ProfileCurve};

pub fn enter_interpolate_curve(mut commands: Commands) {
    commands.spawn((InterpolateCurve::default(),));
}

pub fn update_interpolate_curve(
    mut next_state: ResMut<NextState<AppState>>,
    _commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_button_input: Res<ButtonInput<KeyCode>>,
    mut curve: Query<&mut InterpolateCurve>,
    cursor_ray: Res<CursorRay>,
    _raycast: Raycast,
    mut gizmos: Gizmos,
) {
    if let Some(cursor_ray) = **cursor_ray {
        if let Some(d) = cursor_ray.intersect_plane(Vec3::ZERO, Plane3d::default()) {
            let p = cursor_ray.get_point(d);
            gizmos.circle(p, Direction3d::Y, 0.1, Color::WHITE);

            if mouse_button_input.just_pressed(MouseButton::Left) {
                let mut c = curve.single_mut();
                c.points.push(p);
            }
        }
    }

    if let Ok(c) = curve.get_single() {
        gizmos.linestrip(c.points.clone(), Color::WHITE);

        let n = c.points.len();
        if n >= 3 {
            let points: Vec<_> = c.points.iter().map(|p| Point3::from(*p)).collect();
            let interpolated =
                NurbsCurve3D::try_interpolate(&points, (n - 1).min(3), None, None).unwrap();
            let tess = interpolated.tessellate(None);
            gizmos.linestrip(tess.iter().map(|p| Vec3::from(*p)), Color::GRAY);
        }
    }

    if key_button_input.just_pressed(KeyCode::Enter)
        || key_button_input.just_pressed(KeyCode::Space)
        || key_button_input.just_pressed(KeyCode::Escape)
    {
        next_state.set(AppState::Idle);
    }
}

pub fn exit_interpolate_curve(
    mut commands: Commands,
    curve: Query<(Entity, &InterpolateCurve)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    _normal_materials: ResMut<'_, Assets<NormalMaterial>>,
) {
    curve.iter().for_each(|(e, c)| {
        commands.entity(e).despawn();

        let points: Vec<_> = c.points.iter().map(|p| Point3::from(*p)).collect();
        if points.len() > 3 {
            let degree = (points.len() - 1).min(3);
            let interpolated = NurbsCurve3D::try_interpolate(&points, degree, None, None).unwrap();

            let mut line = Mesh::new(bevy::render::mesh::PrimitiveTopology::LineStrip, default());
            let line_vertices = interpolated
                .tessellate(Some(1e-3))
                .iter()
                .map(|p| [p.x, p.y, p.z])
                .collect();
            line.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x3(line_vertices),
            );
            commands.spawn((
                ProfileCurve(interpolated),
                MaterialMeshBundle {
                    mesh: meshes.add(line),
                    material: line_materials.add(LineMaterial {
                        color: Color::ALICE_BLUE,
                    }),
                    ..Default::default()
                },
            ));
        }
    });
}
