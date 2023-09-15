use bevy::prelude::*;
use bevy_aabb_instancing::VertexPullingRenderPlugin;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

mod block;
mod block_model;
mod optimizer;
mod ui;

use block_model::{BlockModelDB, BlockModelResource};
use optimizer::OptimizeParams;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use ui::init_optimizer;

fn main() {
    App::new()
        .add_state::<AppState>()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ui::FileDragAndDropInputResource::default())
        .insert_resource(ui::FileDragAndDropResource::default())
        .insert_resource(BlockModelResource::default())
        .insert_resource(BlockModelDB::default())
        .insert_resource(OptimizeParams::default())
        .add_plugins(DefaultPlugins)
        .add_plugins((
            VertexPullingRenderPlugin { outlines: true },
            LookTransformPlugin,
            OrbitCameraPlugin::default(),
            EguiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Startup, configure_visuals_system)
        .add_systems(Update, (ui::ui_system, ui::detect_file_drop))
        .add_systems(Update, ui::file_drop.run_if(in_state(AppState::FileDrop)))
        .add_systems(
            Update,
            init_optimizer.run_if(in_state(AppState::OptimizeInit)),
        )
        .run();
}

#[derive(Default, Debug, Hash, PartialEq, Eq, Clone, States)]
pub enum AppState {
    #[default]
    Running,
    FileDrop,
    OptimizeInit,
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //camera
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_rotate_sensitivity: Vec2::splat(0.08),
                mouse_translate_sensitivity: Vec2::splat(1000.0),
                mouse_wheel_zoom_sensitivity: 0.2,
                smoothing_weight: 0.0,
                enabled: true,
                pixels_per_line: 53.0,
            },
            Vec3::new(14424.87, 108851.7, 3597.5),
            Vec3::new(14424.87, 108851.7, 3197.5),
            Vec3::Y,
        ));

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 5.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
}
