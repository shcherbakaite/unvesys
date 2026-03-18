/*
 Wire list Excell formatter

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use xlsxwriter::prelude::FormatColor;
use std::collections::HashMap;
use xlsxwriter::*;
use crate::xlsxtable::*;
use xlsxwriter::format::*;
use xlsxwriter::worksheet::PaperType;
use crate::flexible_table::{TableData, wire_list_columns};

pub struct WireListXlsxFormatter<'a> {
    table : XLSXTable,
    workbook : &'a Workbook,
    sheet : Worksheet<'a>,
    current_row: u32,
    bg_colormap: &'a HashMap<String, FormatColor>
}

impl WireListXlsxFormatter<'_> {
    // Column definitions
    // Wire
    const WIRE_ITEM: u16 = 0;
    // Wire
    const SHORT_DESCR: u16 = 1;
    // From
    const FROM_DEVICE: u16 = 2;
    const FROM_DASH: u16 = 3;
    const FROM_PIN: u16 = 4;
    // Terminal
    const FROM_TERM_PARTNO: u16 = 5; // Merge
    const FROM_TERM_NAME: u16 = 6;   // ^
    // Wire material
    const WIRE_PARTNO: u16 = 7; // Merge
    const WIRE_NAME: u16 = 8;   // ^
    const WIRE_COLOR: u16 = 9;
    const WIRE_LEN: u16 = 10;
    // Terminal
    const TO_TERM_PARTNO: u16 = 11; // Merge
    const TO_TERM_NAME: u16 = 12;   // ^
    // To
    const TO_DEVICE: u16 = 13;
    const TO_DASH: u16 = 14;
    const TO_PIN: u16 = 15;

    const LABEL_FROM: u16 = 17;
    const LABEL_TO: u16 = 18;

    // Margins
    const LEFT:u16 = 0;
    const TOP:u32 = 0;

    pub fn new<'a>(workbook: &'a xlsxwriter::Workbook, bg_colormap: &'a HashMap<std::string::String, xlsxwriter::format::FormatColor>) -> WireListXlsxFormatter<'a> {
        let mut table = XLSXTable::new();
        let mut format = Format::new();
        format.set_align(FormatAlignment::Center);
        table.set_default_format(format);
        WireListXlsxFormatter {
            table : table,
            workbook : workbook,
            sheet : workbook.add_worksheet(None).unwrap(),
            current_row : Self::TOP + 1,
            bg_colormap : bg_colormap
        }
    }

    pub fn print_title(&mut self, title: &str) {
        self.sheet.set_header(title);
    }

    pub fn print_header(&mut self) {
        let row = Self::TOP;
        // Wire
        self.table.set_cell(row, Self::LEFT + Self::WIRE_ITEM, "Wire Item");
        self.table.set_col_width_pixels(Self::LEFT + Self::WIRE_ITEM, 150);
        // Short Descr
        self.table.set_cell(row, Self::LEFT + Self::SHORT_DESCR, "Description");
        self.table.set_col_width_pixels(Self::LEFT + Self::SHORT_DESCR, 150);
        // From
        self.table.set_cell(row, Self::LEFT + Self::FROM_DEVICE, "Device");
        self.table.set_cell(row, Self::LEFT + Self::FROM_DASH, "-");
        self.table.set_col_width_pixels(Self::LEFT + Self::FROM_DASH, 20);
        self.table.set_cell(row, Self::LEFT + Self::FROM_PIN, "Pin");
        self.table.set_col_width_pixels(Self::LEFT + Self::FROM_PIN, 35);
        // Terminal
        self.table.set_cell(row, Self::LEFT + Self::FROM_TERM_PARTNO, "Termination");
        self.table.set_col_width_pixels(Self::LEFT + Self::FROM_TERM_PARTNO, 125);
        self.table.set_cell(row, Self::LEFT + Self::FROM_TERM_NAME, "");
        self.table.set_col_width_pixels(Self::LEFT + Self::FROM_TERM_NAME, 125);
        // Wire
        self.table.set_cell(row, Self::LEFT + Self::WIRE_PARTNO, "Wire");
        self.table.set_col_width_pixels(Self::LEFT + Self::WIRE_PARTNO, 125);
        self.table.set_cell(row, Self::LEFT + Self::WIRE_NAME, "");
        self.table.set_cell(row, Self::LEFT + Self::WIRE_COLOR, "Color");
        self.table.set_cell(row, Self::LEFT + Self::WIRE_LEN, "Length");
        // Terminal
        self.table.set_cell(row, Self::LEFT + Self::TO_TERM_PARTNO, "Termination");
        self.table.set_col_width_pixels(Self::LEFT + Self::TO_TERM_PARTNO, 125);
        self.table.set_cell(row, Self::LEFT + Self::TO_TERM_NAME, "");
        self.table.set_col_width_pixels(Self::LEFT + Self::TO_TERM_NAME, 125);
        // To
        self.table.set_cell(row, Self::LEFT + Self::TO_DEVICE, "Device");
        self.table.set_cell(row, Self::LEFT + Self::TO_DASH, "-");
        self.table.set_col_width_pixels(Self::LEFT + Self::TO_DASH, 20);
        self.table.set_cell(row, Self::LEFT + Self::TO_PIN, "Pin");
        self.table.set_col_width_pixels(Self::LEFT + Self::TO_PIN, 35);
        // From/To Label Columns
        self.table.set_cell(row, Self::LEFT + Self::LABEL_FROM, "From");
        self.table.set_cell(row, Self::LEFT + Self::LABEL_TO, "To");
    }

    /// Write all rows from a FlexibleTable
    pub fn format_from_table(&mut self, data: &crate::flexible_table::FlexibleTable) {
        let meta_provider = |row: usize| -> Option<(bool, Option<String>)> {
            data.row_meta(row).map(|m| (m.group_boundary_below, m.color_code.clone()))
        };
        self.format_from_table_with_meta(data, meta_provider);
    }

    /// Write all rows from table data with custom row metadata
    pub fn format_from_table_with_meta<F>(&mut self, data: &impl TableData, mut meta: F)
    where
        F: FnMut(usize) -> Option<(bool, Option<String>)>,
    {
        for row in 0..data.row_count() {
            let wire_item = data.get_by_name(row, wire_list_columns::WIRE_NAME).unwrap_or("");
            let material = data.get_by_name(row, wire_list_columns::MATERIAL).unwrap_or("");
            let spec = data.get_by_name(row, wire_list_columns::SPEC).unwrap_or("");
            let wire_mat_spec = format!("{} {}", material, spec);

            self.table.set_cell(self.current_row, Self::LEFT + Self::WIRE_ITEM, wire_item);
            self.table.set_cell(self.current_row, Self::LEFT + Self::SHORT_DESCR, data.get_by_name(row, wire_list_columns::SHORT_DESCRIPTION).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::FROM_DEVICE, data.get_by_name(row, wire_list_columns::WIRE_FROM_PINLIST).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::FROM_DASH, "-");
            self.table.set_cell(self.current_row, Self::LEFT + Self::FROM_PIN, data.get_by_name(row, wire_list_columns::WIRE_FROM_CAVITY).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::FROM_TERM_PARTNO, data.get_by_name(row, wire_list_columns::FROM_TERM_PARTNO).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::FROM_TERM_NAME, data.get_by_name(row, wire_list_columns::FROM_TERM_NAME).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::WIRE_PARTNO, data.get_by_name(row, wire_list_columns::CUSTOMER_PART_NUMBER).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::WIRE_NAME, &wire_mat_spec);
            self.table.set_cell(self.current_row, Self::LEFT + Self::WIRE_COLOR, data.get_by_name(row, wire_list_columns::COLOR_DESCRIPTION).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::WIRE_LEN, data.get_by_name(row, wire_list_columns::MODIFIED_LENGTH).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::TO_TERM_PARTNO, data.get_by_name(row, wire_list_columns::TO_TERM_PARTNO).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::TO_TERM_NAME, data.get_by_name(row, wire_list_columns::TO_TERM_NAME).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::TO_DEVICE, data.get_by_name(row, wire_list_columns::WIRE_TO_PINLIST).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::TO_DASH, "-");
            self.table.set_cell(self.current_row, Self::LEFT + Self::TO_PIN, data.get_by_name(row, wire_list_columns::WIRE_TO_CAVITY).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::LABEL_FROM, data.get_by_name(row, wire_list_columns::FROM_LABEL).unwrap_or(""));
            self.table.set_cell(self.current_row, Self::LEFT + Self::LABEL_TO, data.get_by_name(row, wire_list_columns::TO_LABEL).unwrap_or(""));

            let row_meta = meta(row);
            if let Some((_, ref color_code)) = row_meta {
                if let Some(ref cc) = color_code {
                    let color_code_upper = cc.to_uppercase();
                    self.table.modify_region_format(&XLSXTableRegion {
                        first_row: self.current_row,
                        first_col: Self::LEFT,
                        last_row: self.current_row,
                        last_col: Self::LEFT + Self::TO_PIN,
                    }, &|format| {
                        format.set_bg_color(*self.bg_colormap.get(&color_code_upper).unwrap_or(&FormatColor::White));
                    });
                }
            }

            self.current_row += 1;

            if row_meta.map(|(gb, _)| gb).unwrap_or(false) {
                self.table.set_region_border_bottom(&XLSXTableRegion {
                    first_row: self.current_row - 1,
                    first_col: Self::LEFT,
                    last_row: self.current_row - 1,
                    last_col: Self::LEFT + Self::TO_PIN,
                }, FormatBorder::Medium);
            }
        }
    }
}

impl Drop for WireListXlsxFormatter<'_> {
    fn drop(&mut self) {
        // Finalize outside border
        self.table.set_region_border(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::TO_PIN
        }, FormatBorder::Medium);
        // Header border
        let header_region = &XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT,
            last_row: Self::TOP,
            last_col: Self::LEFT + Self::TO_PIN
        };
        self.table.set_region_border(&header_region, FormatBorder::Medium);
        // Header format
        self.table.modify_region_format(&header_region, &|format| {
            format.set_bold();
        });
        // Wire item separator
        self.table.set_region_border_right(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::WIRE_ITEM
        }, FormatBorder::Dotted);
        // Left wire end separator
        self.table.set_region_border_right(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::FROM_TERM_NAME
        }, FormatBorder::Dotted);
        // Right wire end separator
        self.table.set_region_border_right(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::WIRE_LEN
        }, FormatBorder::Dotted);
        // Left align Description column
         self.table.modify_region_format(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT + Self::SHORT_DESCR,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::SHORT_DESCR
        }, &|format| {
            format.set_align(FormatAlignment::Left);
        });
        // Right align FROM device column
         self.table.modify_region_format(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT + Self::FROM_DEVICE,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::FROM_DEVICE
        }, &|format| {
            format.set_align(FormatAlignment::Right);
        });
        // Left align FROM pin column
         self.table.modify_region_format(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT + Self::FROM_PIN,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::FROM_PIN
        }, &|format| {
            format.set_align(FormatAlignment::Left);
        });
        // Right align TO device column
         self.table.modify_region_format(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT + Self::TO_DEVICE,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::TO_DEVICE
        }, &|format| {
            format.set_align(FormatAlignment::Right);
        });
        // Left align TO pin column
         self.table.modify_region_format(&XLSXTableRegion {
            first_row: Self::TOP,
            first_col: Self::LEFT + Self::TO_PIN,
            last_row: self.current_row - 1,
            last_col: Self::LEFT + Self::TO_PIN
        }, &|format| {
            format.set_align(FormatAlignment::Left);
        });
        // Set paper
        self.sheet.set_paper(PaperType::Tabloid);
        // Set orientation
        self.sheet.set_landscape();

        self.table.render_to_worksheet(&mut self.sheet);
    }
}

/// Export any FlexibleTable to XLSX with a simple format (header + data rows).
/// Used for connector connections and other non-wire-list tables.
pub fn flexible_table_to_xlsx_generic(
    workbook: &Workbook,
    table: &crate::flexible_table::FlexibleTable,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::flexible_table::TableData;
    use xlsxwriter::format::FormatAlignment;

    let mut sheet = workbook.add_worksheet(None)?;
    let _ = sheet.set_header(title);

    let mut header_format = Format::new();
    header_format.set_bold();
    let mut default_format = Format::new();
    default_format.set_align(FormatAlignment::Center);

    // Header row
    for (col, col_spec) in table.columns.iter().enumerate() {
        let _ = sheet.write_string(0, col as u16, &col_spec.display, Some(&header_format));
        sheet.set_column_pixels(col as u16, col as u16, 100, None);
    }

    // Data rows
    for row in 0..table.row_count() {
        for col in 0..table.column_count() {
            let value = table.get(row, col).unwrap_or("");
            let _ = sheet.write_string((row + 1) as u32, col as u16, value, Some(&default_format));
        }
    }

    Ok(())
}

pub fn color_map() -> Box<HashMap<String, FormatColor>> {
    let mut bg_color_map: HashMap<String, FormatColor> = HashMap::new();
    bg_color_map.insert("PK".to_string(), FormatColor::Custom(0xffccff)); // 5v format
    bg_color_map.insert("RD".to_string(), FormatColor::Custom(0xff9999)); // 12v format
    bg_color_map.insert("BN".to_string(), FormatColor::Custom(0xd3a77b)); // 24v format
    bg_color_map.insert("OR".to_string(), FormatColor::Custom(0xfff2cc)); // 48v format
    bg_color_map.insert("BL".to_string(), FormatColor::Custom(0xddebf7)); // GND format
    bg_color_map.insert("YL".to_string(), FormatColor::Custom(0xffffd1)); // Analog/Bat+ format
    bg_color_map.insert("TN".to_string(), FormatColor::Custom(0xead5c0)); // 24v DO
    bg_color_map.insert("BK".to_string(), FormatColor::Custom(0xf2f2f2)); // 24v DI
    bg_color_map.insert("GN".to_string(), FormatColor::Custom(0xc6e0b4)); // Sinking output
    bg_color_map.insert("PR".to_string(), FormatColor::Custom(0xbc9dd4)); // Purple
    return Box::new(bg_color_map);
}



// pub fn process_connection<'a>(connection: (&'a  Connection<'a>, &'a Option<&'a str>), library: &Library ) -> WireEndEntry {
//     let mut wire_end_info : WireEndEntry = Default::default();
//     match connection {
//         (Connection::Connector(connector,pin), termination) => {
//             if connector.is_ring() {
//                 // If ring is connected to some device, find that device
//                 let ring_connection = connector.get_ring_connection();
//                 match ring_connection {
//                     Some(ring_connection) => {
//                         match ring_connection {
//                             Connection::Device(mated_device,mated_pin) => {
//                                 wire_end_info.device = mated_device.get_name().into();
//                                 wire_end_info.pin = mated_pin.get_name().into();
//                             }
//                             Connection::GroundDevice(mated_device,mated_pin) => {
//                                 wire_end_info.device = mated_device.get_name().into();
//                                 wire_end_info.pin = mated_pin.get_name().into();
//                             }
//                             _ => {
//                                 println!("Ring can't connect to device {}", connector.get_name().to_string());
//                             }
//                         }
//                     }
//                     None => {}
//                 }
//                 // Ring is not connected anywhere, leave device empty
//                 // Show ring as termination
//                 wire_end_info.termination = connector.get_customer_partno().into();
//                 let partno = connector.get_partno();
//                 wire_end_info.termination_name = library.lookup_terminal_short_name(partno).unwrap_or_default().into();
//             } else
//             {
//                 //println!("{}", connector.get_name());
//                 // wire_end_info.device = connector.get_name().into();
//                 // wire_end_info.pin = pin.get_name().into();
//                 // wire_end_info.termination = "TODO".into();

//                 // Same as devices
//                 wire_end_info.device = connector.get_name().into();
//                 wire_end_info.pin = pin.get_name().into();
//                 wire_end_info.termination = "TODO".into();
//                 if let Some(termination) = termination {
//                     //println!("termination {}", termination);
//                     let terminal_partnumber = library.lookup_customer_partnumber(*termination);
//                     wire_end_info.termination = terminal_partnumber.unwrap_or_default().into();
//                     wire_end_info.termination_name = library.lookup_terminal_short_name(*termination).unwrap_or_default().into();
//                 }
//             }
//         }
//         (Connection::Device(device,pin), termination) => {
//             wire_end_info.device = device.get_name().into();
//             wire_end_info.pin = pin.get_name().into();
//             wire_end_info.termination = "TODO".into();
//             if let Some(termination) = termination {
//                 if termination.trim() == "^" { // two wire going to same terminal of device
//                     wire_end_info.termination = "^".into(); // leave ^ alone for now
//                     wire_end_info.termination_name = "".into();
//                 } else {
//                     let terminal_partnumber = library.lookup_customer_partnumber(*termination);
//                     wire_end_info.termination = terminal_partnumber.unwrap_or_default().into();
//                     wire_end_info.termination_name = library.lookup_terminal_short_name(*termination).unwrap_or_default().into();
//                 }
//             }
//         }
//         (Connection::GroundDevice(device,pin), termination) => {
//             wire_end_info.device = device.get_name().into();
//             wire_end_info.pin = pin.get_name().into();
//             wire_end_info.termination = "TODO".into();
//             if let Some(termination) = termination {
//                 let terminal_partnumber = library.lookup_customer_partnumber(*termination);
//                 wire_end_info.termination = terminal_partnumber.unwrap_or_default().into();
//                 wire_end_info.termination_name = library.lookup_terminal_short_name(*termination).unwrap_or_default().into();
//             }
//         }
//         (Connection::Splice(splice,pin), _) => {
//             wire_end_info.device = splice.get_name().into();
//             wire_end_info.pin = pin.get_name().into();
//             wire_end_info.termination = "".into();
//             // For splices, use splice part number and short name instead of termination
//             if let Some(library_partnumber) = splice.get_partno() {
//                 let customer_partnumber = library.lookup_customer_partnumber(library_partnumber);
//                 wire_end_info.termination = customer_partnumber.unwrap_or(library_partnumber).into();
//                 wire_end_info.termination_name = library.lookup_terminal_short_name(library_partnumber).unwrap_or_default().into();
//             }
//             // TODO: Read properties of the device to find out which side of the splice wire is meant to 
//         }
//     }
//     //wire_end_info.termination = "TODO".into();
//     wire_end_info
// }


// pub fn output_connector_io(project: &Project, library: &Library, design_name: &str, connector: &str, filepath: &str ) -> Result<(), Box<dyn std::error::Error>> {
//     let colormap = color_map();
//     if let Some(design) = project.get_design(design_name) {
//         if let Ok(workbook) = Workbook::new(filepath) {
//             // Get harness wires            
//             let wires = design.get_connectivity().get_connector_wires(&connector);

//         }
//     }

//     todo!();
// }

// pub fn generate_grouped_wirelist(library: &Library, connectivity: &Connectivity, harness: &str) -> Result<Vec<Vec<WireEntry>>, Box<dyn std::error::Error>> {
//     // Get harness wires            
//     let wires = connectivity.get_wires(&harness);

//     // Processed wire list
//     let mut wire_list: WireList = WireList::new();

//     for wire in wires {
//         let connections = wire.get_connections();
//         let connection_left = connections.get(0);

//         // This is where most of VeSys non-sense is fixed regarding where wire is connected and what goes on it
//         let left_wire_end = connection_left.map(|(connection_left, termination)| {
//             let mut left_wire_end = process_connection((connection_left, termination), &library);
//             left_wire_end
//         });

//         let connection_right = connections.get(1);
//         let right_wire_end = connection_right.map(|(connection_right, termination)| {
//             let mut right_wire_end = process_connection((connection_right, termination), &library);
//             right_wire_end
//         });

//         wire_list.wires.insert(
//             WireEntry {
//                 name : wire.get_name().into(),
//                 descr : wire.get_short_descr().into(),
//                 partno : wire.get_customer_partno().into(),
//                 material : wire.get_material().into(),
//                 spec : wire.get_spec().into(),
//                 color_code : wire.get_color().into(),
//                 color_description : library.get_color_description(wire.get_color()).unwrap_or_default().into(),
//                 length : wire.get_length(),
//                 left : left_wire_end.clone(),
//                 right : right_wire_end.clone(),
//                 twisted_with : wire.get_twisted_with().map(|x| x.into()) // check if wire is in twist with any other
//             }
//         );
//     }

//     let mut wiregroups : Vec<Vec<WireEntry>> = traverse(&wire_list);

//     Ok(wiregroups)
// }



// pub fn output_cutlist(project: &Project,library: &Library, design_name: &str, harness: &str, filepath: &str ) -> Result<(), Box<dyn std::error::Error>> {
    
//     if let Some(design) = project.get_design(design_name) {
//         let connectivity = design.get_connectivity();
//         output_cutlist_from_connectivity(library, &connectivity, harness, filepath);

//     } else if let Some(harnessdesign) = project.get_harness_design(design_name) {
//         let connectivity = harnessdesign.get_connectivity();
//         output_cutlist_from_connectivity(library, &connectivity, "", filepath);
//     }

//     Ok(())
// }


