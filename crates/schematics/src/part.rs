use crate::SchematicError;
use derive_builder::Builder;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[cfg(feature = "edatasheet")]
use crate::edatasheet;

pub type MetadataKey = String;

pub const SHEET_NAME_KEY: &str = "sheet_name";
pub const FOOTPRINT_KEY: &str = "footprint";
pub const MPN_KEY: &str = "mpn";

#[derive(Debug, Clone)]
pub struct PartRef(pub Rc<RefCell<Part>>);

impl PartRef {
    pub fn new(part: Part) -> Self {
        let part = Rc::new(RefCell::new(part));
        Self(part)
    }

    pub fn as_deref(&self) -> impl Deref<Target = Part> + '_ {
        self.0.borrow()
    }

    pub fn as_deref_mut(&mut self) -> impl DerefMut<Target = Part> + '_ {
        self.0.borrow_mut()
    }
}

impl Hash for PartRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash by address of the inner Rc
        std::ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl PartialEq for PartRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for PartRef {}

/// `Part` represents an electronic part, e.g. a resistor or an IC.
#[derive(Debug, Builder)]
#[builder(build_fn(error = "SchematicError"))]
pub struct Part {
    pub name: String,
    #[builder(setter(custom), default = "HashMap::new()")]
    pub ports_by_terminal_identifier: HashMap<String, PortRef>,
    #[builder(default = "None")]
    #[cfg(feature = "edatasheet")]
    pub datasheet: Option<edatasheet::Component>,
    #[builder(default = "None")]
    pub datasheet_url: Option<String>,
    #[builder(setter(custom), default = "HashMap::new()")]
    pub metadata: HashMap<MetadataKey, String>,
}

impl Part {
    /// Returns a reference to the port with the given terminal identifier, if
    /// it exists.
    pub fn get_port(&self, terminal_identifier: &str) -> Option<PortRef> {
        self.ports_by_terminal_identifier
            .get(terminal_identifier)
            .cloned()
    }
}

impl PartBuilder {
    pub fn port(&mut self, terminal_identifier: &str, signal: &str) -> &mut Self {
        let port = Port::new(terminal_identifier, signal);
        let ports = self
            .ports_by_terminal_identifier
            .get_or_insert_with(HashMap::new);
        ports.insert(terminal_identifier.to_string(), PortRef::new(port));
        self
    }

    pub fn metadata(&mut self, key: &str, value: &str) -> &mut Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct PortRef(pub Rc<RefCell<Port>>);

impl PortRef {
    pub fn new(port: Port) -> Self {
        let port = Rc::new(RefCell::new(port));
        Self(port)
    }

    pub fn as_deref(&self) -> impl Deref<Target = Port> + '_ {
        self.0.borrow()
    }

    pub fn as_deref_mut(&mut self) -> impl DerefMut<Target = Port> + '_ {
        self.0.borrow_mut()
    }
}

impl Hash for PortRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl PartialEq for PortRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for PortRef {}

#[derive(Debug, Builder, Clone)]
pub struct Port {
    pub terminal_identifier: String,
    pub signal: String,
}

impl Port {
    pub fn new(terminal_identifier: &str, signal: &str) -> Self {
        Self {
            terminal_identifier: terminal_identifier.to_string(),
            signal: signal.to_string(),
        }
    }
}
