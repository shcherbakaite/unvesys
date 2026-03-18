/*
 Table conversion - build FlexibleTable from various sources

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
*/

use crate::flexible_table::{ColumnSpec, FlexibleRow, FlexibleTable, RowMeta};
use crate::vesys_table_reader::VysysTableReader;
use crate::vysisxml::XmlTableGroup;
use std::collections::HashMap;

/// Map from XML column names to canonical wire list column IDs (when they differ)
fn harness_column_mapping() -> HashMap<String, String> {
    let mut m = HashMap::new();
    // Common XML column name variations -> canonical ID
    m.insert("From Device".to_string(), "WIRE_FROM_PINLIST".to_string());
    m.insert("From Cavity".to_string(), "WIRE_FROM_CAVITY".to_string());
    m.insert("To Device".to_string(), "WIRE_TO_PINLIST".to_string());
    m.insert("To Cavity".to_string(), "WIRE_TO_CAVITY".to_string());
    m.insert("Wire Name".to_string(), "WIRE_NAME".to_string());
    m.insert("Length".to_string(), "MODIFIED_LENGTH".to_string());
    m
}

/// Convert VysysTableReader (HarnessWireTable) to FlexibleTable
pub fn vysys_table_to_flexible_table(table_reader: &VysysTableReader) -> FlexibleTable {
    let column_map = &table_reader.column_map;
    let mapping = harness_column_mapping();

    let mut col_entries: Vec<_> = column_map
        .iter()
        .map(|(name, (idx, display))| (*idx, name.clone(), display.clone()))
        .collect();
    col_entries.sort_by_key(|(idx, _, _)| *idx);

    let columns: Vec<ColumnSpec> = col_entries
        .iter()
        .map(|(_idx, name, display)| {
            let canonical = mapping.get(name).cloned().unwrap_or_else(|| name.clone());
            ColumnSpec {
                id: canonical,
                display: display.clone(),
            }
        })
        .collect();

    let xml_column_names: Vec<&String> = col_entries.iter().map(|(_, name, _)| name).collect();
    let mut table = FlexibleTable::new(columns);

    for row in table_reader.get_row_iter() {
        let cells: Vec<String> = xml_column_names
            .iter()
            .map(|k| row.get_column(k).unwrap_or("N/A").to_string())
            .collect();
        table.rows.push(FlexibleRow {
            cells,
            meta: RowMeta::default(),
        });
    }

    table
}

/// Convert XmlTableGroup (raw harness table) to FlexibleTable - for dump preview
pub fn table_group_to_flexible_table(table_group: &XmlTableGroup) -> FlexibleTable {
    let table_reader = VysysTableReader::new(table_group);
    vysys_table_to_flexible_table(&table_reader)
}
