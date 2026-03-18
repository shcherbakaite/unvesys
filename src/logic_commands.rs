/*
 Logic design commands

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use crate::vysis::lookup_internal_harness_property;
use crate::vysis::Component;
use chrono::Local;
use polars::prelude::*;
use std::fs::File;
use crate::flexible_table::flexible_table_to_labels_csv;
use polars::datatypes::AnyValue;
use polars::frame::DataFrame;
use polars::frame::row::Row;
use crate::shchleuniger::wirelist_to_schleuniger_ascii;
use crate::shchleuniger::SchleunigerASCIIConfig;
use std::io::Write;
use crate::wire_list_xlsx_formatter::color_map;

use xlsxwriter::Workbook;

use crate::wirelist::{generate_grouped_wirelist, wirelist_to_flexible_table};
use crate::wire_list_xlsx_formatter::WireListXlsxFormatter;

use crate::vysis::Project;

use crate::vysyslib::Library;

use sanitise_file_name::sanitise;

pub fn export_xslx_wirelist(project: &Project,library: &Library, design_name: &str, harness: &str, filepath: &str ) -> Result<(), Box<dyn std::error::Error>> {
    let colormap = color_map();
    if let Some(design) = project.get_design(design_name) {
        let connectivity = design.get_connectivity();

        //let filepath = sanitise(filepath);
        //println!("{:?}", filepath);
        if let Ok(workbook) = Workbook::new(&filepath) {
            let wiregroups = generate_grouped_wirelist(library, &connectivity, harness).unwrap();
            let table = wirelist_to_flexible_table(wiregroups);

            let mut xlsx_formatter = WireListXlsxFormatter::new(&workbook, &colormap);
            xlsx_formatter.print_header();
            xlsx_formatter.format_from_table(&table);

            // Get harness part number
            let properties = design.get_design_properties();
            let pn = lookup_internal_harness_property(properties, "PN", harness ).unwrap_or_default();

            // Print title
            let current_date = Local::now().format("%m/%d/%Y").to_string();
            xlsx_formatter.print_title(&format!("{}, {}, {}, {}", design_name, harness, pn, current_date));
        }
        else 
        {

        }
    // outout device index
    } else {
        // can't open path
        // return
    }

    Ok(())
}

pub fn logic_harness_shchleuniger_export<W: Write>(project: &Project, library: &Library, design_name: &str, harness: &str, writer: W) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(design) = project.get_design(design_name) {
        let connectivity = design.get_connectivity();
        let wiregroups = generate_grouped_wirelist(library, &connectivity, harness).unwrap();
        let table = wirelist_to_flexible_table(wiregroups);
        wirelist_to_schleuniger_ascii(&SchleunigerASCIIConfig::default(), &table, writer);
    }
    Ok(())
}

pub fn logic_harness_labels_csv_export(project: &Project, library: &Library, design_name: &str, harness: &str, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Export DataFrame to CSV
    let mut file = File::create(filepath)?;
    logic_harness_labels_export(project, library, design_name, harness, file)
}


pub fn logic_harness_labels_export<W: Write>(project: &Project, library: &Library, design_name: &str, harness: &str, mut writer: W) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(design) = project.get_design(design_name) {
        let connectivity = design.get_connectivity();
        let wiregroups = generate_grouped_wirelist(library, &connectivity, harness).unwrap();
        let table = wirelist_to_flexible_table(wiregroups);
        flexible_table_to_labels_csv(&table, &mut writer)?;
    }
    Ok(())
}

fn group_by_sum(df: &DataFrame) -> PolarsResult<DataFrame> {
    df
    .group_by(["mpn", "partnum", "descr"])?
    .sum()
}

pub fn logic_harness_bom_export(project: &Project, library: &Library, design_name: &str, harness: &str, filepath: &str) -> Result<(), Box<dyn std::error::Error>>  {
    // Export DataFrame to CSV
    let mut writer = File::create(filepath)?;

    let mut bom_df = get_logic_harness_bom_dataframe(project, library, design_name, harness).map(|df| {
        let mut df = df.select(["partnum", "descr", "quantity_sum"]).unwrap();
        df.rename("partnum", "Part Number");
        df.rename("descr", "Description");
        df.rename("quantity_sum", "Quantity/Length");
        df
    });

    CsvWriter::new(&mut writer)
    .include_header(true)
    .finish(&mut bom_df?)?;
    Ok(())
}

pub fn get_logic_harness_bom_dataframe(project: &Project, library: &Library, design_name: &str, harness: &str) -> Result<DataFrame, String> {
    if let Some(design) = project.get_design(design_name) {
        let connectivity = design.get_connectivity();
        let wires = connectivity.get_wires(harness);
        let components = connectivity.get_harness_components(harness);

        let mut bom_rows = Vec::new();
        let na = "N/A".to_owned();
        
        let fields = [
            Field::new("mpn", DataType::String), 
            Field::new("partnum", DataType::String), 
            Field::new("descr", DataType::String), 
            Field::new("quantity", DataType::Float32)];
        
        let schema: Schema = Schema::from_iter(fields);

        for c in components.iter() {
            if let Some(partnumber) = c.get_partno() {
                let customer_partnumber = c.get_customer_partno().unwrap_or(&na);
                let description = c.lookup_library_description(library).unwrap_or(&na);
                let mut quantity = 1.0;
                if let Component::Wire(w) = c {
                    quantity = w.dom.wirelength;
                }
                let row : Row = Row::new([AnyValue::String(partnumber), AnyValue::String(&customer_partnumber), AnyValue::String(&description), AnyValue::Float32(quantity)].to_vec());
                bom_rows.push(row);
            }
        }

        let df = DataFrame::from_rows_and_schema(&bom_rows, &schema);
        let df = group_by_sum(&df.unwrap());
        println!("{:?}",&df);
        Ok(df.unwrap())
    } else {
        Err("No design".to_owned())
    }
}
