/*
 Wire list data grid - egui table view

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
*/

use crate::flexible_table::{FlexibleTable, FlexibleRow};
use egui_data_table::{DataTable, Renderer, RowViewer};
use std::borrow::Cow;

/// RowViewer for FlexibleRow - wire list with group separators
pub struct WireListRowViewer {
    column_names: Vec<String>,
}

impl WireListRowViewer {
    pub fn new(columns: &[String]) -> Self {
        WireListRowViewer {
            column_names: columns.to_vec(),
        }
    }
}

impl RowViewer<FlexibleRow> for WireListRowViewer {
    fn num_columns(&mut self) -> usize {
        self.column_names.len()
    }

    fn column_name(&mut self, column: usize) -> Cow<'static, str> {
        self.column_names
            .get(column)
            .map(|s| Cow::Owned(s.clone()))
            .unwrap_or(Cow::Borrowed(""))
    }

    fn is_sortable_column(&mut self, column: usize) -> bool {
        column < self.column_names.len()
    }

    fn create_cell_comparator(&mut self) -> impl Fn(&FlexibleRow, &FlexibleRow, usize) -> std::cmp::Ordering {
        let column_names_len = self.column_names.len();
        move |a: &FlexibleRow, b: &FlexibleRow, col: usize| {
            if col >= column_names_len {
                return std::cmp::Ordering::Equal;
            }
            let a_val = a.cells.get(col).map(|s| s.as_str()).unwrap_or("");
            let b_val = b.cells.get(col).map(|s| s.as_str()).unwrap_or("");
            a_val.cmp(b_val)
        }
    }

    fn show_cell_view(&mut self, ui: &mut egui::Ui, row: &FlexibleRow, column: usize) {
        let text = row.cells.get(column).map(|s| s.as_str()).unwrap_or("");
        ui.label(text);
    }

    fn show_cell_editor(
        &mut self,
        ui: &mut egui::Ui,
        row: &mut FlexibleRow,
        column: usize,
    ) -> Option<egui::Response> {
        if let Some(cell) = row.cells.get_mut(column) {
            let resp = ui.add(
                egui::TextEdit::singleline(cell)
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(2.0, 2.0)),
            );
            Some(resp)
        } else {
            None
        }
    }

    fn set_cell_value(&mut self, src: &FlexibleRow, dst: &mut FlexibleRow, column: usize) {
        if let (Some(s), Some(d)) = (src.cells.get(column), dst.cells.get_mut(column)) {
            *d = s.clone();
        } else if column >= dst.cells.len() {
            dst.cells.resize(column + 1, String::new());
            if let Some(s) = src.cells.get(column) {
                dst.cells[column] = s.clone();
            }
        }
    }

    fn new_empty_row(&mut self) -> FlexibleRow {
        FlexibleRow {
            cells: vec![String::new(); self.column_names.len()],
            meta: Default::default(),
        }
    }

    fn clone_row(&mut self, row: &FlexibleRow) -> FlexibleRow {
        FlexibleRow {
            cells: row.cells.clone(),
            meta: row.meta.clone(),
        }
    }

    fn row_separator_below(&mut self, row: &FlexibleRow) -> bool {
        row.meta.group_boundary_below
    }

    fn row_separator_editable(&mut self) -> bool {
        true
    }

    fn toggle_separator_below(&mut self, row: &mut FlexibleRow) {
        row.meta.group_boundary_below = !row.meta.group_boundary_below;
    }
}

/// Build DataTable from FlexibleTable
pub fn flexible_table_to_data_table(table: &FlexibleTable) -> DataTable<FlexibleRow> {
    table.rows.clone().into_iter().collect()
}

/// Show wire list in a scrollable grid
pub fn wire_list_grid_ui(ui: &mut egui::Ui, table: &mut DataTable<FlexibleRow>, column_names: &[String]) {
    let mut viewer = WireListRowViewer::new(column_names);
    ui.add(Renderer::new(table, &mut viewer));
}
