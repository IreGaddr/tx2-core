use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use tsify::Tsify;
use crate::entity::{Entity, EntityId};
use crate::world::World;
use crate::component::{Component, ComponentId};
use std::collections::HashMap;

#[wasm_bindgen(start)]
pub fn main_wasm() {
    #[cfg(feature = "wasm")]
    {
        console_error_panic_hook::set_once();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEntityId(pub u32);

impl From<EntityId> for WasmEntityId {
    fn from(id: EntityId) -> Self {
        WasmEntityId(id)
    }
}

impl From<WasmEntityId> for EntityId {
    fn from(wasm_id: WasmEntityId) -> Self {
        wasm_id.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmComponentData {
    pub id: String,
    pub data: JsValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEntity {
    pub id: u32,
}

impl From<Entity> for WasmEntity {
    fn from(entity: Entity) -> Self {
        WasmEntity { id: entity.id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmWorldSnapshot {
    pub entities: Vec<WasmSerializedEntity>,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmSerializedEntity {
    pub id: u32,
    pub components: Vec<WasmSerializedComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmSerializedComponent {
    pub id: String,
    pub data: serde_json::Value,
}

#[wasm_bindgen]
pub struct WasmWorld {
    inner: World,
}

#[wasm_bindgen]
impl WasmWorld {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmWorld {
        WasmWorld {
            inner: World::new(),
        }
    }

    #[wasm_bindgen(js_name = createEntity)]
    pub fn create_entity(&mut self) -> WasmEntity {
        let entity = self.inner.create_entity();
        WasmEntity::from(entity)
    }

    #[wasm_bindgen(js_name = createEntityWithId)]
    pub fn create_entity_with_id(&mut self, id: u32) -> Result<WasmEntity, JsValue> {
        let entity = self.inner.create_entity_with_id(id);
        Ok(WasmEntity::from(entity))
    }

    #[wasm_bindgen(js_name = destroyEntity)]
    pub fn destroy_entity(&mut self, entity_id: u32) -> bool {
        self.inner.destroy_entity(entity_id)
    }

    #[wasm_bindgen(js_name = hasEntity)]
    pub fn has_entity(&self, entity_id: u32) -> bool {
        self.inner.has_entity(entity_id)
    }

    #[wasm_bindgen(js_name = getAllEntities)]
    pub fn get_all_entities(&self) -> Result<JsValue, JsValue> {
        let entities: Vec<WasmEntity> = self.inner
            .get_all_entities()
            .into_iter()
            .map(|e| WasmEntity::from(*e))
            .collect();

        serde_wasm_bindgen::to_value(&entities)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = addComponent)]
    pub fn add_component(&mut self, entity_id: u32, component_id: String, data: JsValue) -> Result<(), JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let component = DynamicComponent {
            id: component_id.clone(),
            data: json_value,
        };

        self.inner.add_component(entity_id, Box::new(component));
        Ok(())
    }

    #[wasm_bindgen(js_name = removeComponent)]
    pub fn remove_component(&mut self, entity_id: u32, component_id: String) -> bool {
        self.inner.remove_component(entity_id, &component_id)
    }

    #[wasm_bindgen(js_name = hasComponent)]
    pub fn has_component(&self, entity_id: u32, component_id: String) -> bool {
        self.inner.has_component(entity_id, &component_id)
    }

    #[wasm_bindgen(js_name = getComponent)]
    pub fn get_component(&self, entity_id: u32, component_id: String) -> Result<JsValue, JsValue> {
        let components = self.inner.get_all_components(entity_id);

        for component in components {
            if component.component_id() == component_id {
                let json_value = component.to_json();
                return serde_wasm_bindgen::to_value(&json_value)
                    .map_err(|e| JsValue::from_str(&e.to_string()));
            }
        }

        Ok(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getAllComponents)]
    pub fn get_all_components(&self, entity_id: u32) -> Result<JsValue, JsValue> {
        let components = self.inner.get_all_components(entity_id);

        let serialized: Vec<WasmSerializedComponent> = components
            .into_iter()
            .map(|c| WasmSerializedComponent {
                id: c.component_id(),
                data: c.to_json(),
            })
            .collect();

        serde_wasm_bindgen::to_value(&serialized)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = createSnapshot)]
    pub fn create_snapshot(&self) -> Result<JsValue, JsValue> {
        let entities = self.inner.get_all_entities();

        let serialized_entities: Vec<WasmSerializedEntity> = entities
            .into_iter()
            .map(|entity| {
                let components = self.inner.get_all_components(entity.id);
                let serialized_components: Vec<WasmSerializedComponent> = components
                    .into_iter()
                    .map(|c| WasmSerializedComponent {
                        id: c.component_id(),
                        data: c.to_json(),
                    })
                    .collect();

                WasmSerializedEntity {
                    id: entity.id,
                    components: serialized_components,
                }
            })
            .collect();

        let snapshot = WasmWorldSnapshot {
            entities: serialized_entities,
            timestamp: 0.0,
        };

        serde_wasm_bindgen::to_value(&snapshot)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = restoreFromSnapshot)]
    pub fn restore_from_snapshot(&mut self, snapshot: JsValue) -> Result<(), JsValue> {
        let snapshot: WasmWorldSnapshot = serde_wasm_bindgen::from_value(snapshot)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.inner.clear();

        for entity in snapshot.entities {
            self.inner.create_entity_with_id(entity.id);

            for component in entity.components {
                let dynamic_component = DynamicComponent {
                    id: component.id.clone(),
                    data: component.data,
                };
                self.inner.add_component(entity.id, Box::new(dynamic_component));
            }
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = clear)]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[wasm_bindgen(js_name = query)]
    pub fn query(&mut self, include_components: Vec<String>, exclude_components: Vec<String>) -> Result<JsValue, JsValue> {
        use crate::query::QueryDescriptor;

        let descriptor = QueryDescriptor {
            include: include_components.into_iter().collect(),
            exclude: exclude_components.into_iter().collect(),
        };

        let result_set = self.inner.query(descriptor);
        let entity_ids: Vec<u32> = result_set.into_iter().collect();

        serde_wasm_bindgen::to_value(&entity_ids)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DynamicComponent {
    id: String,
    data: serde_json::Value,
}

impl Component for DynamicComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn component_id(&self) -> ComponentId {
        self.id.clone()
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> serde_json::Value {
        self.data.clone()
    }
}

#[wasm_bindgen]
pub fn get_wasm_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn benchmark_entity_creation(count: u32) -> f64 {
    use web_sys::window;

    let performance = window()
        .expect("no global `window` exists")
        .performance()
        .expect("should have performance");

    let start = performance.now();

    let mut world = World::new();
    for _ in 0..count {
        world.create_entity();
    }

    let end = performance.now();
    end - start
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_world_creation() {
        let world = WasmWorld::new();
        assert!(true);
    }

    #[wasm_bindgen_test]
    fn test_entity_creation() {
        let mut world = WasmWorld::new();
        let entity = world.create_entity();
        assert!(entity.id > 0);
    }

    #[wasm_bindgen_test]
    fn test_component_operations() {
        let mut world = WasmWorld::new();
        let entity = world.create_entity();

        let data = serde_wasm_bindgen::to_value(&serde_json::json!({
            "x": 10.0,
            "y": 20.0
        })).unwrap();

        world.add_component(entity.id, "Position".to_string(), data).unwrap();
        assert!(world.has_component(entity.id, "Position".to_string()));
    }
}
