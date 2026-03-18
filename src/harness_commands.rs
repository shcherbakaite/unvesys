/*
 Harness design commands

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use std::fs::File;
use crate::flexible_table::{flexible_table_to_labels_csv, wire_list_columns, FlexibleTable, TableData, TableDataMut};
use crate::vysyslib::Library;
use std::io::Write;
use crate::vesys_table_reader::VysysTableReader;
use crate::vysis::HarnessDesign;
use csv::Writer;
use crate::vysisxml::XmlTableGroup;
use std::path::PathBuf;
use crate::shchleuniger::{wirelist_to_schleuniger_ascii, SchleunigerASCIIConfig};
use crate::table_convert::vysys_table_to_flexible_table;
use sanitise_file_name::sanitise;

/// Dump all harness tables into CSV
pub fn dump_tables(table_groups: &Vec<XmlTableGroup>, basename: &str, dir: &str) -> std::io::Result<()> {
    let mut i = 0;
    let mut path : PathBuf = dir.into();
    for group in table_groups.iter() {
        println!("{:?}", group.title);
        for table in group.tablefamily.table.iter() {
            if let Some(datacache) = &table.tabledatacache {
                //println!("{:?}", datacache.colhdrnames);
                let mut path = path.clone();
                let mut title = group.title.clone();
                //title.truncate(200);
                let filename = format!("{}-{}-{}.csv", basename, title, i);
                let filename = sanitise(&filename);
                path.push(filename.clone());
                println!("{:?}", path);
                i = i + 1;
                let mut wtr = Writer::from_path(path)?;
                let header = &datacache.colhdrnames.row;
                let header_names : Vec<String> = header.cellvals.iter().map(|v| {
                    v.cval.val.clone()
                }).collect();
                println!("{:?}", header_names);
                wtr.write_record(&header_names)?;

                for datarow in datacache.datavalues.datarow.iter() {
                    let cols : Vec<String> = datarow.cellval.iter().map(|v| {
                        v.cval.val.clone()
                    }).collect();

                    wtr.write_record(&cols)?;
                }

            }
        }
    }

    Ok(())
}

/// Get SHCHLEUNIGER wire processing property of the wire from the library
pub fn lookup_wire_processing<'a>(library: &'a Library, harness_design: &'a HarnessDesign<'a>, wire_name: &'a str) -> Option<&'a str> {
    harness_design.get_connectivity().get_wire_by_name(wire_name).and_then(|wire| {
        wire.dom.partnumber.as_ref().and_then(|part_number| {
            library.lookup_wire_property(&part_number, "PROCESSING")
        })
    })
}

pub fn harness_labels_csv_export(_library: &Library, harness_design: &HarnessDesign, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filepath)?;
    harness_labels_export(_library, harness_design, file)
}

/// Export harness design HarnessWireTable into CSV label file
pub fn harness_labels_export<W: Write>(_library: &Library, harness_design: &HarnessDesign, mut writer: W) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let table_groups = harness_design.get_table_groups();
    let harness_wire_table = table_groups.into_iter().find(|x| x.decorationname == "HarnessWireTable");

    if let Some(harness_wire_table) = harness_wire_table {
        println!("{}", &harness_wire_table.title);
        let table_reader = VysysTableReader::new(&harness_wire_table);
        let table = vysys_table_to_flexible_table(&table_reader);
        flexible_table_to_labels_csv(&table, &mut writer)?;
    } else {
        return Err("HarnessWireTable not found".into());
    }
    Ok(())
}



/// Export harness design HarnessWireTable into SHCHLEUNIGER ASCII file for the wire cutting machine
pub fn harness_schleuniger_ascii_export<W: Write>(library: &Library, harness_design: &HarnessDesign, writer: W) -> std::result::Result<(), String> {
    let table_groups = harness_design.get_table_groups();
    let harness_wire_table = table_groups.into_iter().find(|x| x.decorationname == "HarnessWireTable");

    if let Some(harness_wire_table) = harness_wire_table {
        println!("{}", &harness_wire_table.title);
        let table_reader = VysysTableReader::new(&harness_wire_table);
        let mut table = vysys_table_to_flexible_table(&table_reader);

        if table.column_index(wire_list_columns::PROCESSING).is_none() {
            table.columns.push(crate::flexible_table::ColumnSpec {
                id: wire_list_columns::PROCESSING.to_string(),
                display: "Processing".to_string(),
            });
            let proc_col = table.column_count() - 1;
            for row in 0..table.row_count() {
                let wire_name = table.get_by_name(row, wire_list_columns::WIRE_NAME).unwrap_or("").to_string();
                let processing = lookup_wire_processing(library, harness_design, &wire_name).unwrap_or("N/A");
                table.set(row, proc_col, processing);
            }
        }

        wirelist_to_schleuniger_ascii(&SchleunigerASCIIConfig::default(), &table, writer);
        Ok(())
    } else {
        Err("No wire table!".to_string())
    }
}