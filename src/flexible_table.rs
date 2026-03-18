/*
 Flexible table - unified intermediate representation for wire lists

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
*/

/// Pure data access - no presentation
pub trait TableData {
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;
    fn column_name(&self, col: usize) -> Option<&str>;
    fn column_index(&self, name: &str) -> Option<usize>;

    fn get(&self, row: usize, col: usize) -> Option<&str>;
    fn get_by_name(&self, row: usize, column: &str) -> Option<&str> {
        self.column_index(column).and_then(|c| self.get(row, c))
    }
}

/// Mutable data access (for editing, building)
pub trait TableDataMut: TableData {
    fn set(&mut self, row: usize, col: usize, value: impl Into<String>) -> bool;
    fn push_row(&mut self, cells: impl IntoIterator<Item = impl Into<String>>);
}

/// Border style for cell formatting
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Thin,
    Medium,
    Thick,
}

/// Per-cell formatting - full control over borders, colors, fonts
#[derive(Clone, Debug, Default)]
pub struct CellFormat {
    pub border_top: Option<BorderStyle>,
    pub border_right: Option<BorderStyle>,
    pub border_bottom: Option<BorderStyle>,
    pub border_left: Option<BorderStyle>,
    pub bg_color: Option<[u8; 4]>,
    pub fg_color: Option<[u8; 4]>,
    pub font_bold: bool,
    pub font_italic: bool,
    pub font_size: Option<f32>,
}

/// Formatting is separate from data - can be computed or stored
pub trait TableFormat {
    fn format(&self, row: usize, col: usize) -> CellFormat;
}

/// Column specification
#[derive(Clone, Debug)]
pub struct ColumnSpec {
    pub id: String,
    pub display: String,
}

/// Row metadata for wire list (group boundaries, color hints)
#[derive(Clone, Debug, Default)]
pub struct RowMeta {
    pub group_boundary_below: bool,
    pub color_code: Option<String>,
}

/// A single row with cells and optional metadata
#[derive(Clone, Debug)]
pub struct FlexibleRow {
    pub cells: Vec<String>,
    pub meta: RowMeta,
}

/// Unified table representation
#[derive(Clone, Debug)]
pub struct FlexibleTable {
    pub columns: Vec<ColumnSpec>,
    pub rows: Vec<FlexibleRow>,
}

impl FlexibleTable {
    pub fn new(columns: Vec<ColumnSpec>) -> Self {
        FlexibleTable {
            columns,
            rows: Vec::new(),
        }
    }

    pub fn with_columns(columns: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        let columns: Vec<ColumnSpec> = columns
            .into_iter()
            .map(|(id, display)| ColumnSpec {
                id: id.into(),
                display: display.into(),
            })
            .collect();
        Self::new(columns)
    }

    pub fn row_meta(&self, row: usize) -> Option<&RowMeta> {
        self.rows.get(row).map(|r| &r.meta)
    }
}

impl TableData for FlexibleTable {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn column_count(&self) -> usize {
        self.columns.len()
    }

    fn column_name(&self, col: usize) -> Option<&str> {
        self.columns.get(col).map(|c| c.id.as_str())
    }

    fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.id == name)
    }

    fn get(&self, row: usize, col: usize) -> Option<&str> {
        let row_data = self.rows.get(row)?;
        let cell = row_data.cells.get(col)?;
        Some(cell.as_str())
    }
}

impl TableDataMut for FlexibleTable {
    fn set(&mut self, row: usize, col: usize, value: impl Into<String>) -> bool {
        let Some(row_data) = self.rows.get_mut(row) else {
            return false;
        };
        if col >= row_data.cells.len() {
            row_data.cells.resize(col + 1, String::new());
        }
        row_data.cells[col] = value.into();
        true
    }

    fn push_row(&mut self, cells: impl IntoIterator<Item = impl Into<String>>) {
        let cells: Vec<String> = cells.into_iter().map(|c| c.into()).collect();
        self.rows.push(FlexibleRow {
            cells,
            meta: RowMeta::default(),
        });
    }
}

/// Write generic CSV (all columns) from table - similar to XLSX export
pub fn flexible_table_to_csv_generic<W: std::io::Write>(
    table: &FlexibleTable,
    writer: &mut W,
) -> std::io::Result<()> {
    fn escape_csv(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
    // Header
    let header: Vec<String> = table.columns.iter().map(|c| escape_csv(&c.display)).collect();
    writeln!(writer, "{}", header.join(","))?;
    // Rows
    for row in &table.rows {
        let cells: Vec<String> = row
            .cells
            .iter()
            .map(|c| escape_csv(c))
            .collect();
        writeln!(writer, "{}", cells.join(","))?;
    }
    Ok(())
}

/// Write labels CSV (From, To columns) from table data
pub fn flexible_table_to_labels_csv<W: std::io::Write>(
    data: &impl TableData,
    writer: &mut W,
) -> std::io::Result<()> {
    writeln!(writer, "From,To")?;
    for row in 0..data.row_count() {
        let from = data.get_by_name(row, wire_list_columns::FROM_LABEL).unwrap_or("");
        let to = data.get_by_name(row, wire_list_columns::TO_LABEL).unwrap_or("");
        let (from_val, to_val) = if !from.is_empty() && !to.is_empty() {
            (from.to_string(), to.to_string())
        } else {
            let from_dev = data.get_by_name(row, wire_list_columns::WIRE_FROM_PINLIST).unwrap_or("");
            let from_cav = data.get_by_name(row, wire_list_columns::WIRE_FROM_CAVITY).unwrap_or("");
            let to_dev = data.get_by_name(row, wire_list_columns::WIRE_TO_PINLIST).unwrap_or("");
            let to_cav = data.get_by_name(row, wire_list_columns::WIRE_TO_CAVITY).unwrap_or("");
            (format!("{}-{}", from_dev, from_cav), format!("{}-{}", to_dev, to_cav))
        };
        writeln!(writer, "{},{}", from_val, to_val)?;
    }
    Ok(())
}

/// Canonical wire list column IDs
pub mod wire_list_columns {
    pub const WIRE_NAME: &str = "WIRE_NAME";
    pub const SHORT_DESCRIPTION: &str = "SHORT_DESCRIPTION";
    pub const CUSTOMER_PART_NUMBER: &str = "CUSTOMER_PART_NUMBER";
    pub const MATERIAL: &str = "MATERIAL";
    pub const SPEC: &str = "SPEC";
    pub const COLOR: &str = "COLOR";
    pub const COLOR_DESCRIPTION: &str = "COLOR_DESCRIPTION";
    pub const WIRE_FROM_PINLIST: &str = "WIRE_FROM_PINLIST";
    pub const WIRE_FROM_CAVITY: &str = "WIRE_FROM_CAVITY";
    pub const WIRE_TERMINAL_STRIP_LEN1: &str = "WIRE_TERMINAL_STRIP_LEN1";
    pub const WIRE_TO_PINLIST: &str = "WIRE_TO_PINLIST";
    pub const WIRE_TO_CAVITY: &str = "WIRE_TO_CAVITY";
    pub const WIRE_TERMINAL_STRIP_LEN2: &str = "WIRE_TERMINAL_STRIP_LEN2";
    pub const MODIFIED_LENGTH: &str = "MODIFIED_LENGTH";
    pub const TWIST_WIDTH: &str = "TWIST_WIDTH";
    pub const PROCESSING: &str = "PROCESSING";
    pub const FROM_LABEL: &str = "FROM_LABEL";
    pub const TO_LABEL: &str = "TO_LABEL";
    pub const FROM_TERM_PARTNO: &str = "FROM_TERM_PARTNO";
    pub const FROM_TERM_NAME: &str = "FROM_TERM_NAME";
    pub const TO_TERM_PARTNO: &str = "TO_TERM_PARTNO";
    pub const TO_TERM_NAME: &str = "TO_TERM_NAME";
}
