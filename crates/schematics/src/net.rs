use std::{
    cell::RefCell,
    collections::HashSet,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{component::ComponentRef, part::PortRef, SchematicError};

#[derive(Debug, Clone)]
pub struct NetRef(pub Rc<RefCell<Net>>);

impl NetRef {
    pub fn new(net: Net) -> NetRef {
        NetRef(Rc::new(RefCell::new(net)))
    }

    pub fn as_deref(&self) -> impl Deref<Target = Net> + '_ {
        self.0.borrow()
    }

    pub fn as_deref_mut(&self) -> impl DerefMut<Target = Net> + '_ {
        self.0.borrow_mut()
    }

    pub fn name(&self) -> String {
        self.as_deref().name.clone()
    }
}

impl Hash for NetRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl PartialEq for NetRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for NetRef {}

#[derive(Debug, Clone)]
pub enum NetType {
    Unknown,
    Power,
    Ground,
    Digital,
    Analog,
}

#[derive(Debug, Builder)]
#[builder(build_fn(error = "SchematicError"))]
pub struct Net {
    pub name: String,
    #[builder(default = "NetType::Unknown")]
    pub net_type: NetType,
    #[builder(default = "HashSet::new()")]
    pub connections: HashSet<(ComponentRef, PortRef)>,
}

impl Net {
    pub fn connect(&mut self, component: ComponentRef, port: PortRef) {
        self.connections.insert((component, port));
    }
}
