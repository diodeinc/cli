#[macro_use]
extern crate derive_builder;

use std::collections::HashMap;

pub use std::fmt::Display;

use component::{Component, ComponentRef};
use derive_builder::UninitializedFieldError;
use net::{Net, NetRef};
use part::{Part, PartRef};
use thiserror::Error;

pub mod component;
pub mod edatasheet;
pub mod net;
pub mod part;
pub mod standard_library;

/// `Schematic` encodes the logical representation of an electrical design. It
/// does not contain any visual information to support manual layout or
/// rendering of electrical components, but should be conducive to schematic
/// type-checking and generating artifacts like netlists or Atopile code.
///
/// A schematic contains a library of `Part`s, each of which defines a specific
/// electrical component (e.g. a resistor or an IC). Instances of these parts
/// are represented as `Component`s. The pins on parts are connected together
/// via `Net`s.
#[derive(Debug)]
pub struct Schematic {
    /// A part represents an entry in the "library". It can be a concrete
    /// instantiation of a part (e.g. "NRF52840-QIAA-R"), or a generic part
    /// (e.g. "Capacitor"). Each part has 0 or more ports associated with it.
    parts_by_name: HashMap<String, PartRef>,

    /// A component represents an instantiation of a part in a schematic. It
    /// contains a reference to the part and any associated metadata.
    components_by_name: HashMap<String, ComponentRef>,

    /// A net represents a set of connections between ports on components.
    nets_by_name: HashMap<String, NetRef>,
}

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("Name already exists: {0}")]
    NameAlreadyExists(String),
    #[error("Name not found: {0}")]
    NameNotFound(String),
    #[error("Uninitialized field: {0}")]
    UninitializedField(String),
    #[error("Normalization error: {0}")]
    NormalizationError(#[from] NormalizationError),
}

#[derive(Error, Debug)]
pub enum NormalizationError {
    #[error("Name conflict: {0}")]
    NameConflict(String),
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<UninitializedFieldError> for SchematicError {
    fn from(e: UninitializedFieldError) -> Self {
        Self::UninitializedField(e.field_name().to_string())
    }
}

impl Schematic {
    pub fn new() -> Self {
        Self {
            parts_by_name: HashMap::new(),
            components_by_name: HashMap::new(),
            nets_by_name: HashMap::new(),
        }
    }

    pub fn add_part(&mut self, part: Part) -> Result<PartRef, SchematicError> {
        let name = part.name.clone();
        if self.parts_by_name.contains_key(&name) {
            return Err(SchematicError::NameAlreadyExists(name));
        }
        let part_ref = PartRef::new(part);
        self.parts_by_name.insert(name, part_ref.clone());
        Ok(part_ref)
    }

    pub fn add_component(&mut self, component: Component) -> Result<ComponentRef, SchematicError> {
        let name = component.name.clone();
        if self.components_by_name.contains_key(&name) {
            return Err(SchematicError::NameAlreadyExists(name));
        }
        let component_ref = ComponentRef::new(component);
        self.components_by_name.insert(name, component_ref.clone());
        Ok(component_ref)
    }

    pub fn add_net(&mut self, net: Net) -> Result<NetRef, SchematicError> {
        let name = net.name.clone();
        if self.nets_by_name.contains_key(&name) {
            return Err(SchematicError::NameAlreadyExists(name));
        }
        let net_ref = NetRef::new(net);
        self.nets_by_name.insert(name, net_ref.clone());
        Ok(net_ref)
    }

    pub fn get_part(&self, name: &str) -> Option<PartRef> {
        self.parts_by_name.get(name).map(|r| r.clone())
    }

    pub fn get_component(&self, name: &str) -> Option<ComponentRef> {
        self.components_by_name.get(name).map(|r| r.clone())
    }

    pub fn get_net(&self, name: &str) -> Option<NetRef> {
        self.nets_by_name.get(name).map(|r| r.clone())
    }

    pub fn parts_iter(&self) -> impl Iterator<Item = &PartRef> {
        self.parts_by_name.values()
    }

    pub fn components_iter(&self) -> impl Iterator<Item = &ComponentRef> {
        self.components_by_name.values()
    }

    pub fn nets_iter(&self) -> impl Iterator<Item = &NetRef> {
        self.nets_by_name.values()
    }

    pub fn connect(
        &mut self,
        net_name: &str,
        component_name: &str,
        terminal_identifier: &str,
    ) -> Result<(), SchematicError> {
        let component = self
            .components_by_name
            .get(component_name)
            .ok_or(SchematicError::NameNotFound(component_name.to_string()))?
            .clone();

        let port = component
            .as_deref()
            .part
            .as_deref()
            .get_port(terminal_identifier)
            .ok_or(SchematicError::NameNotFound(
                terminal_identifier.to_string(),
            ))?;

        self.nets_by_name
            .get_mut(net_name)
            .ok_or(SchematicError::NameNotFound(net_name.to_string()))?
            .as_deref_mut()
            .connect(component, port);

        Ok(())
    }
}

pub trait Normalizer {
    fn normalize_component_name(&self, name: &str) -> Result<String, NormalizationError>;
    fn normalize_net_name(&self, name: &str) -> Result<String, NormalizationError>;
    fn normalize_part_name(&self, name: &str) -> Result<String, NormalizationError>;
    fn normalize_port_name(
        &self,
        pin_name: &str,
        signal_name: &str,
    ) -> Result<String, NormalizationError>;
}

impl Schematic {
    fn build_normalize_map<'a>(
        &self,
        iter: impl Iterator<Item = &'a String>,
        normalizer_fn: impl Fn(&str) -> Result<String, NormalizationError>,
    ) -> Result<HashMap<String, String>, SchematicError> {
        let mut new_names = HashMap::new();
        for name in iter {
            let new_name = normalizer_fn(name)?;
            if new_names.contains_key(&new_name) {
                return Err(NormalizationError::NameConflict(new_name).into());
            }
            new_names.insert(name.clone(), new_name);
        }
        Ok(new_names)
    }

    pub fn normalize(&mut self, normalizer: impl Normalizer) -> Result<(), SchematicError> {
        // Build a mapping of old to normalized names for components, nets, and
        // parts.
        let new_component_names = self
            .build_normalize_map(self.components_by_name.keys(), |name| {
                normalizer.normalize_component_name(name)
            })?;

        let new_nets = self.build_normalize_map(self.nets_by_name.keys(), |name| {
            normalizer.normalize_net_name(name)
        })?;

        let new_parts = self.build_normalize_map(self.parts_by_name.keys(), |name| {
            normalizer.normalize_part_name(name)
        })?;

        // For each part, build a mapping of old to normalized port names.
        let mut part_pins: HashMap<String, HashMap<String, String>> = HashMap::new();
        for (name, part) in self.parts_by_name.iter() {
            let new_name = new_parts.get(name).unwrap();
            let part = part.as_deref();
            for (terminal_identifier, port) in part.ports_by_terminal_identifier.iter() {
                let new_port_name =
                    normalizer.normalize_port_name(terminal_identifier, &port.as_deref().signal)?;
                part_pins
                    .entry(new_name.clone())
                    .or_insert(HashMap::new())
                    .insert(terminal_identifier.to_string(), new_port_name);
            }
        }

        // If we can normalize everything without error, let's apply the
        // normalization to the schematic.
        self.components_by_name = self
            .components_by_name
            .iter_mut()
            .map(|(name, component)| {
                let new_name = new_component_names[name].clone();
                component.as_deref_mut().name = new_name.clone();
                (new_name, component.clone())
            })
            .collect();

        self.nets_by_name = self
            .nets_by_name
            .iter_mut()
            .map(|(name, net)| {
                let new_name = new_nets[name].clone();
                net.as_deref_mut().name = new_name.clone();
                (new_name, net.clone())
            })
            .collect();

        self.parts_by_name = self
            .parts_by_name
            .iter_mut()
            .map(|(name, part)| {
                let new_name = new_parts[name].clone();
                part.as_deref_mut().name = new_name.clone();

                for (terminal_identifier, port) in
                    part.as_deref_mut().ports_by_terminal_identifier.iter_mut()
                {
                    let new_port_name = part_pins[&new_name][terminal_identifier].clone();
                    port.as_deref_mut().signal = new_port_name.clone();
                }

                (new_name, part.clone())
            })
            .collect();

        Ok(())
    }
}
