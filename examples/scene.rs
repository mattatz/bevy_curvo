use std::cmp::Ordering;

use bevy::{
    core::Zeroable,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
    window::close_on_esc,
};
use bevy_curvo::prelude::NurbsSurfaceMesh;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};

use bevy_mod_raycast::prelude::*;
use bevy_normal_material::{material::NormalMaterial, plugin::NormalMaterialPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_points::{plugin::PointsPlugin, prelude::PointsMaterial};
use nalgebra::{Point3, Translation3, Vector3};

use curvo::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Idle,
    InterpolateCurve,
    ExtrudeCurve,
}

#[derive(Component, Default, Debug)]
struct InterpolateCurve {
    points: Vec<Vec3>,
}

#[derive(Component, Debug)]
struct ProfileCurve(NurbsCurve3D<f32>);

#[derive(Component, Debug)]
struct ExtrudeCurve(NurbsCurve3D<f32>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<LineMaterial>::default())
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(PointsPlugin)
        .add_plugins(NormalMaterialPlugin)
        .add_plugins(DefaultRaycastingPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(AppPlugin)
        .run();
}

struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, close_on_esc)
            .insert_state(AppState::Idle)
            .add_systems(Update, (update_ui, visualize_geometry))
            .add_systems(OnEnter(AppState::Idle), (enter_idle,))
            .add_systems(OnExit(AppState::Idle), (exit_idle,))
            .add_systems(
                OnEnter(AppState::InterpolateCurve),
                (enter_interpolate_curve,),
            )
            .add_systems(
                Update,
                update_interpolate_curve.run_if(in_state(AppState::InterpolateCurve)),
            )
            .add_systems(
                OnExit(AppState::InterpolateCurve),
                (exit_interpolate_curve,),
            )
            .add_systems(
                Update,
                update_extrude_curve.run_if(in_state(AppState::ExtrudeCurve)),
            )
            .add_systems(OnExit(AppState::ExtrudeCurve), (exit_extrude_curve,));
    }
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _line_materials: ResMut<Assets<LineMaterial>>,
    _points_materials: ResMut<Assets<PointsMaterial>>,
    _normal_materials: ResMut<'_, Assets<NormalMaterial>>,
) {
    let center = Vec3::zeroed();
    let camera = Camera3dBundle {
        /*
        projection: OrthographicProjection {
            5.,
            near: 1e-1,
            far: 1e4,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..Default::default()
        }
        .into(),
        */
        transform: Transform::from_translation(center + Vec3::new(10., 10., 10.))
            .looking_at(center, Vec3::Y),
        ..Default::default()
    };
    commands.spawn((camera, PanOrbitCamera::default()));
    commands.spawn(InfiniteGridBundle::default());
}

fn update_ui(
    mut contexts: EguiContexts,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    curves: Query<&ProfileCurve>,
) {
    let has_profile_curves = curves.iter().count() > 0;
    let current_state = current_state.get();

    egui::Window::new("bevy_curvo example")
        .collapsible(false)
        .drag_to_scroll(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("mode");
            ui.group(|group| {
                if group
                    .add_enabled(
                        true,
                        egui::Button::new("idle").selected(matches!(current_state, AppState::Idle)),
                    )
                    .clicked()
                {
                    next_state.set(AppState::Idle);
                }
                if group
                    .add_enabled(
                        true,
                        egui::Button::new("interpolate curve")
                            .selected(matches!(current_state, AppState::InterpolateCurve)),
                    )
                    .clicked()
                {
                    next_state.set(AppState::InterpolateCurve);
                }
                if group
                    .add_enabled(
                        has_profile_curves,
                        egui::Button::new("extrude curve")
                            .selected(matches!(current_state, AppState::ExtrudeCurve)),
                    )
                    .clicked()
                {
                    next_state.set(AppState::ExtrudeCurve);
                }
            });
        });
}

fn visualize_geometry(_current_state: Res<State<AppState>>, _curves: Query<&ProfileCurve>) {}

fn enter_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = true;
}

fn exit_idle(_commands: Commands, mut camera: Query<&mut PanOrbitCamera>) {
    camera.single_mut().enabled = false;
}

fn enter_interpolate_curve(mut commands: Commands) {
    commands.spawn((InterpolateCurve::default(),));
}

fn update_interpolate_curve(
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

fn exit_interpolate_curve(
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

fn update_extrude_curve(
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

fn exit_extrude_curve(mut commands: Commands, extrusion: Query<Entity, With<ExtrudeCurve>>) {
    extrusion.iter().for_each(|e| {
        commands.entity(e).despawn();
    });
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}
