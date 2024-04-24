mod common;
mod components;
mod materials;
mod systems;

use common::*;
use components::*;
use materials::*;
use systems::*;

use bevy::{core::Zeroable, prelude::*, window::close_on_esc};

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};

use bevy_mod_raycast::prelude::*;
use bevy_normal_material::{material::NormalMaterial, plugin::NormalMaterialPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_points::{plugin::PointsPlugin, prelude::PointsMaterial};

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
