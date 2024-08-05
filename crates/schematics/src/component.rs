use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::{
    part::{MetadataKey, PartRef, PortRef},
    SchematicError,
};

#[derive(Debug, Clone)]
pub struct ComponentRef(pub Rc<RefCell<Component>>);

impl ComponentRef {
    pub fn new(component: Component) -> ComponentRef {
        ComponentRef(Rc::new(RefCell::new(component)))
    }

    pub fn as_deref(&self) -> impl Deref<Target = Component> + '_ {
        self.0.borrow()
    }

    pub fn as_deref_mut(&mut self) -> impl DerefMut<Target = Component> + '_ {
        self.0.borrow_mut()
    }
}

impl Hash for ComponentRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl PartialEq for ComponentRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl Eq for ComponentRef {}

#[derive(Debug, Builder)]
#[builder(build_fn(error = "SchematicError"))]
pub struct Component {
    pub name: String,
    pub part: PartRef,
    #[builder(setter(custom), default = "HashMap::new()")]
    pub metadata: HashMap<MetadataKey, String>,
}

impl Component {
    pub fn get_port(&self, name: &str) -> Option<PortRef> {
        self.part.as_deref().get_port(name)
    }
}

impl ComponentBuilder {
    pub fn metadata(&mut self, key: &str, value: &str) -> &mut Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.to_string(), value.to_string());
        self
    }
}
