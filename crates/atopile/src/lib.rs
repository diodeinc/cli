mod normalizer;
mod writer;

use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use natord::compare;
use schematics::{component::ComponentRef, part::PartRef, Normalizer, Schematic, SchematicError};
use thiserror::Error;
use writer::AtopileWriter;

pub use normalizer::AtopileNormalizer;

#[derive(Error, Debug)]
pub enum AtopileError {
    #[error("Schematic error: {0}")]
    SchematicError(#[from] SchematicError),

    #[error("FS error: {0}")]
    FsError(#[from] std::io::Error),

    #[error("Name collision: {0}")]
    NameCollisionError(String),
}

pub struct AtopileProject {
    /// The name of the project.
    name: String,

    /// A mapping from filename to the file.
    files_by_name: HashMap<String, AtopileFile>,

    /// A mapping from symbol name to symbol.
    symbols_by_name: HashMap<String, AtopileSymbol>,

    /// A mapping from symbol name to the filename that defines it.
    symbol_name_to_file_name: HashMap<String, String>,
}

impl AtopileProject {
    fn find_or_create_file(&mut self, filename: &str) -> &mut AtopileFile {
        self.files_by_name
            .entry(filename.to_string())
            .or_insert_with(|| AtopileFile {
                filename: filename.to_string(),
                symbol_names: vec![],
            })
    }

    fn find_module(&mut self, module_name: &str) -> Option<&mut AtopileModule> {
        match self.symbols_by_name.get_mut(module_name) {
            Some(AtopileSymbol::Module(m)) => Some(m),
            _ => None,
        }
    }

    fn define_symbol(
        &mut self,
        filename: String,
        symbol: AtopileSymbol,
    ) -> Result<&AtopileSymbol, AtopileError> {
        let symbol_name = symbol.name().to_string();

        if self.symbols_by_name.contains_key(&symbol_name) {
            return Err(AtopileError::NameCollisionError(symbol.name().to_string()));
        }

        let file = self.find_or_create_file(&filename);
        file.symbol_names.push(symbol.name().to_string());

        self.symbol_name_to_file_name
            .insert(symbol.name().to_string(), filename.clone());

        self.symbols_by_name.insert(symbol_name.clone(), symbol);

        Ok(&self.symbols_by_name[&symbol_name])
    }

    /// Collect all of the imports for a given file. Returns a list of (filename,
    /// symbol_name) pairs.
    fn collect_imports(&self, filename: &str) -> HashSet<(String, String)> {
        let file = self.files_by_name.get(filename).expect("file not found");

        let mut imports: HashSet<(String, String)> = HashSet::new();
        for symbol_name in &file.symbol_names {
            let symbol = self
                .symbols_by_name
                .get(symbol_name)
                .expect("symbol not found");
            if let AtopileSymbol::Module(m) = symbol {
                for definition in &m.definitions {
                    let file_name = self.symbol_name_to_file_name.get(&definition.symbol_name);
                    if let Some(file_name) = file_name {
                        imports.insert((file_name.clone(), definition.symbol_name.clone()));
                    }
                }
            }
        }

        imports
    }
}

pub struct AtopileFile {
    /// The name of the file.
    filename: String,

    /// The names of the symbols defined in the file.
    symbol_names: Vec<String>,
}

pub enum AtopileSymbol {
    Component(AtopileComponent),
    Module(AtopileModule),
}

impl AtopileSymbol {
    fn name(&self) -> &str {
        match self {
            AtopileSymbol::Component(c) => c.name.as_str(),
            AtopileSymbol::Module(m) => m.name.as_str(),
        }
    }
}

pub struct AtopileComponent {
    /// The name of the Atopile component.
    name: String,

    /// A mapping from signal name to a list of pin terminal identifiers that
    /// the signal maps to.
    signals: HashMap<String, Vec<String>>,

    /// The part from the schematic that is being described.
    part: PartRef,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AtopileDefinition {
    /// The name of the definition.
    /// `**R1** = new Resistor`
    name: String,

    /// The name of the symbol being defined.
    ///
    /// `R1 = new **Resistor**`
    symbol_name: String,

    /// The component that this definition is for.
    component: Option<ComponentRef>,
}

pub struct AtopileModule {
    /// The name of the module.
    name: String,

    /// The items in the module.
    definitions: Vec<AtopileDefinition>,

    /// The nets defined in this module, defined as a map from the net name to
    /// a list of ports it connects.
    nets: HashMap<String, Vec<String>>,
}

impl AtopileProject {
    fn sheet_for_component(&self, component: &ComponentRef) -> String {
        let sheet_name = component
            .as_deref()
            .metadata
            .get("Sheetname")
            .cloned()
            .map(|s| s.to_string())
            .unwrap_or(self.name.clone());

        // Normalize the sheet name.
        AtopileNormalizer::default()
            .normalize_part_name(&sheet_name)
            .expect("failed to normalize sheet name")
    }

    pub fn from_schematic(name: String, schematic: &Schematic) -> Result<Self, AtopileError> {
        // Capitalize the first letter of the project name.
        let name = name.chars().next().unwrap().to_uppercase().to_string() + &name[1..];

        let mut project = Self {
            name: name.clone(),
            files_by_name: HashMap::new(),
            symbols_by_name: HashMap::new(),
            symbol_name_to_file_name: HashMap::new(),
        };

        // Keep track of all of the sheet names we've seen.
        let mut sheet_names = HashSet::<String>::new();

        // Create a library file for each part.
        for part in schematic.parts_iter() {
            let mut signals = HashMap::new();
            for (pin_name, port) in part.as_deref().ports_by_terminal_identifier.iter() {
                signals
                    .entry(port.as_deref().signal.clone())
                    .or_insert_with(Vec::new)
                    .push(pin_name.clone());
            }

            let symbol = AtopileSymbol::Component(AtopileComponent {
                name: part.as_deref().name.clone(),
                signals,
                part: part.clone(),
            });

            project.define_symbol(format!("library/{}.ato", part.as_deref().name), symbol)?;
        }

        // Creaet a module for each sheet, and instantiate each component in the
        // sheet.
        for component in schematic.components_iter() {
            // Use the sheet name as the module name.
            let sheet_name = project.sheet_for_component(component);
            sheet_names.insert(sheet_name.clone());

            // Create a module for the sheet if we don't have one.
            if project.find_module(&sheet_name).is_none() {
                let module = AtopileModule {
                    name: sheet_name.to_string(),
                    definitions: vec![],
                    nets: HashMap::new(),
                };
                project
                    .define_symbol(format!("{}.ato", sheet_name), AtopileSymbol::Module(module))?;
            }

            // Add a definition for the component.
            let module = project.find_module(&sheet_name).expect("module not found");
            module.definitions.push(AtopileDefinition {
                name: component.as_deref().name.clone(),
                symbol_name: component.as_deref().part.as_deref().name.clone(),
                component: Some(component.clone()),
            });
        }

        // Create a module for the root.
        project.define_symbol(
            format!("{}.ato", project.name.to_lowercase()),
            AtopileSymbol::Module(AtopileModule {
                name: project.name.clone(),
                definitions: vec![],
                nets: HashMap::new(),
            }),
        )?;

        // Parse the nets, and put them in a sheet-specific module or the root
        // module, depending on how far they spean.
        for net in schematic.nets_iter() {
            let net_name = net.name();
            let sheet_names = net
                .as_deref()
                .connections
                .iter()
                .map(|(c, _)| project.sheet_for_component(c))
                .collect::<Vec<String>>();

            // If all of the components in the net are in the same sheet, we'll
            // put the net in that module. Otherwise, we'll put it in the root.
            let place_in_root =
                sheet_names.len() == 0 || !sheet_names.iter().all(|s| s == &sheet_names[0]);
            let net_module = if place_in_root {
                &project.name.clone()
            } else {
                &sheet_names[0]
            };

            let component_to_sheet = net
                .as_deref()
                .connections
                .iter()
                .map(|(c, _)| (c.clone(), project.sheet_for_component(c)))
                .collect::<HashMap<_, _>>();

            let module = project.find_module(net_module).expect("module not found");

            for (component, port) in net.as_deref().connections.iter() {
                let connections = module.nets.entry(net_name.clone()).or_insert_with(Vec::new);

                if place_in_root {
                    connections.push(format!(
                        "{}.{}.{}",
                        component_to_sheet[component],
                        component.as_deref().name,
                        port.as_deref().signal
                    ));
                } else {
                    connections.push(format!(
                        "{}.{}",
                        component.as_deref().name,
                        port.as_deref().signal
                    ));
                }
            }
        }

        // Finally, add the sheet module definitions to the root.
        let root_module = project.find_module(&name).expect("root module not found");
        for sheet_name in sheet_names.iter() {
            root_module.definitions.push(AtopileDefinition {
                name: sheet_name.to_string(),
                symbol_name: sheet_name.to_string(),
                component: None,
            });
        }

        Ok(project)
    }

    pub fn generate_to_directory(
        &self,
        output_dir: &std::path::PathBuf,
    ) -> Result<(), AtopileError> {
        for (filename, file) in &self.files_by_name {
            let file_path = output_dir.join("elec").join("src").join(filename);
            self.write_file(&file, &file_path)?;
        }

        Ok(())
    }

    fn write_file(
        &self,
        atopile_file: &AtopileFile,
        path: &std::path::PathBuf,
    ) -> Result<(), AtopileError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::File::create(path)?;
        let mut writer = AtopileWriter::new(file);

        let mut imports: Vec<_> = self
            .collect_imports(&atopile_file.filename)
            .into_iter()
            .collect();
        imports.sort_by(|a, b| compare(&a.0, &b.0));

        for (filename, symbol_name) in imports.iter() {
            writer.write_line(&format!("from \"{}\" import {}", filename, symbol_name))?;
        }

        if imports.len() > 0 {
            writer.write_line("")?;
        }

        let mut sorted_symbol_names = atopile_file.symbol_names.clone();
        sorted_symbol_names.sort();

        for symbol_name in sorted_symbol_names.iter() {
            let symbol = self.symbols_by_name.get(symbol_name).unwrap();
            match symbol {
                AtopileSymbol::Component(c) => self.write_component(c, &mut writer)?,
                AtopileSymbol::Module(m) => self.write_module(m, &mut writer)?,
            }
        }

        Ok(())
    }

    fn write_component<T: Write>(
        &self,
        component: &AtopileComponent,
        writer: &mut AtopileWriter<T>,
    ) -> Result<(), AtopileError> {
        writer.start_block(&format!("component {}:", component.name))?;
        let mut sorted_signal_names = component.signals.keys().collect::<Vec<&String>>();
        sorted_signal_names.sort_by(|a, b| compare(a, b));
        for signal_name in sorted_signal_names.iter() {
            writer.write_line(&format!("signal {}", signal_name))?;
            let mut sorted_pin_names = component.signals[*signal_name].clone();
            sorted_pin_names.sort_by(|a, b| compare(a, b));
            for pin_name in sorted_pin_names.iter() {
                writer.write_line(&format!("{} ~ pin {}", signal_name, pin_name))?;
            }
            writer.write_line("")?;
        }

        let mpn = component.part.as_deref().metadata.get("MPN").cloned();
        let footprint = component.part.as_deref().metadata.get("Footprint").cloned();

        if let Some(mpn) = mpn {
            writer.write_line(&format!("mpn = \"{}\"", mpn))?;
        }

        if let Some(footprint) = footprint {
            writer.write_line(&format!("footprint = \"{}\"", footprint))?;
        }

        writer.end_block()?;
        Ok(())
    }

    fn write_module<T: Write>(
        &self,
        module: &AtopileModule,
        writer: &mut AtopileWriter<T>,
    ) -> Result<(), AtopileError> {
        writer.start_block(&format!("module {}:", module.name))?;
        let mut sorted_definition_names = module.definitions.clone();
        sorted_definition_names.sort_by(|a, b| compare(&a.name, &b.name));
        for definition in sorted_definition_names.iter() {
            writer.write_line(&format!(
                "{} = new {}",
                definition.name, definition.symbol_name
            ))?;

            if definition.component.is_some() {
                writer.write_line(&format!(
                    "{}.designator = \"{}\"",
                    definition.name, definition.name
                ))?;
            }

            writer.ensure_break()?;
        }

        let mut sorted_net_names = module.nets.keys().collect::<Vec<&String>>();
        sorted_net_names.sort_by(|a, b| compare(a, b));
        for net_name in sorted_net_names.iter() {
            let mut sorted_ports: Vec<_> = module.nets[*net_name].iter().cloned().collect();
            sorted_ports.sort();
            sorted_ports.dedup();

            if sorted_ports.len() < 2 {
                // Don't bother describing unused nets.
                continue;
            }

            writer.write_line(&format!("signal {}", net_name))?;
            for port in sorted_ports.iter() {
                writer.write_line(&format!("{} ~ {}", net_name, port))?;
            }

            writer.write_line("")?;
        }
        writer.end_block()?;
        Ok(())
    }
}
