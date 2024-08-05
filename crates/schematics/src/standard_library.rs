use crate::{Schematic, SchematicError};

#[allow(dead_code)]
mod resistor {
    use crate::{part::PartBuilder, Schematic, SchematicError};
    pub const RESISTANCE_KEY: &str = "resistance";

    pub fn register(schematic: &mut Schematic) -> Result<(), SchematicError> {
        let part = PartBuilder::default()
            .name("Resistor".to_string())
            .port("1".into(), "p1".into())
            .port("2".into(), "p2".into())
            .build()?;

        schematic.add_part(part)?;
        Ok(())
    }
}

#[allow(dead_code)]
mod capacitor {
    use crate::{part::PartBuilder, Schematic, SchematicError};
    pub const CAPACITANCE_KEY: &str = "capacitance";

    pub fn register(schematic: &mut Schematic) -> Result<(), SchematicError> {
        let part = PartBuilder::default()
            .name("Capacitor".to_string())
            .port("1".into(), "p1".into())
            .port("2".into(), "p2".into())
            .build()?;

        schematic.add_part(part)?;
        Ok(())
    }
}

impl Schematic {
    pub fn register_standard_library(&mut self) -> Result<(), SchematicError> {
        resistor::register(self)?;
        capacitor::register(self)?;
        Ok(())
    }
}

#[test]
fn test_stl() {
    let mut schematic = Schematic::new();
    schematic.register_standard_library().unwrap();
}
