use std::collections::HashMap;
use crate::entity::{Entity, EntityId, create_entity_id};
use crate::component::{Component, ComponentStore, ComponentId};
use crate::query::{Query, QueryBuilder, QueryCache, QueryDescriptor};
use serde::{Serialize, Deserialize};

pub struct World {
    pub(crate) entities: HashMap<EntityId, Entity>,
    pub(crate) component_store: ComponentStore,
    pub(crate) query_cache: QueryCache,
    time: f64,
    fixed_time: f64,
    accumulator: f64,
    running: bool,
    paused: bool,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            component_store: ComponentStore::new(),
            query_cache: QueryCache::new(),
            time: 0.0,
            fixed_time: 0.0,
            accumulator: 0.0,
            running: false,
            paused: false,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new();
        self.entities.insert(entity.id, entity);
        self.query_cache.mark_all_dirty();
        entity
    }

    pub fn create_entity_with_id(&mut self, id: EntityId) -> Entity {
        if self.entities.contains_key(&id) {
            panic!("Entity with id {} already exists", id);
        }
        let entity = Entity::with_id(id);
        self.entities.insert(id, entity);
        self.query_cache.mark_all_dirty();
        entity
    }

    pub fn destroy_entity(&mut self, entity_id: EntityId) -> bool {
        if self.entities.remove(&entity_id).is_some() {
            self.component_store.remove_all_components(entity_id);
            self.query_cache.mark_all_dirty();
            return true;
        }
        false
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Option<&Entity> {
        self.entities.get(&entity_id)
    }

    pub fn has_entity(&self, entity_id: EntityId) -> bool {
        self.entities.contains_key(&entity_id)
    }

    pub fn get_all_entities(&self) -> Vec<&Entity> {
        self.entities.values().collect()
    }

    pub fn add_component(&mut self, entity_id: EntityId, component: Box<dyn Component>) {
        if !self.entities.contains_key(&entity_id) {
            panic!("Entity {} does not exist", entity_id);
        }
        let component_id = component.component_id();
        self.component_store.add(entity_id, component);
        self.query_cache.mark_dirty_for_component(&component_id);
    }

    pub fn remove_component(&mut self, entity_id: EntityId, component_id: &str) -> bool {
        let removed = self.component_store.remove(entity_id, component_id);
        if removed {
            self.query_cache.mark_dirty_for_component(component_id);
        }
        removed
    }

    pub fn get_component<T: Component>(&self, entity_id: EntityId) -> Option<&T> {
        self.component_store.get::<T>(entity_id)
    }

    pub fn get_all_components(&self, entity_id: EntityId) -> Vec<&Box<dyn Component>> {
        self.component_store.get_all(entity_id)
    }

    pub fn has_component(&self, entity_id: EntityId, component_id: &str) -> bool {
        self.component_store.has(entity_id, component_id)
    }

    pub fn query(&mut self, descriptor: QueryDescriptor) -> std::collections::HashSet<EntityId> {
        let query = self.query_cache.get(descriptor);
        query.execute(&self.component_store)
    }

    pub fn query_builder(&self) -> QueryBuilder {
        QueryBuilder::new()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.component_store.clear();
        self.query_cache.clear();
    }
}
