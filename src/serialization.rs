use serde::{Deserialize, Serialize};
use crate::entity::EntityId;
use crate::component::ComponentId;
use crate::world::World;
use tx2_link::{
    SerializedComponent, SerializedEntity, WorldSnapshot, Delta, DeltaChange,
    protocol::ComponentData,
};

pub struct Serializer;

impl Serializer {
    pub fn create_snapshot(world: &World) -> WorldSnapshot {
        let mut entities = Vec::new();
        
        for entity in world.get_all_entities() {
            let mut serialized_components = Vec::new();
            for component in world.get_all_components(entity.id) {
                serialized_components.push(SerializedComponent {
                    id: component.component_id(),
                    data: ComponentData::from_json_value(component.to_json()),
                });
            }
            
            entities.push(SerializedEntity {
                id: entity.id,
                components: serialized_components,
            });
        }

        WorldSnapshot {
            entities,
            timestamp: 0.0,
            version: "1.0.0".to_string(),
        }
    }
}

pub struct DeltaCompressor {
    inner: tx2_link::DeltaCompressor,
}

impl DeltaCompressor {
    pub fn new() -> Self {
        Self {
            inner: tx2_link::DeltaCompressor::new(),
        }
    }

    pub fn create_delta(&mut self, world: &World) -> Delta {
        let snapshot = Serializer::create_snapshot(world);
        self.inner.create_delta(snapshot)
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
