use std::collections::{HashSet, HashMap};
use crate::component::{ComponentStore, ComponentId};
use crate::entity::EntityId;

#[derive(Debug, Clone)]
pub enum QueryFilter {
    All(Vec<ComponentId>),
    Any(Vec<ComponentId>),
    None(Vec<ComponentId>),
}

#[derive(Default)]
pub struct QueryDescriptor {
    pub all: Vec<ComponentId>,
    pub any: Vec<ComponentId>,
    pub none: Vec<ComponentId>,
}

pub struct Query {
    filters: Vec<QueryFilter>,
    cached_results: Option<HashSet<EntityId>>,
    dirty: bool,
}

impl Query {
    pub fn new(descriptor: QueryDescriptor) -> Self {
        let mut filters = Vec::new();
        if !descriptor.all.is_empty() {
            filters.push(QueryFilter::All(descriptor.all));
        }
        if !descriptor.any.is_empty() {
            filters.push(QueryFilter::Any(descriptor.any));
        }
        if !descriptor.none.is_empty() {
            filters.push(QueryFilter::None(descriptor.none));
        }

        if filters.is_empty() {
            panic!("Query must have at least one filter");
        }

        Self {
            filters,
            cached_results: None,
            dirty: true,
        }
    }

    pub fn matches(&self, entity_id: EntityId, store: &ComponentStore) -> bool {
        for filter in &self.filters {
            match filter {
                QueryFilter::All(components) => {
                    if !components.iter().all(|c| store.has(entity_id, c)) {
                        return false;
                    }
                }
                QueryFilter::Any(components) => {
                    if !components.iter().any(|c| store.has(entity_id, c)) {
                        return false;
                    }
                }
                QueryFilter::None(components) => {
                    if components.iter().any(|c| store.has(entity_id, c)) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn execute(&mut self, store: &ComponentStore) -> HashSet<EntityId> {
        if !self.dirty {
            if let Some(results) = &self.cached_results {
                return results.clone();
            }
        }

        let candidates = self.get_candidate_entities(store);
        let mut results = HashSet::new();

        for entity_id in candidates {
            if self.matches(entity_id, store) {
                results.insert(entity_id);
            }
        }

        self.cached_results = Some(results.clone());
        self.dirty = false;
        results
    }

    fn get_candidate_entities(&self, store: &ComponentStore) -> HashSet<EntityId> {
        let mut candidates: Option<HashSet<EntityId>> = None;

        // Union entities for any-filters
        let any_filters: Vec<&QueryFilter> = self.filters.iter().filter(|f| matches!(f, QueryFilter::Any(_))).collect();
        if !any_filters.is_empty() {
            let mut any_candidates = HashSet::new();
            for filter in any_filters {
                if let QueryFilter::Any(components) = filter {
                    for component_id in components {
                        for entity_id in store.get_entities_with_component(component_id) {
                            any_candidates.insert(entity_id);
                        }
                    }
                }
            }
            candidates = Some(any_candidates);
        }

        // Narrow candidates by all-filters via intersection
        let all_filters: Vec<&QueryFilter> = self.filters.iter().filter(|f| matches!(f, QueryFilter::All(_))).collect();
        for filter in all_filters {
            if let QueryFilter::All(components) = filter {
                for component_id in components {
                    let entities = store.get_entities_with_component(component_id);
                    if let Some(current_candidates) = &mut candidates {
                        current_candidates.retain(|id| entities.contains(id));
                    } else {
                        candidates = Some(entities);
                    }
                }
            }
        }

        candidates.unwrap_or_else(|| store.get_all_entities())
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct QueryBuilder {
    descriptor: QueryDescriptor,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            descriptor: QueryDescriptor::default(),
        }
    }

    pub fn all(mut self, components: Vec<ComponentId>) -> Self {
        self.descriptor.all = components;
        self
    }

    pub fn any(mut self, components: Vec<ComponentId>) -> Self {
        self.descriptor.any = components;
        self
    }

    pub fn none(mut self, components: Vec<ComponentId>) -> Self {
        self.descriptor.none = components;
        self
    }

    pub fn build(self) -> Query {
        Query::new(self.descriptor)
    }
}

pub struct QueryCache {
    queries: HashMap<String, Query>,
}

impl QueryCache {
    pub fn new() -> Self {
        Self {
            queries: HashMap::new(),
        }
    }

    pub fn get(&mut self, descriptor: QueryDescriptor) -> &mut Query {
        let key = self.get_key(&descriptor);
        self.queries.entry(key).or_insert_with(|| Query::new(descriptor))
    }

    fn get_key(&self, descriptor: &QueryDescriptor) -> String {
        let mut parts = Vec::new();
        if !descriptor.all.is_empty() {
            let mut sorted = descriptor.all.clone();
            sorted.sort();
            parts.push(format!("all:{}", sorted.join(",")));
        }
        if !descriptor.any.is_empty() {
            let mut sorted = descriptor.any.clone();
            sorted.sort();
            parts.push(format!("any:{}", sorted.join(",")));
        }
        if !descriptor.none.is_empty() {
            let mut sorted = descriptor.none.clone();
            sorted.sort();
            parts.push(format!("none:{}", sorted.join(",")));
        }
        parts.join("|")
    }

    pub fn mark_all_dirty(&mut self) {
        for query in self.queries.values_mut() {
            query.mark_dirty();
        }
    }

    pub fn mark_dirty_for_component(&mut self, component_id: &str) {
        for (key, query) in self.queries.iter_mut() {
            if key.contains(component_id) {
                query.mark_dirty();
            }
        }
    }

    pub fn clear(&mut self) {
        self.queries.clear();
    }
}
