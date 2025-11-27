use std::any::Any;
use std::collections::{HashMap, HashSet};
use crate::entity::EntityId;
use serde::{Serialize, de::DeserializeOwned};

pub type ComponentId = String;

pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn component_id(&self) -> ComponentId;
    fn clone_box(&self) -> Box<dyn Component>;
    fn to_json(&self) -> serde_json::Value;
}

impl<T> Component for T
where
    T: Any + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn component_id(&self) -> ComponentId {
        std::any::type_name::<T>().to_string()
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

pub struct ComponentStore {
    // Map<EntityId, Map<ComponentId, Vec<Box<dyn Component>>>>
    components: HashMap<EntityId, HashMap<ComponentId, Vec<Box<dyn Component>>>>,
    // Map<ComponentId, Set<EntityId>>
    component_index: HashMap<ComponentId, HashSet<EntityId>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            component_index: HashMap::new(),
        }
    }

    pub fn add(&mut self, entity_id: EntityId, component: Box<dyn Component>) {
        let component_id = component.component_id();
        
        let entity_components = self.components.entry(entity_id).or_insert_with(HashMap::new);
        let list = entity_components.entry(component_id.clone()).or_insert_with(Vec::new);
        list.push(component);

        let index = self.component_index.entry(component_id).or_insert_with(HashSet::new);
        index.insert(entity_id);
    }

    pub fn remove(&mut self, entity_id: EntityId, component_id: &str) -> bool {
        if let Some(entity_components) = self.components.get_mut(&entity_id) {
            if entity_components.remove(component_id).is_some() {
                if let Some(index) = self.component_index.get_mut(component_id) {
                    index.remove(&entity_id);
                    if index.is_empty() {
                        self.component_index.remove(component_id);
                    }
                }
                return true;
            }
        }
        false
    }

    pub fn get<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        let component_id = std::any::type_name::<T>().to_string();
        if let Some(entity_components) = self.components.get(&entity_id) {
            if let Some(list) = entity_components.get(&component_id) {
                if let Some(comp) = list.first() {
                    return comp.as_any().downcast_ref::<T>();
                }
            }
        }
        None
    }
    
    pub fn get_all_by_type<T: Component>(&self, entity_id: EntityId) -> Vec<&T> {
        let component_id = std::any::type_name::<T>().to_string();
        let mut result = Vec::new();
        if let Some(entity_components) = self.components.get(&entity_id) {
            if let Some(list) = entity_components.get(&component_id) {
                for comp in list {
                    if let Some(typed) = comp.as_any().downcast_ref::<T>() {
                        result.push(typed);
                    }
                }
            }
        }
        result
    }

    pub fn has(&self, entity_id: EntityId, component_id: &str) -> bool {
        if let Some(entity_components) = self.components.get(&entity_id) {
            return entity_components.contains_key(component_id);
        }
        false
    }

    pub fn get_all(&self, entity_id: EntityId) -> Vec<&Box<dyn Component>> {
        let mut result = Vec::new();
        if let Some(entity_components) = self.components.get(&entity_id) {
            for list in entity_components.values() {
                result.extend(list);
            }
        }
        result
    }

    pub fn get_entities_with_component(&self, component_id: &str) -> HashSet<EntityId> {
        self.component_index.get(component_id).cloned().unwrap_or_default()
    }

    pub fn remove_all_components(&mut self, entity_id: EntityId) {
        if let Some(entity_components) = self.components.remove(&entity_id) {
            for component_id in entity_components.keys() {
                if let Some(index) = self.component_index.get_mut(component_id) {
                    index.remove(&entity_id);
                    if index.is_empty() {
                        self.component_index.remove(component_id);
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.components.clear();
        self.component_index.clear();
    }

    pub fn get_all_entities(&self) -> HashSet<EntityId> {
        self.components.keys().cloned().collect()
    }
}
