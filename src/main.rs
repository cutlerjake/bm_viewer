use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    math::Vec3A,
    prelude::*,
    render::{
        primitives::Aabb,
        render_resource::{Extent3d, TextureDimension},
        view::{window, RenderLayers},
    },
    sprite::Anchor,
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_aabb_instancing::{Cuboid, Cuboids, VertexPullingRenderPlugin};
use bevy_egui::{
    egui::{self, remap},
    EguiContexts, EguiPlugin,
};
use image::EncodableLayout;
use image::{ImageBuffer, Rgba};

mod block;
mod block_model;
mod optimizer;
mod ui;

use block_model::{BlockModelDB, BlockModelResource};
use colorgrad::Gradient;
use optimizer::OptimizeParams;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransform, LookTransformPlugin,
};
use ui::{init_optimizer, OccupiedScreenSpace, ViewAll};

fn main() {
    App::new()
        .add_state::<AppState>()
        .add_event::<ViewAll>()
        .add_event::<ColorBarSelectionEvent>()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ui::FileResource::default())
        .insert_resource(ui::FileInputResource::default())
        .insert_resource(BlockModelResource::default())
        .insert_resource(BlockModelDB::default())
        .insert_resource(OptimizeParams::default())
        .init_resource::<OccupiedScreenSpace>()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            VertexPullingRenderPlugin { outlines: true },
            LookTransformPlugin,
            OrbitCameraPlugin::default(),
            EguiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Startup, configure_visuals_system)
        .add_systems(Startup, (camera_2d))
        .add_systems(Update, show_color_bar)
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
    commands.spawn(Camera3dBundle::default()).insert((
        OrbitCameraBundle::new(
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
        ),
        RenderLayers::from_layers(&[0]),
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

fn camera_2d(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            camera: Camera {
                order: 2,
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::from_layers(&[1]),
    ));
}

#[derive(Event, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ColorBarSelectionEvent {
    grid: String,
    column: String,
}
fn show_color_bar(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    mut entities_to_delete: Local<Vec<Entity>>,
    mut color_bar_selection_event: EventReader<ColorBarSelectionEvent>,
    mut current_color_bar_selection: Local<HashMap<ColorBarSelectionEvent, Vec<Entity>>>,
    blockmodels: Res<BlockModelDB>,
) {
    let cb_event = color_bar_selection_event.iter().next();

    if let Some(event) = cb_event {
        //if entry already exists, despant entities
        if let Some(entities) = current_color_bar_selection.remove(event) {
            entities.iter().for_each(|entity| {
                commands.entity(*entity).despawn_recursive();
            });
            return;
        } else {
            current_color_bar_selection.insert(event.clone(), Vec::new());
        }
    }
    if !occupied_screen_space.is_changed() && cb_event.is_none() {
        return;
    }

    let window = windows.single();
    let gradient = colorgrad::turbo();
    let width = 500;
    let height = 50;
    let (dmin, dmax) = gradient.domain();
    let image_buffer = ImageBuffer::from_fn(width, height, |x, _| {
        let rgba = gradient.at(remap(x as f64, 0.0f64..=width as f64, dmin..=dmax));

        let col_f32 = Color::Rgba {
            red: rgba.r as f32,
            green: rgba.g as f32,
            blue: rgba.b as f32,
            alpha: rgba.a as f32,
        }
        .as_rgba_f32();

        Rgba(col_f32)
    });

    let extent = Extent3d {
        width: width as u32,
        height: height as u32,
        ..Default::default()
    };

    let bevy_image = Image::new(
        extent,
        TextureDimension::D2,
        image_buffer.as_bytes().to_vec(),
        bevy::render::render_resource::TextureFormat::Rgba32Float,
    );

    let im = images.add(bevy_image);

    //despawn current entities
    current_color_bar_selection
        .values_mut()
        .for_each(|mut entities| {
            entities.drain(..).for_each(|entity| {
                commands.entity(entity).despawn_recursive();
            });
        });

    // entities_to_delete.drain(..).for_each(|entity| {
    //     commands.entity(entity).despawn_recursive();
    // });

    // if current_color_bar_selection.is_none() {
    //     return;
    // }

    for (cb, mut entities) in current_color_bar_selection.iter_mut() {
        entities.push(
            commands
                .spawn((
                    SpriteBundle {
                        texture: im.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            0.0 + occupied_screen_space.left as f32 / 2.0
                                - occupied_screen_space.right as f32 / 2.0,
                            -window.height() / 2.0
                                + 2f32 * height as f32
                                + occupied_screen_space.bottom as f32 / 2.0
                                - occupied_screen_space.top as f32 / 2.0,
                            1.0,
                        )),

                        ..Default::default()
                    },
                    RenderLayers::layer(1),
                ))
                .id(),
        );

        let min: f32 = blockmodels
            .block_models
            .get(cb.grid.as_str())
            .unwrap()
            .df
            .column(cb.column.as_str())
            .unwrap()
            .min()
            .unwrap();

        let max: f32 = blockmodels
            .block_models
            .get(cb.grid.as_str())
            .unwrap()
            .df
            .column(cb.column.as_str())
            .unwrap()
            .max()
            .unwrap();

        for i in 0..=10 {
            let text = Text::from_section(
                format!("{:.3}", remap(i as f32, 0.0..=10.0, min..=max)),
                TextStyle {
                    ..Default::default()
                },
            );

            entities.push(
                commands
                    .spawn((
                        Text2dBundle {
                            text,
                            transform: Transform::from_translation(Vec3::new(
                                0.0 + occupied_screen_space.left as f32 / 2.0
                                    - occupied_screen_space.right as f32 / 2.0
                                    + i as f32 * width as f32 / 10.0
                                    - width as f32 / 2f32,
                                -window.height() / 2.0
                                    + height as f32
                                    + occupied_screen_space.bottom as f32 / 2.0
                                    - occupied_screen_space.top as f32 / 2.0,
                                1.0,
                            )),
                            ..Default::default()
                        },
                        RenderLayers::layer(1),
                    ))
                    .id(),
            );
        }

        let text = Text::from_section(
            format!("{}: {}", cb.grid, cb.column),
            TextStyle {
                ..Default::default()
            },
        );

        entities.push(
            commands
                .spawn((
                    Text2dBundle {
                        text,
                        transform: Transform::from_translation(Vec3::new(
                            0.0 + occupied_screen_space.left as f32 / 2.0
                                - occupied_screen_space.right as f32 / 2.0,
                            -window.height() / 2.0
                                + 3f32 * height as f32
                                + occupied_screen_space.bottom as f32 / 2.0
                                - occupied_screen_space.top as f32 / 2.0,
                            1.0,
                        )),
                        ..Default::default()
                    },
                    RenderLayers::layer(1),
                ))
                .id(),
        );
    }
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
