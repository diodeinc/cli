// use schematics::{ComponentBuilder, Net, NetType, PartBuilder, Schematic};

// #[test]
// fn schematics_test() {
//     let mut schematic = Schematic::new();

//     // Build part library
//     let resistor = PartBuilder::new()
//         .name("resistor")
//         .pin("1".into(), "P1".into())
//         .pin("2".into(), "P2".into())
//         .build()
//         .register(&mut schematic)
//         .unwrap();

//     // Build schematic
//     let r1 = ComponentBuilder::new()
//         .name("r1")
//         .part(resistor)
//         .build()
//         .register(&mut schematic)
//         .unwrap();

//     let vdd = Net::new("vdd", NetType::Power).register(&mut schematic);
//     let gnd = Net::new("gnd", NetType::Ground).register(&mut schematic);

//     // Connect nets
//     schematic.connect(vdd, r1, "P1").unwrap();
//     schematic.connect(gnd, r1, "P2").unwrap();

//     assert_eq!(schematic.part(resistor).unwrap().id, resistor);
//     assert_eq!(schematic.component(r1).unwrap().id, r1);
//     assert_eq!(schematic.net(vdd).unwrap().id, vdd);
// }
