/*
 Cutting machine outputs

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use std::str::FromStr;
use std::io::Write;
use csv::Terminator;
use csv::WriterBuilder;
use crate::flexible_table::{TableData, wire_list_columns};


pub struct SchleunigerASCIIConfig {
    left_position: f32,
    right_position: f32,
    min_double_label_length: f32 // length of wire before switching to single centered label
}

impl Default for SchleunigerASCIIConfig {
    fn default() -> Self {
        SchleunigerASCIIConfig { 
            left_position:3.25, 
            right_position: 2.0, 
            min_double_label_length: 8.0
        } 
    }
}

fn center_label(config: &SchleunigerASCIIConfig, record: Vec<String>) -> Vec<String> {
    // Center label
    let length_f32 = f32::from_str(&record[2]);
    if let Ok(length_f32) = length_f32 {
        if length_f32 < config.min_double_label_length {
            let pos = length_f32/2.0 + 0.5;
            let mut record = record.clone();
            record[9] = pos.to_string();
            record[10] = "".to_string();
            record[11] = "".to_string();
            return record;
        } else {
            return record;
        }
    } else {
        return record;
    }
}

pub fn wirelist_to_schleuniger_ascii<W: Write>(config: &SchleunigerASCIIConfig, wire_list: &impl TableData, writer: W) {
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .terminator(Terminator::CRLF)
        .from_writer(writer);

    wtr.write_record(vec![
        String::from("Import"), String::from("ASCII"),
    ]);
    wtr.write_record(vec![
        String::from("Units"), String::from("inch"),
    ]);
    wtr.write_record(vec![
        String::from("Area"), String::from("TT"),
    ]);
    wtr.write_record(vec![
        String::from("Name"),
        String::from("Part"),
        String::from("Length"),
        String::from("Style"),
        String::from("Stripping type"),
        String::from("Right strip"),
        String::from("Left strip"),
        String::from("Partial strip %"),
        String::from("Marker left text"),
        String::from("Marker left position"),
        String::from("Marker right text"),
        String::from("Marker right position"),
        String::from("Autorotation"),
    ]);

    for row in 0..wire_list.row_count() {
        let wire_from_pinlist = wire_list.get_by_name(row, wire_list_columns::WIRE_FROM_PINLIST).unwrap_or("");
        let wire_from_cavity = wire_list.get_by_name(row, wire_list_columns::WIRE_FROM_CAVITY).unwrap_or("");
        let wire_terminal_strip_len1 = wire_list.get_by_name(row, wire_list_columns::WIRE_TERMINAL_STRIP_LEN1).unwrap_or("");
        let wire_to_pinlist = wire_list.get_by_name(row, wire_list_columns::WIRE_TO_PINLIST).unwrap_or("");
        let wire_to_cavity = wire_list.get_by_name(row, wire_list_columns::WIRE_TO_CAVITY).unwrap_or("");
        let wire_terminal_strip_len2 = wire_list.get_by_name(row, wire_list_columns::WIRE_TERMINAL_STRIP_LEN2).unwrap_or("");
        let modified_length = wire_list.get_by_name(row, wire_list_columns::MODIFIED_LENGTH).unwrap_or("");
        let processing = wire_list.get_by_name(row, wire_list_columns::PROCESSING).unwrap_or("");

        let from = format!("{}-{}", wire_from_pinlist, wire_from_cavity);
        let to = format!("{}-{}", wire_to_pinlist, wire_to_cavity);
        let article_name = format!("{}/{}", from, to);
        let part = (row + 1).to_string();
        let length = modified_length.to_string();
        let style = processing.to_string();
        let stripping_type = "9".to_owned();
        let right_strip = wire_terminal_strip_len1.to_string();
        let left_strip = wire_terminal_strip_len2.to_string();
        let partial_strip = "50%".to_owned();
        let marker_text = "\\#C@7\\&n\\&@7".to_owned();
        let marker_left_position = config.left_position;
        let marker_right_position = config.right_position;
        let autorotation = "X".to_owned();

        wtr.write_record(center_label(config, vec![
            article_name,
            part,
            length,
            style,
            stripping_type,
            right_strip,
            left_strip,
            partial_strip,
            marker_text.clone(),
            marker_left_position.to_string(),
            marker_text,
            marker_right_position.to_string(),
            autorotation,
        ]));
    }
}