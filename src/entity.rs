use serde::{Deserialize, Serialize};
use std::fmt;

pub type EntityId = u32;

static mut NEXT_ENTITY_ID: u32 = 1;

pub fn create_entity_id() -> EntityId {
    unsafe {
        let id = NEXT_ENTITY_ID;
        NEXT_ENTITY_ID += 1;
        id
    }
}

pub fn reset_entity_id_counter(start: u32) {
    unsafe {
        NEXT_ENTITY_ID = start;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: create_entity_id(),
        }
    }

    pub fn with_id(id: EntityId) -> Self {
        Self { id }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.id)
    }
}
