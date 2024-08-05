use kicad_format::{parse_netlist_file, KiCadParseError};
use schematics::{
    component::ComponentBuilder, net::NetBuilder, part::PartBuilder, Schematic, SchematicError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchematicImportError {
    #[error("Failed to parse Kicad file: {0}")]
    ParserError(#[from] KiCadParseError),
    #[error("Failed to register standard library: {0}")]
    SchematicError(#[from] SchematicError),
    #[error("Failed to interpret Kicad netlist: {0}")]
    InterpretationError(String),
}

/// Import a Kicad netlist file into a Schematic.
pub fn schematics_from_kicad_netlist(file: &str) -> Result<Schematic, SchematicImportError> {
    let mut schematic = Schematic::new();
    schematic.register_standard_library()?;

    let netlist = parse_netlist_file(file)?;

    // Register a Part for each library part.
    for netlist_part in netlist.libparts.iter() {
        let pins: Vec<(String, String)> = netlist_part
            .pins
            .as_ref()
            .unwrap_or(&[].to_vec())
            .iter()
            .map(|p| (p.num.clone(), p.name.clone()))
            .collect();

        let mut pb = PartBuilder::default();
        pb.name(netlist_part.part.clone());

        for (num, name) in pins {
            pb.port(num.as_str(), name.as_str());
        }

        for field in netlist_part.fields.iter() {
            pb.metadata(field.name.as_str(), field.value.as_deref().unwrap_or(""));
        }

        let part = pb.build()?;
        schematic.add_part(part)?;
    }

    // Register a Component for each component in the netlist.
    for netlist_component in netlist.components.iter() {
        let mut cb = ComponentBuilder::default();
        cb.name(netlist_component.ref_.clone());

        let partname = netlist_component.libsource.part.clone();
        let part =
            schematic
                .get_part(&partname)
                .ok_or(SchematicImportError::InterpretationError(format!(
                    "Part {} not found",
                    partname
                )))?;

        cb.part(part.clone());

        for property in netlist_component.properties.iter() {
            cb.metadata(
                property.name.as_str(),
                property.value.as_deref().unwrap_or(""),
            );
        }

        let component = cb.build()?;
        schematic.add_component(component)?;
    }

    // Register a Net for each net in the netlist.
    for netlist_net in netlist.nets.iter() {
        let mut nb = NetBuilder::default();
        nb.name(netlist_net.name.clone());
        let net = nb.build()?;
        let net = schematic.add_net(net)?;

        for node in netlist_net.nodes.iter() {
            let component = schematic.get_component(&node.ref_).ok_or(
                SchematicImportError::InterpretationError(format!(
                    "Component {} not found",
                    node.ref_
                )),
            )?;
            let port = component.as_deref().get_port(&node.pin).ok_or(
                SchematicImportError::InterpretationError(format!("Port {} not found", node.pin)),
            )?;

            let component_name = &component.as_deref().name.clone();
            let port_name = &port.as_deref().terminal_identifier.clone();

            schematic.connect(&net.name(), component_name, port_name)?;
        }
    }

    Ok(schematic)
}
