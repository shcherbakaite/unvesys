/*
 Wire list

 Vladislav Shcherbakov
 Copyright Firefly Automatix 2024
 9/18/2024 3:34:10 PM
*/

use crate::flexible_table::{ColumnSpec, FlexibleRow, FlexibleTable, RowMeta};
use crate::traverse::traverse;
use crate::vysis::Connectivity;
use crate::vysyslib::Library;
use crate::vysis::Connection;
use std::hash::Hasher;
use std::hash::Hash;
use std::collections::HashSet;
use std::cmp::Ordering::*;

#[derive(Clone)]
pub struct WireList {
    pub wires:HashSet<WireEntry>,
}

#[derive(Clone, Debug)]
pub struct WireEntry {
    pub name: Box<str>,
    pub descr: Box<str>,
    pub partno: Box<str>,
    pub material: Box<str>,
    pub spec: Box<str>,
    pub processing: Box<str>,
    pub color_code: Box<str>,
    pub color_description: Box<str>,
    pub length: f32,
    pub left: Option<WireEndEntry>,
    pub right: Option<WireEndEntry>,
    pub twisted_with: Option<Box<str>>
}

impl PartialEq for WireEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for WireEntry {}

impl Hash for WireEntry {

fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }

}

impl WireEntry {
    pub fn swap(&mut self) {
        let left_old = self.left.clone();
        self.left = self.right.clone();
        self.right = left_old;
    }
}

#[derive(Default)]
#[derive(Clone)]
#[derive(Debug)]
pub struct WireEndEntry {
    pub device : Box<str>,
    pub pin : Box<str>,
    pub termination_partnumber : Box<str>,
    pub termination_name : Box<str>,
    pub strip: f32
}




impl WireList {
    pub fn new() -> WireList {
        WireList {
            wires: HashSet::new()
        }
    }


    // pub fn get_wires_between_devices(&self, from: &str, _to: &str) {
    //     for wire in self.wires.iter() {
    //         match (wire.left, wire.right) {
    //             (Some(left_end), Some(right_end)) => {

    //             }
    //         }
    //     }
    // }

    //remove
    //add

    pub fn get_wires_between_devices(&self, from: &str, to: &str) -> Vec<WireEntry> {
        let mut result = Vec::new();

        for wire_entry in self.wires.iter() {
            match (&wire_entry.left, &wire_entry.right)  {
                (Some(left_end), Some(right_end))  => {
                    // If device ends match, add wire name to the list
                    if ((left_end.device.as_ref() == from) && (right_end.device.as_ref() == to)) ||
                       ((right_end.device.as_ref() == from) && (left_end.device.as_ref() == to)) {
                        result.push(wire_entry.clone());
                    }
                }
                _ => {} // skip if either end is missing
            }
        };
        return result;
    }

}

pub fn sort_wirelist_by_left_device_pin(wirelist: &mut Vec<WireEntry>) {
    wirelist.sort_by(|a,b| {
        match (&a.left, &b.left) {
            (Some(left_end_a), Some(left_end_b)) => {
                let device_cmp = left_end_a.device.cmp(&left_end_b.device);
                let pin_number_a_opt = left_end_a.pin.parse::<i32>();
                let pin_number_b_opt = left_end_b.pin.parse::<i32>();
                // If same device name, compare by pin
                if (device_cmp == Equal) {
                    match (pin_number_a_opt, pin_number_b_opt) {
                        (Ok(pin_num_a), Ok(pin_num_b)) => {
                            pin_num_a.cmp(&pin_num_b)
                        }
                        _ => {
                            // compare alphanumerically
                            //print!("{}{}", left_end_a.pin, )
                            let pin_cmp = left_end_a.pin.cmp(&left_end_b.pin);
                            if (pin_cmp == Equal) {
                                device_cmp // if pins are equal defer to device comparison  
                            } else {
                                pin_cmp
                            }
                        }
                    }
                } else { // one of the pins is not a number
                    device_cmp
                }
            }
            _ => Equal
        }

    });
}

fn process_connection<'a>(connection: (&'a  Connection<'a>, &'a Option<&'a str>), library: &Library ) -> WireEndEntry {
    let mut wire_end_info : WireEndEntry = Default::default();
    match connection {
        (Connection::Connector(connector,pin), termination) => {
            if connector.is_ring() {
                // If ring is connected to some device, find that device
                let ring_connection = connector.get_ring_connection();
                match ring_connection {
                    Some(ring_connection) => {
                        match ring_connection {
                            Connection::Device(mated_device,mated_pin) => {
                                wire_end_info.device = mated_device.get_name().into();
                                wire_end_info.pin = mated_pin.get_name().into();
                            }
                            Connection::GroundDevice(mated_device,mated_pin) => {
                                wire_end_info.device = mated_device.get_name().into();
                                wire_end_info.pin = mated_pin.get_name().into();
                            }
                            _ => {
                                println!("Ring can't connect to device {}", connector.get_name().to_string());
                            }
                        }
                    }
                    None => {}
                }
                // Ring is not connected anywhere, leave device empty
                // Show ring as termination
                wire_end_info.termination_partnumber = connector.get_customer_partno().into();
                let partno = connector.get_partno();
                let terminal_dom = library.lookup_terminal_part(partno);
                wire_end_info.termination_name = library.lookup_terminal_short_name(terminal_dom).unwrap_or_default().into();
                wire_end_info.strip = terminal_dom.and_then(|terminal_dom| Some(terminal_dom.striplength)).unwrap_or_default().into();
            } else
            {
                //println!("{}", connector.get_name());
                // wire_end_info.device = connector.get_name().into();
                // wire_end_info.pin = pin.get_name().into();
                // wire_end_info.termination = "TODO".into();

                // Same as devices
                wire_end_info.device = connector.get_name().into();
                wire_end_info.pin = pin.get_name().into();
                wire_end_info.termination_partnumber = "TODO".into();
                if let Some(termination) = termination {
                    //println!("termination {}", termination);
                    let terminal_partnumber = library.lookup_customer_partnumber(*termination);
                    wire_end_info.termination_partnumber = terminal_partnumber.unwrap_or_default().into();
                    let terminal_dom = library.lookup_terminal_part(termination);
                    wire_end_info.termination_name = library.lookup_terminal_short_name(terminal_dom).unwrap_or_default().into();
                    wire_end_info.strip = terminal_dom.and_then(|terminal_dom| Some(terminal_dom.striplength)).unwrap_or_default().into();
                }
            }
        }
        (Connection::Device(device,pin), termination) => {
            wire_end_info.device = device.get_name().into();
            wire_end_info.pin = pin.get_name().into();
            wire_end_info.termination_partnumber = "TODO".into();
            if let Some(termination) = termination {
                if termination.trim() == "^" { // two wire going to same terminal of device
                    wire_end_info.termination_partnumber = "^".into(); // leave ^ alone for now
                    wire_end_info.termination_name = "".into();
                } else {
                    let terminal_partnumber = library.lookup_customer_partnumber(*termination);
                    wire_end_info.termination_partnumber = terminal_partnumber.unwrap_or_default().into();
                    let terminal_dom = library.lookup_terminal_part(termination);
                    wire_end_info.termination_name = library.lookup_terminal_short_name(terminal_dom).unwrap_or_default().into();
                    wire_end_info.strip = terminal_dom.and_then(|terminal_dom| Some(terminal_dom.striplength)).unwrap_or_default().into();
                }
            }
        }
        (Connection::GroundDevice(device,pin), termination) => {
            wire_end_info.device = device.get_name().into();
            wire_end_info.pin = pin.get_name().into();
            wire_end_info.termination_partnumber = "TODO".into();
            if let Some(termination) = termination {
                let terminal_partnumber = library.lookup_customer_partnumber(*termination);
                wire_end_info.termination_partnumber = terminal_partnumber.unwrap_or_default().into();
                let terminal_dom = library.lookup_terminal_part(termination);
                wire_end_info.termination_name = library.lookup_terminal_short_name(terminal_dom).unwrap_or_default().into();
                wire_end_info.strip = terminal_dom.and_then(|terminal_dom| Some(terminal_dom.striplength)).unwrap_or_default().into();
            }
        }
        (Connection::Splice(splice,pin), _) => {
            wire_end_info.device = splice.get_name().into();
            wire_end_info.pin = pin.get_name().into();
            wire_end_info.termination_partnumber= "".into();
            // For splices, use splice part number and short name instead of termination
            if let Some(library_partnumber) = splice.get_partno() {
                let customer_partnumber = library.lookup_customer_partnumber(library_partnumber);
                wire_end_info.termination_partnumber = customer_partnumber.unwrap_or(library_partnumber).into();
                let splice_dom = library.lookup_splice_part(library_partnumber);
                wire_end_info.termination_name = library.lookup_splice_short_name(splice_dom).unwrap_or_default().into();
                wire_end_info.strip = splice_dom.and_then(|splice_dom| Some(splice_dom.striplength)).unwrap_or_default().into();
            }
            // TODO: Read properties of the device to find out which side of the splice wire is meant to 
        }
    }
    //wire_end_info.termination = "TODO".into();
    wire_end_info
}


pub fn generate_grouped_wirelist(library: &Library, connectivity: &Connectivity, harness: &str) -> Result<Vec<Vec<WireEntry>>, Box<dyn std::error::Error>> {
    // Get harness wires            
    let wires = connectivity.get_wires(&harness);

    // Processed wire list
    let mut wire_list: WireList = WireList::new();

    for wire in wires {
        let connections = wire.get_connections();
        let connection_left = connections.get(0);

        // This is where most of VeSys non-sense is fixed regarding where wire is connected and what goes on it
        let left_wire_end = connection_left.map(|(connection_left, termination)| {
            let mut left_wire_end = process_connection((connection_left, termination), &library);
            left_wire_end
        });

        let connection_right = connections.get(1);
        let right_wire_end = connection_right.map(|(connection_right, termination)| {
            let mut right_wire_end = process_connection((connection_right, termination), &library);
            right_wire_end
        });

        wire_list.wires.insert(
            WireEntry {
                name : wire.get_name().into(),
                descr : wire.get_short_descr().into(),
                partno : wire.get_customer_partno().into(),
                material : wire.get_material().into(),
                spec : wire.get_spec().into(),
                processing : wire.dom.partnumber.as_ref().and_then(|part_number| {
                    library.lookup_wire_property(&part_number, "PROCESSING")
                }).unwrap_or_default().into(),
                color_code : wire.get_color().into(),
                color_description : library.get_color_description(wire.get_color()).unwrap_or_default().into(),
                length : wire.get_length(),
                left : left_wire_end.clone(),
                right : right_wire_end.clone(),
                twisted_with : wire.get_twisted_with().map(|x| x.into()) // check if wire is in twist with any other
            }
        );
    }

    let mut wiregroups : Vec<Vec<WireEntry>> = traverse(&wire_list);

    Ok(wiregroups)
}

/// Convert grouped wire list to FlexibleTable
pub fn wirelist_to_flexible_table(grouped_wirelist: Vec<Vec<WireEntry>>) -> FlexibleTable {
    use crate::flexible_table::wire_list_columns;

    let columns = vec![
        ColumnSpec { id: wire_list_columns::WIRE_NAME.to_string(), display: "Wire Item".to_string() },
        ColumnSpec { id: wire_list_columns::SHORT_DESCRIPTION.to_string(), display: "Description".to_string() },
        ColumnSpec { id: wire_list_columns::CUSTOMER_PART_NUMBER.to_string(), display: "Part No".to_string() },
        ColumnSpec { id: wire_list_columns::MATERIAL.to_string(), display: "Material".to_string() },
        ColumnSpec { id: wire_list_columns::SPEC.to_string(), display: "Spec".to_string() },
        ColumnSpec { id: wire_list_columns::COLOR.to_string(), display: "Color".to_string() },
        ColumnSpec { id: wire_list_columns::COLOR_DESCRIPTION.to_string(), display: "Color Desc".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_FROM_PINLIST.to_string(), display: "From Device".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_FROM_CAVITY.to_string(), display: "From Pin".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_TERMINAL_STRIP_LEN1.to_string(), display: "From Strip".to_string() },
        ColumnSpec { id: wire_list_columns::FROM_TERM_PARTNO.to_string(), display: "From Term".to_string() },
        ColumnSpec { id: wire_list_columns::FROM_TERM_NAME.to_string(), display: "From Term Name".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_TO_PINLIST.to_string(), display: "To Device".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_TO_CAVITY.to_string(), display: "To Pin".to_string() },
        ColumnSpec { id: wire_list_columns::WIRE_TERMINAL_STRIP_LEN2.to_string(), display: "To Strip".to_string() },
        ColumnSpec { id: wire_list_columns::TO_TERM_PARTNO.to_string(), display: "To Term".to_string() },
        ColumnSpec { id: wire_list_columns::TO_TERM_NAME.to_string(), display: "To Term Name".to_string() },
        ColumnSpec { id: wire_list_columns::MODIFIED_LENGTH.to_string(), display: "Length".to_string() },
        ColumnSpec { id: wire_list_columns::TWIST_WIDTH.to_string(), display: "Twist".to_string() },
        ColumnSpec { id: wire_list_columns::PROCESSING.to_string(), display: "Processing".to_string() },
        ColumnSpec { id: wire_list_columns::FROM_LABEL.to_string(), display: "From".to_string() },
        ColumnSpec { id: wire_list_columns::TO_LABEL.to_string(), display: "To".to_string() },
    ];

    let mut table = FlexibleTable::new(columns);

    for mut group in grouped_wirelist {
        sort_wirelist_by_left_device_pin(&mut group);
        let last_in_group = group.len().saturating_sub(1);
        for (i, wire) in group.into_iter().enumerate() {
            let left = wire.left.clone().unwrap_or_default();
            let right = wire.right.clone().unwrap_or_default();
            let twisted = wire.twisted_with.as_ref().map(|x| format!(" (⤫ {})", x)).unwrap_or_default();
            let wire_item = format!("{}{}", wire.name.as_ref(), twisted);
            let from_label = format!("{}-{}", left.device.as_ref(), left.pin.as_ref());
            let to_label = format!("{}-{}", right.device.as_ref(), right.pin.as_ref());

            let cells = vec![
                wire_item,
                wire.descr.to_string(),
                wire.partno.to_string(),
                wire.material.to_string(),
                wire.spec.to_string(),
                wire.color_code.to_string(),
                wire.color_description.to_string(),
                left.device.to_string(),
                left.pin.to_string(),
                left.strip.to_string(),
                left.termination_partnumber.to_string(),
                left.termination_name.to_string(),
                right.device.to_string(),
                right.pin.to_string(),
                right.strip.to_string(),
                right.termination_partnumber.to_string(),
                right.termination_name.to_string(),
                wire.length.to_string(),
                wire.twisted_with.unwrap_or_default().to_string(),
                wire.processing.to_string(),
                from_label,
                to_label,
            ];

            let meta = RowMeta {
                group_boundary_below: i == last_in_group,
                color_code: if wire.color_code.is_empty() { None } else { Some(wire.color_code.to_string()) },
            };

            table.rows.push(FlexibleRow { cells, meta });
        }
    }

    table
}

fn format_connection(conn: &crate::vysis::Connection) -> String {
    use crate::vysis::Connection;
    match conn {
        Connection::Device(d, p) => format!("{}-{}", d.get_name(), p.get_name()),
        Connection::GroundDevice(d, p) => format!("{}-{}", d.get_name(), p.get_name()),
        Connection::Connector(c, p) => format!("{}-{}", c.get_name(), p.get_name()),
        Connection::Splice(s, p) => format!("{}-{}", s.get_name(), p.get_name()),
    }
}

/// Build a table showing all pins of a connector and what wires are connected to each.
/// Shows ALL pins from the connector (even empty ones) with wire name, description, and FROM-TO.
pub fn connector_connections_to_flexible_table<'a>(
    connectivity: &crate::vysis::Connectivity<'a>,
    connector_dom: &crate::vysisxml::XmlConnector,
) -> FlexibleTable {
    use crate::flexible_table::{ColumnSpec, FlexibleRow, RowMeta};
    use crate::vysis::Connection;
    use std::collections::HashMap;

    let connector_name = connector_dom.name.as_str();
    let harness = connector_dom.harness.as_deref().unwrap_or("");

    #[derive(Clone)]
    struct WireInfo {
        name: String,
        description: String,
        from_label: String,
        to_label: String,
    }

    // Build map: pin_id -> Vec<WireInfo>
    let mut pin_wires: HashMap<String, Vec<WireInfo>> = HashMap::new();

    // Initialize with all pins from connector (even empty)
    for pin in &connector_dom.pin {
        pin_wires.entry(pin.id.clone()).or_default();
    }

    // Scan wires for this harness and find connections to this connector
    let wires = connectivity.get_wires(harness);
    for wire in wires {
        for conn_dom in &wire.dom.connection {
            if let Some(connection) = connectivity.get_connection_by_pinref(conn_dom.pinref.as_ref()) {
                if let Connection::Connector(conn, _pin) = connection {
                    if conn.get_name() == connector_name {
                        let connections = wire.get_connections();
                        let from_label = connections
                            .first()
                            .map(|(c, _)| format_connection(c))
                            .unwrap_or_default();
                        let to_label = connections
                            .get(1)
                            .map(|(c, _)| format_connection(c))
                            .unwrap_or_default();

                        pin_wires
                            .entry(conn_dom.pinref.clone())
                            .or_default()
                            .push(WireInfo {
                                name: wire.get_name().to_string(),
                                description: wire.get_short_descr().to_string(),
                                from_label,
                                to_label,
                            });
                    }
                }
            }
        }
    }

    // Sort pins by name for consistent display
    let mut pin_ids: Vec<_> = pin_wires.keys().cloned().collect();
    pin_ids.sort_by(|a, b| {
        let name_a = connector_dom.pin.iter().find(|p| &p.id == a).map(|p| &p.name).unwrap_or(a);
        let name_b = connector_dom.pin.iter().find(|p| &p.id == b).map(|p| &p.name).unwrap_or(b);
        name_a.cmp(name_b)
    });

    let columns = vec![
        ColumnSpec {
            id: "pin".to_string(),
            display: "Pin".to_string(),
        },
        ColumnSpec {
            id: "wire".to_string(),
            display: "Wire".to_string(),
        },
        ColumnSpec {
            id: "description".to_string(),
            display: "Description".to_string(),
        },
        ColumnSpec {
            id: "from".to_string(),
            display: "From".to_string(),
        },
        ColumnSpec {
            id: "to".to_string(),
            display: "To".to_string(),
        },
    ];

    let mut table = FlexibleTable::new(columns);

    for pin_id in pin_ids {
        let pin_name = connector_dom
            .pin
            .iter()
            .find(|p| p.id == pin_id)
            .map(|p| p.name.as_str())
            .unwrap_or(pin_id.as_str());
        let wire_infos = pin_wires.get(&pin_id).cloned().unwrap_or_default();

        if wire_infos.is_empty() {
            table.rows.push(FlexibleRow {
                cells: vec![
                    pin_name.to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ],
                meta: RowMeta::default(),
            });
        } else {
            for info in wire_infos {
                table.rows.push(FlexibleRow {
                    cells: vec![
                        pin_name.to_string(),
                        info.name,
                        info.description,
                        info.from_label,
                        info.to_label,
                    ],
                    meta: RowMeta::default(),
                });
            }
        }
    }

    table
}
