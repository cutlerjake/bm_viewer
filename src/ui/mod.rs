use bevy::{prelude::*, utils::HashMap};
use bevy_aabb_instancing::{CuboidMaterial, CuboidMaterialMap, COLOR_MODE_RGB};
use bevy_egui::{
    egui::{self, Widget},
    EguiContexts,
};
use itertools::izip;
use polars::prelude::{CsvReader, SerReader};

use crate::{
    block_model::{BlockModel, BlockModelDB, BlockModelResource},
    optimizer::OptimizeParams,
    AppState,
};

#[derive(Event)]
pub struct ViewAll;

pub fn ui_system(
    mut contexts: EguiContexts,
    block_models: Res<BlockModelDB>,
    mut selected: Local<String>,
    mut checked: Local<HashMap<String, (Vec<bool>, Vec<Vec<Entity>>)>>,
    mut commands: Commands,
    mut material_map: ResMut<CuboidMaterialMap>,
    mut next_state: ResMut<NextState<AppState>>,
    mut file_dnd: ResMut<FileInputResource>,
    mut event_writer: EventWriter<ViewAll>,
) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        file_dnd.path_buf = path.clone();
                        file_dnd.window = None;
                        next_state.set(AppState::FileInput);
                    }
                }
            });
        });
    });

    egui::TopBottomPanel::bottom("Bottom panel").show(ctx, |ui| {
        if ui.button("View All").clicked() {
            event_writer.send(ViewAll);
        }
    });

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        egui::ComboBox::from_label("Blockmodels")
            .selected_text(selected.clone())
            .show_ui(ui, |ui| {
                for (name, _) in block_models.block_models.iter() {
                    ui.selectable_value(&mut *selected, name.clone(), name.clone());
                }
            });

        ui.heading("Columns");
        ui.separator();
        if *selected != "" {
            let (ref mut checked, ref mut entities) =
                checked.entry(selected.clone()).or_insert_with(|| {
                    (
                        block_models
                            .block_models
                            .get(&*selected)
                            .unwrap()
                            .columns
                            .iter()
                            .map(|_| false)
                            .collect::<Vec<_>>(),
                        vec![
                            Vec::new();
                            block_models
                                .block_models
                                .get(&*selected)
                                .unwrap()
                                .columns
                                .len()
                        ],
                    )
                });

            for (col, mut check, ents) in izip!(
                block_models
                    .block_models
                    .get(&*selected)
                    .unwrap()
                    .columns
                    .iter(),
                checked.iter_mut(),
                entities.iter_mut()
            ) {
                //ui.label(col);
                if ui.checkbox(&mut check, col).changed() {
                    if *check == true {
                        //draw bm
                        let cuboids_abbb = block_models
                            .block_models
                            .get(&*selected)
                            .unwrap()
                            .aabb_instances(col.clone(), colorgrad::turbo(), 22500);

                        let material_id = material_map.push(CuboidMaterial {
                            color_mode: COLOR_MODE_RGB,
                            ..default()
                        });
                        for (cuboids, aabb) in cuboids_abbb.into_iter() {
                            let mut ent = commands.spawn(SpatialBundle::default());
                            ent.insert((cuboids, aabb, material_id));

                            ents.push(ent.id());
                        }
                    } else {
                        //erase bm
                        ents.drain(..).for_each(|ent| {
                            commands.entity(ent).despawn_recursive();
                        });
                    }
                }
            }

            // ui.separator();

            // if ui.button("Optimize").clicked() {
            //     next_state.set(AppState::OptimizeInit);
            // }
        }
    });
}

#[derive(Resource, Default)]
pub struct FileInputResource {
    pub path_buf: std::path::PathBuf,
    pub window: Option<Entity>,
}

pub fn detect_file_drop(
    mut next_state: ResMut<NextState<AppState>>,
    mut dnd_evr: EventReader<FileDragAndDrop>,
    mut file_dnd: ResMut<FileInputResource>,
) {
    for ev in dnd_evr.iter() {
        if let FileDragAndDrop::DroppedFile { path_buf, window } = ev {
            file_dnd.path_buf = path_buf.clone();
            file_dnd.window = Some(window.clone());
            next_state.set(AppState::FileInput);
        }
    }
}

#[derive(Resource, Default)]
pub struct FileResource {
    name: String,
    x_col: String,
    y_col: String,
    z_col: String,
    x_size_col: String,
    y_size_col: String,
    z_size_col: String,
}

pub fn file_drop(
    mut contexts: EguiContexts,
    dnd_data: Res<FileInputResource>,
    mut menu_data: ResMut<FileResource>,
    mut next_state: ResMut<NextState<AppState>>,
    mut bm_res: ResMut<BlockModelResource>,
    mut bm_db: ResMut<BlockModelDB>,
) {
    let ctx = contexts.ctx_mut();

    let window = egui::Window::new("Blockmodel File");
    window.show(ctx, |ui| {
        ui.label("Blockmodel File");
        ui.label(dnd_data.path_buf.to_str().unwrap_or("No file selected"));

        egui::Grid::new("some_unique_id").show(ui, |ui| {
            ui.label("block model name");
            ui.end_row();

            ui.text_edit_singleline(&mut menu_data.name);
            ui.end_row();
            ui.label("X column");
            ui.label("Y column");
            ui.label("Z column");
            ui.end_row();

            ui.text_edit_singleline(&mut menu_data.x_col);
            ui.text_edit_singleline(&mut menu_data.y_col);
            ui.text_edit_singleline(&mut menu_data.z_col);
            ui.end_row();

            ui.label("X size column");
            ui.label("Y size column");
            ui.label("Z size column");
            ui.end_row();

            ui.text_edit_singleline(&mut menu_data.x_size_col);
            ui.text_edit_singleline(&mut menu_data.y_size_col);
            ui.text_edit_singleline(&mut menu_data.z_size_col);
            ui.end_row();

            ui.label(""); // spacing
            if ui.button("Load").clicked() {
                let df = CsvReader::from_path(dnd_data.path_buf.clone())
                    .expect("Unable to read file")
                    .has_header(true)
                    .finish()
                    .expect("Unable to creat df");

                let bm = BlockModel::new(
                    menu_data.name.clone(),
                    df,
                    menu_data.x_col.clone(),
                    menu_data.y_col.clone(),
                    menu_data.z_col.clone(),
                    menu_data.x_size_col.clone(),
                    menu_data.y_size_col.clone(),
                    menu_data.z_size_col.clone(),
                );

                *bm_res = BlockModelResource {
                    block_model: Some(bm.clone()),
                };

                bm_db.block_models.insert(menu_data.name.clone(), bm);

                next_state.set(AppState::Running);
            }

            if ui.button("Close").clicked() {
                next_state.set(AppState::Running);
            }
            ui.end_row();
        });
    });
}

pub fn init_optimizer(
    mut optimizer_init_data: ResMut<OptimizeParams>,
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut bm_res: ResMut<BlockModelResource>,
    mut bm_db: ResMut<BlockModelDB>,
) {
    let ctx = contexts.ctx_mut();
    let window = egui::Window::new("Optimization Parameters");
    window.show(ctx, |ui| {
        ui.label("Optimization Parameters");

        egui::Grid::new("some_unique_id").show(ui, |ui| {
            ui.label("Grade Column");
            ui.label("Tonnage Column");
            ui.label("Cutoff");
            ui.label("Min Life");
            ui.label("Processing Capacity");
            ui.label("Mining Rate");
            ui.end_row();

            ui.text_edit_singleline(&mut optimizer_init_data.grade_col);
            ui.text_edit_singleline(&mut optimizer_init_data.tonnage_col);
            ui.add(egui::DragValue::new(&mut optimizer_init_data.cutoff));
            ui.add(egui::DragValue::new(&mut optimizer_init_data.min_life));
            ui.add(egui::DragValue::new(&mut optimizer_init_data.proc_cap));
            ui.add(egui::DragValue::new(&mut optimizer_init_data.mining_rate));
            ui.end_row();

            ui.label(""); // spacing
            if ui.button("Optimize").clicked() {
                // let bm = bm_res.block_model.take().unwrap();
                // let bm = bm.optimize(
                //     optimizer_init_data.grade_col.clone(),
                //     optimizer_init_data.tonnage_col.clone(),
                //     optimizer_init_data.cutoff,
                //     optimizer_init_data.min_life,
                //     optimizer_init_data.proc_cap,
                //     optimizer_init_data.mining_rate,
                // );

                // *bm_res = BlockModelResource {
                //     block_model: Some(bm.clone()),
                // };

                // bm_db.block_models.insert(bm.name.clone(), bm);

                next_state.set(AppState::Running);
            }

            if ui.button("Close").clicked() {
                next_state.set(AppState::Running);
            }
            ui.end_row();
        });
    });
}

// #[derive(Default, Resource)]
// pub struct CurrentBlockModel {
//     bm: Option<BlockModel>,
// }

// #[derive(Default, Resource)]
// pub struct LoadedBlockModels {
//     bms: Vec<BlockModel>,
// }
