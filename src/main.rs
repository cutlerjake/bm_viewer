use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb};
use bevy_aabb_instancing::{Cuboid, Cuboids, VertexPullingRenderPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

mod block;
mod block_model;
mod optimizer;
mod ui;

use block_model::{BlockModelDB, BlockModelResource};
use optimizer::OptimizeParams;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransform, LookTransformPlugin,
};
use ui::{init_optimizer, ViewAll};

fn main() {
    App::new()
        .add_state::<AppState>()
        .add_event::<ViewAll>()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ui::FileResource::default())
        .insert_resource(ui::FileInputResource::default())
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
        .add_systems(Update, ui::file_drop.run_if(in_state(AppState::FileInput)))
        .add_systems(Update, view_all)
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
    FileInput,
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

fn view_all(
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &mut Transform)>,
    mut view_all_event: EventReader<ViewAll>,
    bounding_boxes: Query<&Aabb, With<Cuboids>>,
) {
    if view_all_event.iter().next().is_none() {
        return;
    }
    //println!("HELLO");
    // Can only control one camera at a time.
    let (mut transform, mut scene_transform) =
        if let Some((_, transform, scene_transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
            (transform, scene_transform)
        } else {
            return;
        };

    //compute bounding box
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let mut boxes_flag = false;
    for bounding_box in bounding_boxes.iter() {
        boxes_flag = true;
        min = min.min((bounding_box.min()).into());
        max = max.max((bounding_box.max()).into());
    }

    if !boxes_flag {
        return;
    }
    let scene_box = Aabb::from_min_max(min, max);

    let length = scene_box.half_extents.max_element();
    let fov_angle = f32::to_radians(45.0); //proj.fov;
    let dist = length / (fov_angle / 2.0).tan();

    *transform = LookTransform::new(
        (scene_box.center + Vec3A::new(0.0, 0.0, dist)).into(),
        scene_box.center.into(),
        Vec3::Y,
    )
    .into();
}
