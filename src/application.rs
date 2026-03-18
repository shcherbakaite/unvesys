/*
 Main application

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use wildflower::Pattern;
use crate::egui::Button;
use crate::logic_commands::*;
use std::thread;
use std::path::PathBuf;
use crate::outline::ProjectOutline;
use egui::Label;
use egui::Sense;
use egui::CollapsingHeader;
use std::fs::File;
#[cfg(target_os = "windows")]
use winreg::enums::HKEY_CURRENT_USER;
use egui::RichText;
use std::sync::{Arc, Mutex};
//#[cfg(target_os = "windows")]
#[cfg(target_os = "windows")]
use winreg::*;
use std::io::prelude::*;
use std::io;
use egui::{Color32};
use ::egui::menu;
use std::cell::RefCell;
use std::time::{Duration, SystemTime};

use crate::vysis::*;
use crate::vysyslib::*;
use crate::harness_commands::*;
use crate::wire_list_grid::{flexible_table_to_data_table, wire_list_grid_ui};
use crate::flexible_table::{FlexibleTable, TableData, TableDataMut};
use crate::wirelist::{wirelist_to_flexible_table, connector_connections_to_flexible_table};
use crate::wire_list_xlsx_formatter::{color_map, flexible_table_to_xlsx_generic, WireListXlsxFormatter};
use crate::shchleuniger::{wirelist_to_schleuniger_ascii, SchleunigerASCIIConfig};
use crate::flexible_table::{flexible_table_to_labels_csv, flexible_table_to_csv_generic};
use xlsxwriter::Workbook;
use chrono::Local;

use sanitise_file_name::sanitise;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
use egui::WidgetText;

// ISSUE: https://github.com/bodil/smartstring/issues/7
// WORKAROUND: use format! in place of + operator to contacatenate strings


static BG_GRAPHIC: &str =
r"     








































      It appears you are trying to make a harness...


                                              ▄████▄
                                             ▐▌░░░░▐▌
                                          ▄▀▀█▀░░░░▐▌
                                          ▄░▐▄░░░░░▐▌▀▀▄
                                        ▐▀░▄▄░▀▌░▄▀▀░▀▄░▀
                                        ▐░▀██▀░▌▐░▄██▄░▌
                                         ▀▄░▄▄▀░▐░░▀▀░░▌
                                            █░░░░▀▄▄░▄▀
                                            █░█░░░░█░▐
                                            █░█░░░▐▌░█ 
                                            █░█░░░▐▌░█ 
                                            ▐▌▐▌░░░█░█
                                            ▐▌░█▄░▐▌░█
                                             █░░▀▀▀░░▐▌
                                             ▐▌░░░░░░█
                                              █▄░░░░▄█
                                               ▀████▀


";

static LOG_EXPIRATION: Duration = Duration::from_secs(5);

/// Status colors darkened for better visibility in Light mode
fn status_green() -> Color32 { Color32::from_rgb(0, 128, 0) }
fn status_yellow() -> Color32 { Color32::from_rgb(160, 120, 0) }
fn status_red() -> Color32 { Color32::from_rgb(180, 0, 0) }

struct WireListViewState {
    pub title: String,
    pub table: FlexibleTable,
    pub data_table: egui_data_table::DataTable<crate::flexible_table::FlexibleRow>,
    pub last_exported_path: Option<PathBuf>,
}

enum DockTab {
    Project,
    WireList(WireListViewState),
}

struct ApplicationState {
    project: Option<Project>, // VeSys Project                                                          // Opened document
    project_path: Option<PathBuf>,                                                                      // Path to project XMl for reloading
    library: Option<Library>, // VeSys Library                                                          // Loaded on start
    project_outline: Option<ProjectOutline>,   // Cached UI representation of the VeSys project         // UI representation
    output_dir: String,       // Output directory                                                       // UI storage
    log: RefCell<Vec::<(RichText, SystemTime, Option<Duration>, Option<PathBuf>)>>,  // status messages, optional path for "View" link
    filter: String,
    pending_wire_list: RefCell<Option<WireListViewState>>,
    dark_mode: bool,          // Theme: true = Dark, false = Light
}

struct AppTabViewer {
    state: Arc<Mutex<ApplicationState>>,
}

impl TabViewer for AppTabViewer {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            DockTab::Project => "Project".into(),
            DockTab::WireList(w) => w.title.as_str().into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::Project => {
                if let Ok(mut state) = self.state.lock() {
                    Application::filter_ui(ui, &mut state);
                    Application::project_view_ui(ui, &mut state);
                }
            }
            DockTab::WireList(wire_list) => {
                if let Ok(state) = self.state.lock() {
                    Application::wire_list_tab_ui(ui, wire_list, &state);
                }
            }
        }
    }
}

fn read_file(filename:&str) -> std::io::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn try_load_library_into_state(state: Arc<Mutex<ApplicationState>>, extra_paths: Vec<PathBuf>) {
    std::thread::spawn(move || {
        state.lock().unwrap().log(RichText::new("Loading library").color(status_yellow()), None, None);

        let mut candidates = extra_paths;
        if let Some(mut exe_path) = process_path::get_executable_path() {
            exe_path.set_file_name("Library.xml");
            if !candidates.iter().any(|p| p == &exe_path) {
                candidates.push(exe_path);
            }
        }

        let mut loaded = false;
        for path in candidates {
            match read_file(&path.display().to_string()) {
                Ok(library_xml) => match Library::new(&library_xml) {
                    Ok(library) => {
                        state.lock().unwrap().library = Some(library);
                        loaded = true;
                        break;
                    }
                    Err(_) => {
                        state.lock().unwrap().log(
                            RichText::new(format!("Failed to parse Library.xml at {}", path.display())).color(status_red()),
                            Some(LOG_EXPIRATION),
                            None,
                        );
                    }
                }
                Err(_) => {}
            }
        }

        if loaded {
            state.lock().unwrap().log(RichText::new("Library loaded").color(status_green()), Some(LOG_EXPIRATION), None);
        } else {
            state.lock().unwrap().log(
                RichText::new("Library.xml not found. Place it next to the executable or in the same directory as the project XML.").color(status_red()),
                Some(LOG_EXPIRATION),
                None,
            );
        }
    });
}

fn ui_hover_label_with_menu(ui: &mut egui::Ui, name: &str, context_menu_ui: impl FnOnce(&mut egui::Ui)) {
   let label = ui.add(Label::new(name)
          .selectable(false)
          .sense(Sense::hover()));
    label.context_menu(context_menu_ui);
    // Highlight on hover
    if label.hovered() {
        label.highlight();
    }
}

/// Renders the elements list (Devices, Splices, Connectors, Wires) with filter.
fn elements_list_ui<'a>(
    ui: &mut egui::Ui,
    connectivity: &Connectivity<'a>,
    filter: &Pattern<String>,
    state: &ApplicationState,
    context_title: &str,
) {
    let devices: Vec<_> = connectivity.dom.device.iter()
        .filter(|d| filter.matches(&d.name) || filter.matches(&d.id))
        .collect();
    let splices: Vec<_> = connectivity.dom.splice.iter()
        .filter(|s| filter.matches(&s.name))
        .collect();
    let connectors: Vec<_> = connectivity.dom.connector.iter()
        .filter(|c| filter.matches(&c.name))
        .collect();
    let wires: Vec<_> = connectivity.dom.wire.iter()
        .filter(|w| filter.matches(&w.name) || filter.matches(&w.id))
        .collect();

    if devices.is_empty() && splices.is_empty() && connectors.is_empty() && wires.is_empty() {
        ui.label("No elements match the filter.");
        return;
    }

    if egui::CollapsingHeader::new(format!("Devices ({})", devices.len()))
        .default_open(true)
        .show(ui, |ui| {
            for d in &devices {
                let label = if d.name.is_empty() { &d.id } else { &d.name };
                ui.label(format!("  {}", label));
            }
        }).body_response.is_none() {}

    if egui::CollapsingHeader::new(format!("Splices ({})", splices.len()))
        .default_open(true)
        .show(ui, |ui| {
            for s in &splices {
                ui.label(format!("  {}", s.name));
            }
        }).body_response.is_none() {}

    if egui::CollapsingHeader::new(format!("Connectors ({})", connectors.len()))
        .default_open(true)
        .show(ui, |ui| {
            for c in &connectors {
                ui_hover_label_with_menu(ui, &format!("  {}", c.name), |ui| {
                    if ui.button("View connections").clicked() {
                        let table = connector_connections_to_flexible_table(connectivity, c);
                        let data_table = flexible_table_to_data_table(&table);
                        let title = format!("{} - {} connections", context_title, c.name);
                        let _ = state.pending_wire_list.replace(Some(WireListViewState {
                            title,
                            table,
                            data_table,
                            last_exported_path: None,
                        }));
                        ui.close_menu();
                    }
                });
            }
        }).body_response.is_none() {}

    if egui::CollapsingHeader::new(format!("Wires ({})", wires.len()))
        .default_open(true)
        .show(ui, |ui| {
            for w in &wires {
                let label = if w.name.is_empty() { &w.id } else { &w.name };
                ui.label(format!("  {}", label));
            }
        }).body_response.is_none() {}
}

impl ApplicationState {

    fn update_project_outline(&mut self) {
        if let Some(project) = &self.project {
            self.project_outline = Some(ProjectOutline::new(project));
        } else {
            self.project_outline = None; // Clear project outline
        }
    }

    #[cfg(target_os = "windows")]
    fn load_session_data(&mut self) -> io::Result<()> {
        let hklu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(unvesys_key) = hklu.open_subkey("SOFTWARE\\Unvesys") {
            self.output_dir = unvesys_key.get_value("output_dir").unwrap_or_default();
            self.dark_mode = unvesys_key.get_value::<String, _>("theme")
                .map(|s| s != "light")
                .unwrap_or(true);
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn save_session_data(&mut self) -> io::Result<()> {
        // Save output directory and theme to registry
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = match hkcu.create_subkey("SOFTWARE\\Unvesys") {
            Ok(ok) => ok,
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        };
        key.set_value("output_dir", &self.output_dir)?;
        key.set_value("theme", &if self.dark_mode { "dark" } else { "light" })?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn load_session_data(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn save_session_data(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn log(&self, msg: RichText, expire: Option<Duration>, path: Option<PathBuf>) {
        self.log.borrow_mut().push((msg, SystemTime::now(), expire, path));
    }

    fn with_library_and_project<T>(&self, action: impl FnOnce(&Library, &Project) -> T) -> Result<T, String>  {
        if let Some(project) = &self.project {
            if let Some(library) = &self.library {
                Ok(action(&library, &project))
            } else {
                let msg = "Library not loaded!";
                self.log(RichText::new(msg).color(status_red()), Some(LOG_EXPIRATION), None);
                Err(msg.to_string())
            }
        } else {
            let msg = "Project not loaded!";
                self.log(RichText::new(msg).color(status_red()), Some(LOG_EXPIRATION), None);
            Err(msg.to_string())
        }
    }
}

pub struct Application {
    state: Arc<Mutex<ApplicationState>>,
    dock_state: DockState<DockTab>,
    /// NodeIndex of the table viewer pane (right side) for pushing new wire lists
    table_pane_node: NodeIndex,
}

impl<'a> eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1)); // refresh the UI occasionally

            // Apply theme from state
            {
                let dark_mode = self.state.lock().unwrap().dark_mode;
                ctx.set_visuals(if dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });
            }

            // Draw menu
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                self.menu_ui(ui);
            });

            //Draw bottom panel first, so CentralPanel knows how much space it gets
            egui::TopBottomPanel::bottom("bottom_panel")
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.output_dir_ui(ui);
                    self.log_ui(ui); // BUG: Layout issue, this takes an extra frame to pop up for some reason
                })
            });

            // Process pending wire list tabs - add to table pane (right side)
            if let Some(wire_list) = self.state.lock().unwrap().pending_wire_list.replace(None) {
                self.dock_state.set_focused_node_and_surface((SurfaceIndex::main(), self.table_pane_node));
                self.dock_state.push_to_focused_leaf(DockTab::WireList(wire_list));
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                let state = self.state.clone();
                let mut tab_viewer = AppTabViewer { state };
                DockArea::new(&mut self.dock_state)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut tab_viewer);
            });
    }

    fn on_exit(&mut self, ctx: Option<&eframe::glow::Context>) {
        self.state.lock().unwrap().save_session_data();
    }
}

impl Application {

     pub fn new(cc: &eframe::CreationContext) -> Self {
        // Any slow start-up work goes here
       
        let state = Arc::new(Mutex::new(ApplicationState {
            library: None,
            project: None,
            project_path: None,
            project_outline: None,
            output_dir: String::new(),
            log: Vec::new().into(),
            filter: "*".to_owned(),
            pending_wire_list: RefCell::new(None),
            dark_mode: true,
        }));

        let mut dock_state = DockState::new(vec![DockTab::Project]);
        // Split: project tree 30% left, empty pane 70% right (fills with space, no placeholder tab)
        let [_project_pane, table_pane] = dock_state.main_surface_mut()
            .split_right_empty(NodeIndex::root(), 0.3);

        let application = Self {
            state,
            dock_state,
            table_pane_node: table_pane,
        };

        application.load_library();
        {
            let mut state = application.state.lock().unwrap();
            state.load_session_data();
            cc.egui_ctx.set_visuals(if state.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });
        }
        application
    }

    fn load_project(&self, path: PathBuf) {
        // Clone Arc to avoid using self inside closure
        let state_clone = self.state.clone();

        // Wrap slow loading code in a thread
        std::thread::spawn(move || { // state_clone and path are moved
            let loading_msg = format!("Loading project {:?}", path.file_name().unwrap());
            state_clone.lock().unwrap().log(RichText::new(loading_msg).color(status_yellow()), None, None);
            let xmlpath = path.display().to_string();
            let xml = read_file(&xmlpath);
            match xml {
                Ok(xml) => {
                    match Project::new(&xml) {
                        Ok(project) => {
                            let need_library = {
                                let mut state = state_clone.lock().unwrap();
                                state.project = Some(project);
                                state.update_project_outline();
                                let done_loading_msg = format!("Loaded project {:?}", path.file_name().unwrap());
                                state.log(RichText::new(done_loading_msg).color(status_green()), Some(LOG_EXPIRATION), None);
                                state.library.is_none()
                            };
                            if need_library {
                                let project_dir_lib = path.parent().map(|p| p.join("Library.xml"));
                                if let Some(lib_path) = project_dir_lib {
                                    try_load_library_into_state(state_clone.clone(), vec![lib_path]);
                                }
                            }
                        },
                        _ => state_clone.lock().unwrap().log(RichText::new("Failed to parse project XML!").color(status_red()), Some(LOG_EXPIRATION), None),
                    }
                },
                _ => state_clone.lock().unwrap().log(RichText::new("Failed to load project XML file!").color(status_red()), Some(LOG_EXPIRATION), None),
            }
        });
    }

    fn load_library(&self) {
        try_load_library_into_state(self.state.clone(), vec![]);
    }

    fn menu_ui(&mut self, ui: &mut egui::Ui) {

        egui::menu::bar(ui, |ui| {
                menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        // OPEN
                        if ui.button("Open").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("VeSys XML Project", &["xml"])
                                .pick_file() {
                                self.state.clone().lock().unwrap().project_path = Some(path.clone());
                                self.load_project(path); // spawns a thread
                            }
                            ui.close_menu(); // close menu so it doesn't stay opened
                        }

                        // RELOAD
                        let project_path_is_some = self.state.clone().lock().unwrap().project_path.is_some();
                        if ui.add_enabled(project_path_is_some, Button::new("Reload")).clicked() {
                            self.load_project(self.state.clone().lock().unwrap().project_path.clone().unwrap_or_default());
                            ui.close_menu(); // close menu so it doesn't stay opened
                        }
                    });
                    ui.menu_button("Settings", |ui| {
                        let mut state = self.state.lock().unwrap();
                        if ui.selectable_label(state.dark_mode, "Dark").clicked() {
                            state.dark_mode = true;
                            let _ = state.save_session_data();
                            ui.close_menu();
                        }
                        if ui.selectable_label(!state.dark_mode, "Light").clicked() {
                            state.dark_mode = false;
                            let _ = state.save_session_data();
                            ui.close_menu();
                        }
                    });
                });
            });
    }

    fn filter_ui(ui: &mut egui::Ui, state: &mut ApplicationState) { 
        {
            if let Some(project) = &state.project {
                //let mut filter = String::new();
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.add(egui::TextEdit::singleline(&mut state.filter)
                        .desired_width(f32::INFINITY)
                        .hint_text("WILDCARD SYNTAX: * (any) \\ (escape) ? (single) Ex.: *J5*"));
                });
                ui.add_space(5.0);

            }
        }
    }


    fn project_view_ui(ui: &mut egui::Ui, state: &mut ApplicationState) {

        egui::ScrollArea::vertical()
        .max_width(f32::INFINITY)
        .auto_shrink([false, true])
        .show(ui, |ui| {
                    if let Some(project) = &state.project {
                        let pattern = Pattern::new(state.filter.clone());

                        // let id = ui.make_persistent_id("my_collapsing_header");
                        // //let mut selected = true
                        // egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
                        // .show_header(ui, |ui| {
                        //     ui.toggle_value(&mut state.selected, "Click to select/unselect");
                        //     //ui.radio_value(&mut self.radio_value, false, "");
                        //     //ui.radio_value(&mut self.radio_value, true, "");
                        // })
                        // .body(|ui| {
                        //     ui.label("The body is always custom");
                        // });

                        CollapsingHeader::new(project.get_name())
                        .default_open(true)
                        //.selectable(true) // UPGRADE
                        .show(ui, |ui| {
                            let c = CollapsingHeader::new("Logical Designs")
                            .default_open(true)
                            .show(ui, |ui| {
                                
                                if let Some(project_outline) = &state.project_outline {
                                    for design_outline in &project_outline.designs {
                                        CollapsingHeader::new(&design_outline.name)
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            CollapsingHeader::new("Harnesses")
                                            .default_open(true)
                                            .show(ui, |ui| {
                                                let filtered_harnesses = design_outline.harnesses.iter().filter(|x| pattern.matches(x) );
                                                for harness_name in filtered_harnesses {
                                                    ui_hover_label_with_menu(ui, harness_name, |ui| {
                                                        Application::logic_design_context_menu(ui, state, &design_outline.name, harness_name)
                                                    });
                                                }
                                            });
                                            CollapsingHeader::new("Elements")
                                            .default_open(true)
                                            .show(ui, |ui| {
                                                if let Some(design) = state.project.as_ref().and_then(|p| p.get_design(&design_outline.name)) {
                                                    let connectivity = design.get_connectivity();
                                                    let context_title = design_outline.name.to_string();
                                                    elements_list_ui(ui, &connectivity, &pattern, state, &context_title);
                                                } else {
                                                    ui.label("No connectivity data.");
                                                }
                                            });
                                        }).header_response.context_menu(|ui| { 
                                            if ui.button("Export index file").clicked() {
                                                println!("Exporting index...");
                                                let _ = state.with_library_and_project(|library, project| {
                                                    if let Some(design) = project.get_design(&design_outline.name) {
                                                        let diagrams = design.get_diagram_names();
                                                        for diagram in diagrams {
                                                            println!("{:?}", diagram);
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                    }
                                }
                            });

                            CollapsingHeader::new("Harness Designs")
                            .default_open(true)
                            .show(ui, |ui| {
                                if let Some(project_outline) = &state.project_outline {
                                    let filtered_harnesses: Vec<_> = project_outline.harnessdesigns.iter().filter(|x| pattern.matches(&x.name) ).collect();
                                    for harness_design in filtered_harnesses {
                                        CollapsingHeader::new(&harness_design.name)
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            if let Some(hd) = state.project.as_ref().and_then(|p| p.get_harness_design(&harness_design.name)) {
                                                let connectivity = hd.get_connectivity();
                                                let context_title = harness_design.name.to_string();
                                                elements_list_ui(ui, &connectivity, &pattern, state, &context_title);
                                            } else {
                                                ui.label("No connectivity data.");
                                            }
                                        }).header_response.context_menu(|ui| {
                                            Application::harness_design_context_menu(ui, state, &harness_design.name)
                                        });
                                    }
                                }
                            });
                        });
                    } else {
                        ui.horizontal_centered(|ui| {
                            ui.monospace(BG_GRAPHIC);
                        });
                    }
        });
    }
    
    fn logic_design_context_menu(ui: &mut egui::Ui, state: &ApplicationState, current_design_name: &str, current_harness: &str) {
        if ui.button("View wire list").clicked() {
            let _ = state.with_library_and_project(|library, project| {
                if let Some(design) = project.get_design(current_design_name) {
                    let connectivity = design.get_connectivity();
                    if let Ok(wiregroups) = crate::wirelist::generate_grouped_wirelist(library, &connectivity, current_harness) {
                        let table = wirelist_to_flexible_table(wiregroups);
                        let data_table = flexible_table_to_data_table(&table);
                        let _ = state.pending_wire_list.replace(Some(WireListViewState {
                            title: format!("{} - {}", current_design_name, current_harness),
                            table,
                            data_table,
                            last_exported_path: None,
                        }));
                    }
                }
            });
            ui.close_menu();
        }
        if ui.button("Export Excell wire list").clicked() {
            println!("Generating wire list for {}, {}", current_design_name, current_harness);
            let _ = state.with_library_and_project(|library, project| {
                let mut filepath = PathBuf::from(state.output_dir.clone());
                let filename = sanitise(&(current_harness.to_owned() + ".xlsx"));
                state.log(RichText::new(format!("Generating wire list {}", &filename)).color(status_yellow()), None, None);
                filepath.push(current_harness.to_owned() + ".xlsx");
                export_xslx_wirelist(&project, &library, &current_design_name, &current_harness, &filepath.display().to_string());
                state.log(RichText::new(format!("Finished wire list {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
            });
            ui.close_menu();
        }
        if ui.button("Export CSV label list").clicked() {
            println!("Generating CSV label list for {}, {}", current_design_name, current_harness);
            let _ = state.with_library_and_project(|library, project| {
                let mut filepath = PathBuf::from(state.output_dir.clone());
                let filename = sanitise(&(current_harness.to_owned() + ".xlsx"));
                state.log(RichText::new(format!("Generating CSV labellist {}", &filename)).color(status_yellow()), None, None);
                filepath.push(current_harness.to_owned() + ".csv");
                if let Err(e) = logic_harness_labels_csv_export(&project, &library, &current_design_name, &current_harness, &filepath.display().to_string()) {
                    println!{"{}", e};
                } else {
                    state.log(RichText::new(format!("Finished CSV label list {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
                }
            });
            ui.close_menu();
        }
        if ui.button("Export Schleuniger ASCII").clicked() {

            state.log(RichText::new(format!("{}{}","Exporting Schleuniger ASCII file to ", &state.output_dir)).color(status_yellow()), None, None);
            let _ = state.with_library_and_project(|library, project| {
                let mut filepath = PathBuf::from(state.output_dir.clone());
                let filename = sanitise(&(current_harness.to_owned() + ".xlsx"));
                state.log(RichText::new(format!("Generating wire list {}", &filename)).color(status_yellow()), None, None);
                filepath.push(current_harness.to_owned() + ".txt");
                export_xslx_wirelist(&project, &library, &current_design_name, &current_harness, &filepath.display().to_string());
                if let Ok(mut file) = File::create(filepath) {
                    logic_harness_shchleuniger_export(&project, &library, current_design_name, current_harness,  &mut file);
                    state.log(RichText::new(format!("Exported Schleuniger ASCII file to {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
                } else {
                    state.log(RichText::new(format!("Failed to create {}", &filename)).color(status_red()), Some(LOG_EXPIRATION), None);
                }
                state.log(RichText::new(format!("Finished wire list {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
            });


            ui.close_menu();
        }
        if ui.button("Export CSV BOM").clicked() {
            let _ = state.with_library_and_project(|library, project| {
                let mut filepath = PathBuf::from(state.output_dir.clone());
                let filename = sanitise(&(current_harness.to_owned() + " BOM" + ".csv"));
                filepath.push(filename.to_owned());
                if logic_harness_bom_export(project, library, &current_design_name, &current_harness, &filepath.display().to_string()).is_ok() {
                    state.log(RichText::new(format!("Finished CSV BOM list {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
                } else {
                    state.log(RichText::new(format!("Failed to create {}", &filename)).color(status_red()), Some(LOG_EXPIRATION), None);
                }
            });
            ui.close_menu();
        }
    }

    fn harness_design_context_menu(ui: &mut egui::Ui, state: &ApplicationState, current_design_name: &str) {
        if ui.button("View wire list").clicked() {
            let _ = state.with_library_and_project(|library, project| {
                if let Some(harness_design) = project.get_harness_design(current_design_name) {
                    let connectivity = harness_design.get_connectivity();
                    // Use graph-based wire sorting and grouping (traverse) prior to view
                    if let Ok(wiregroups) = crate::wirelist::generate_grouped_wirelist(library, &connectivity, "") {
                        let table = wirelist_to_flexible_table(wiregroups);
                        let data_table = flexible_table_to_data_table(&table);
                        let title = format!("{} - wire list", current_design_name);
                        let _ = state.pending_wire_list.replace(Some(WireListViewState {
                            title,
                            table,
                            data_table,
                            last_exported_path: None,
                        }));
                    } else {
                        state.log(RichText::new("Failed to generate wire list from connectivity").color(status_yellow()), Some(LOG_EXPIRATION), None);
                    }
                }
            });
            ui.close_menu();
        }

        if ui.button("Dump tables to CSV").clicked() {
            let _ = state.with_library_and_project(|_, project| {
                state.log(RichText::new(format!("{}{}", "Dumping tables to ", &state.output_dir)).color(status_yellow()), None, None);
                if let Some(harness_design) = project.get_harness_design(&current_design_name) {
                    let table_groups = harness_design.get_table_groups();
                    dump_tables(table_groups, &current_design_name, &state.output_dir);
                    state.log(RichText::new(format!("Dumped {} tables from \"{}\" to CSV", &table_groups.len().to_string(), &current_design_name)).color(status_green()), Some(LOG_EXPIRATION), None);
                }
            });
            ui.close_menu();
        }

        if ui.button("Export Schleuniger ASCII").clicked() {
            let _ = state.with_library_and_project(|library, project| {
                state.log(RichText::new(format!("{}{}","Exporting Schleuniger ASCII file to ", &state.output_dir)).color(status_yellow()), None, None);
                if let Some(harness_design) = project.get_harness_design(&current_design_name) {
                    let mut path : PathBuf = state.output_dir.clone().into();
                    let filename = current_design_name.to_owned() + ".txt";
                    path.push(String::from(&filename));
                    if let Ok(mut file) = File::create(path) {
                        harness_schleuniger_ascii_export(&library, &harness_design, &mut file);
                        state.log(RichText::new(format!("Exported Schleuniger ASCII file to {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
                    } else {
                        state.log(RichText::new(format!("Failed to create {}", &filename)).color(status_red()), Some(LOG_EXPIRATION), None);
                    }
                }
                ui.close_menu();
            });
        }

        if ui.button("Export CSV label list").clicked() {
            let _ = state.with_library_and_project(|library, project| {
                state.log(RichText::new(format!("{}{}","Exporting CSV label list file to ", &state.output_dir)).color(status_yellow()), None, None);
                if let Some(harness_design) = project.get_harness_design(&current_design_name) {
                    let mut path : PathBuf = state.output_dir.clone().into();
                    let filename = current_design_name.to_owned() + ".csv";
                    path.push(String::from(&filename));
                    if let Ok(mut file) = File::create(path) {
                        harness_labels_export(&library, &harness_design, &mut file);
                        state.log(RichText::new(format!("Exported CSV label list to {}", &filename)).color(status_green()), Some(LOG_EXPIRATION), None);
                    } else {
                        state.log(RichText::new(format!("Failed to create {}", &filename)).color(status_red()), Some(LOG_EXPIRATION), None);
                    }
                }
                ui.close_menu();
            });
        }

    }

    fn output_dir_ui(&mut self, ui: &mut egui::Ui) {
        let output_dir = &mut self.state.lock().unwrap().output_dir;
        ui.horizontal(|ui| {
            ui.label("Output Folder:");
            ui.add_sized(ui.available_size()-egui::vec2(75.0,0.0),egui::TextEdit::singleline(output_dir)
            .hint_text("Where do you want it?"));
            if ui.add(egui::Button::new("Browse").min_size(ui.available_size())).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() { // set_parent(frame.window_handle()) 
                    println!("{}", &path.display().to_string());
                    *output_dir = path.display().to_string();
                }
            }
            ui.end_row();
        });
    }

    fn log_ui(&mut self, ui: &mut egui::Ui) {
        // Show status with optional View link
        if let Some((status, timestamp, expire, path)) = self.state.try_lock().unwrap().log.borrow().last() {
            let duration = SystemTime::now().duration_since(*timestamp).unwrap_or(Duration::ZERO);

            if duration < expire.unwrap_or(Duration::MAX) {
                ui.horizontal(|ui| {
                    ui.label(status.clone());
                    if let Some(p) = path {
                        if p.exists() {
                            let link_color = ui.visuals().hyperlink_color;
                            if ui.add(
                                egui::Button::new(RichText::new(" View ").color(link_color).underline())
                                    .frame(false)
                            ).clicked() {
                                let _ = opener::open(p);
                            }
                        }
                    }
                });
            } else {
                ui.label("");
            }
        } else {
            ui.label("");
        }
    }

    fn wire_list_tab_ui(ui: &mut egui::Ui, view: &mut WireListViewState, state: &ApplicationState) {
        enum ExportAction {
            None,
            Xlsx,
            Csv,
            Schleuniger,
            Labels,
        }
        let mut action = ExportAction::None;
        let mut export_data: Option<(String, FlexibleTable, String)> = None;

        let output_dir = state.output_dir.clone();
        ui.heading(&view.title);
        ui.add_space(8.0);

        // Sync data_table (including user sorting/reordering/separator edits) to table before export.
        // Use display order so export matches what user sees in the grid.
        view.table.rows = view.data_table.rows_in_display_order();

        let is_connections_view = view.title.contains("connections");
        ui.horizontal(|ui| {
            if ui.button("Export XLSX").clicked() {
                export_data = Some((output_dir.clone(), view.table.clone(), view.title.clone()));
                action = ExportAction::Xlsx;
            }
            if ui.button("Export CSV").clicked() {
                export_data = Some((output_dir.clone(), view.table.clone(), view.title.clone()));
                action = ExportAction::Csv;
            }
            if !is_connections_view && ui.button("Export Schleuniger").clicked() {
                export_data = Some((output_dir.clone(), view.table.clone(), view.title.clone()));
                action = ExportAction::Schleuniger;
            }
            if !is_connections_view && ui.button("Export Labels CSV").clicked() {
                export_data = Some((output_dir.clone(), view.table.clone(), view.title.clone()));
                action = ExportAction::Labels;
            }
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let column_names: Vec<String> = view.table.columns.iter().map(|c| c.display.clone()).collect();
            wire_list_grid_ui(ui, &mut view.data_table, &column_names);
        });

        if let Some((output_dir, table, title)) = export_data {
            let base = title.split(" - ").last().unwrap_or("wirelist").to_string();
            let mut log_msg: Option<String> = None;
            let mut exported_path: Option<PathBuf> = None;
            match action {
                ExportAction::Xlsx => {
                    let filepath = PathBuf::from(&output_dir).join(sanitise(&format!("{}.xlsx", base)));
                    if let Ok(workbook) = Workbook::new(&filepath.display().to_string()) {
                        let xlsx_title = format!("{}, {}", title, Local::now().format("%m/%d/%Y"));
                        let ok = if table.column_index("pin").is_some() {
                            // Connector connections table - use generic export
                            flexible_table_to_xlsx_generic(&workbook, &table, &xlsx_title).is_ok()
                        } else {
                            // Wire list - use wire list formatter
                            let colormap = color_map();
                            let mut formatter = WireListXlsxFormatter::new(&workbook, &colormap);
                            formatter.print_header();
                            formatter.format_from_table(&table);
                            formatter.print_title(&xlsx_title);
                            true
                        };
                        if ok {
                            log_msg = Some(format!("Exported to {:?}", filepath.file_name()));
                            exported_path = Some(filepath);
                        }
                    }
                }
                ExportAction::Csv => {
                    let filepath = PathBuf::from(&output_dir).join(sanitise(&format!("{}.csv", base)));
                    if let Ok(mut file) = File::create(&filepath) {
                        let _ = flexible_table_to_csv_generic(&table, &mut file);
                        log_msg = Some(format!("Exported to {:?}", filepath.file_name()));
                        exported_path = Some(filepath);
                    }
                }
                ExportAction::Schleuniger => {
                    let filepath = PathBuf::from(&output_dir).join(sanitise(&format!("{}.txt", base)));
                    if let Ok(mut file) = File::create(&filepath) {
                        wirelist_to_schleuniger_ascii(&SchleunigerASCIIConfig::default(), &table, &mut file);
                        log_msg = Some(format!("Exported Schleuniger to {:?}", filepath.file_name()));
                        exported_path = Some(filepath);
                    }
                }
                ExportAction::Labels => {
                    let filepath = PathBuf::from(&output_dir).join(sanitise(&format!("{} labels.csv", base)));
                    if let Ok(mut file) = File::create(&filepath) {
                        let _ = flexible_table_to_labels_csv(&table, &mut file);
                        log_msg = Some(format!("Exported labels to {:?}", filepath.file_name()));
                        exported_path = Some(filepath);
                    }
                }
                _ => {}
            }
            if let Some(msg) = log_msg {
                state.log(
                    RichText::new(msg).color(status_green()),
                    Some(LOG_EXPIRATION),
                    exported_path.clone(),
                );
            }
            if let Some(path) = exported_path {
                view.last_exported_path = Some(path);
            }
        }
    }
}
