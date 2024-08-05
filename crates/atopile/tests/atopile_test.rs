use atopile::AtopileExporter;
use insta::assert_snapshot;
use schematics::{ComponentBuilder, Net, NetType, PartBuilder, Schematic};

#[ignore]
#[test]
fn test_export() {
    let mut schematic = Schematic::new();
    schematic.register_standard_library();

    let resistor = PartBuilder::new()
        .name("resistor")
        .pin("1".into(), "P1".into())
        .pin("2".into(), "P2".into())
        .build()
        .register(&mut schematic)
        .unwrap();

    // Build schematic
    let r1 = ComponentBuilder::new()
        .name("r1")
        .part(resistor)
        .build()
        .register(&mut schematic)
        .unwrap();

    let vdd = Net::new("vdd", NetType::Power).register(&mut schematic);
    let gnd = Net::new("gnd", NetType::Ground).register(&mut schematic);

    // Connect nets
    schematic.connect(vdd, r1, "P1".into()).unwrap();
    schematic.connect(gnd, r1, "P2".into()).unwrap();

    // Generate Atopile
    let mut exporter = AtopileExporter::new(&mut schematic);
    let atopile = exporter.export().ok().unwrap();
    assert_snapshot!(atopile, @r###"
    from "generics/resistors.ato" import Resistor
    component resistor:
        signal P1
        signal P2
        P2 ~ pin 2
        P1 ~ pin 1
    module Schematic:
        r1 = new resistor
        signal gnd
        gnd ~ r1.P2
        signal vdd
        vdd ~ r1.P1
    "###);
}
