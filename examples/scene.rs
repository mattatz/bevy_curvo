mod common;
mod components;
mod materials;
mod systems;

use bevy_transform_gizmo::{GizmoPickSource, TransformGizmoPlugin};
use common::*;
use components::*;
use materials::*;
use nalgebra::Point3;
use systems::*;

use bevy::{core::Zeroable, prelude::*};

use bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};

use bevy_mod_picking::prelude::*;
use bevy_mod_raycast::prelude::*;
use bevy_normal_material::{material::NormalMaterial, plugin::NormalMaterialPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_points::{plugin::PointsPlugin, prelude::PointsMaterial};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InfiniteGridPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(PointsPlugin)
        .add_plugins(NormalMaterialPlugin)
        .add_plugins(LineMaterialPlugin)
        .add_plugins(DefaultRaycastingPlugin)
        .add_plugins((DefaultPickingPlugins, TransformGizmoPlugin::default()))
        .add_plugins(EguiPlugin)
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(AppPlugin)
        .run();
}

struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Setting::default())
            .insert_state(AppState::Idle)
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                (absorb_egui_inputs,)
                    .after(bevy_egui::systems::process_input_system)
                    .before(bevy_egui::EguiSet::BeginFrame),
            )
            .add_systems(Update, (detect_mode_reset, update_ui, visualize_geometry))
            .add_systems(OnEnter(AppState::Idle), (enter_idle,))
            .add_systems(Update, update_idle.run_if(in_state(AppState::Idle)))
            .add_systems(OnExit(AppState::Idle), (exit_idle,))
            .add_systems(OnEnter(AppState::Select), (enter_transform_curve,))
            .add_systems(
                Update,
                update_transform_curve.run_if(in_state(AppState::Select)),
            )
            .add_systems(OnExit(AppState::Select), (exit_transform_curve,))
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
            .add_systems(OnExit(AppState::ExtrudeCurve), (exit_extrude_curve,))
            .add_systems(
                Update,
                update_loft_curves.run_if(in_state(AppState::LoftCurves)),
            )
            .add_systems(OnExit(AppState::LoftCurves), (exit_loft_curves,));
    }
}

fn absorb_egui_inputs(mut mouse: ResMut<ButtonInput<MouseButton>>, mut contexts: EguiContexts) {
    if contexts.ctx_mut().is_pointer_over_area() {
        mouse.reset_all();
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    // mut query: Query<(Entity, &Camera)>,
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
    commands.spawn((
        camera,
        PanOrbitCamera::default(),
        GizmoPickSource::default(),
    ));

    commands.spawn(InfiniteGridBundle::default());

    let points = vec![
        Point3::new(-1., 0., -1.),
        Point3::new(1., 0., -1.),
        Point3::new(1., 0., 0.),
        Point3::new(-1., 0., 0.),
        Point3::new(-1., 0., 1.),
        Point3::new(1., 0., 1.),
    ];
    spawn_interp_curve(
        &mut commands,
        &mut meshes,
        &mut line_materials,
        Color::ALICE_BLUE,
        &points,
        3,
    );
}

fn detect_mode_reset(
    mut next_state: ResMut<NextState<AppState>>,
    key_button_input: Res<ButtonInput<KeyCode>>,
) {
    if key_button_input.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Idle);
    }
}

fn update_ui(
    mut contexts: EguiContexts,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    curves: Query<&ProfileCurve>,
) {
    let number_of_curves = curves.iter().count();
    let has_profile_curves = number_of_curves > 0;
    let current_state = current_state.get();

    egui::Window::new("bevy_curvo example")
        .collapsible(false)
        .drag_to_scroll(false)
        .default_width(420.)
        .min_width(420.)
        .max_width(420.)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("# of curves: {}", number_of_curves));
            ui.spacing();

            ui.heading("mode");
            ui.group(|group| {
                group.horizontal(|group| {
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
                if group
                    .add_enabled(
                        has_profile_curves,
                        egui::Button::new("loft curves")
                            .selected(matches!(current_state, AppState::LoftCurves)),
                    )
                    .clicked()
                {
                    next_state.set(AppState::LoftCurves);
                }

                });
            });

            ui.spacing();

            match current_state {
                AppState::Idle => {
                    ui.label("Select modeling mode or select a curve to transform.");
                }
                AppState::Select => {
                    ui.heading("selection mode");
                    ui.label("transform curve.");
                    if ui.button("cancel").clicked() {
                        next_state.set(AppState::Idle);
                    }
                }
                AppState::InterpolateCurve => {
                    ui.heading("interpolate curve mode");
                    ui.label("click to add a point & interpolate curve with the points.\npress enter to confirm.");
                    if ui.button("confirm").clicked() {
                        next_state.set(AppState::Idle);
                    }
                }
                AppState::ExtrudeCurve => {
                    ui.heading("extrude curve mode");
                    ui.label("select a curve & extrude it to create a surface.");
                    if ui.button("cancel").clicked() {
                        next_state.set(AppState::Idle);
                    }
                }
                AppState::LoftCurves => {
                    ui.heading("loft curves mode");
                    ui.label("select curves to loft.\npress enter to confirm.");
                    if ui.button("confirm").clicked() {
                        next_state.set(AppState::Idle);
                    }
                }
                _ => {}
            };
        });
}

fn visualize_geometry(_current_state: Res<State<AppState>>, _curves: Query<&ProfileCurve>) {}
