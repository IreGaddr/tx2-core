use std::collections::{HashMap, HashSet};
use crate::world::World;
use crate::error::{SystemErrorContext, SystemErrorHandler, SystemErrorStrategy, default_error_handler};
use std::sync::{Arc, Mutex};

pub type SystemId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemPhase {
    Init,
    FixedUpdate,
    Update,
    LateUpdate,
    Cleanup,
}

pub struct SystemContext<'a> {
    pub world: &'a mut World,
    pub delta_time: f64,
    pub time: f64,
    pub phase: SystemPhase,
}

pub trait SystemFn: Send + Sync {
    fn run(&mut self, ctx: SystemContext) -> Result<(), String>;
}

impl<F> SystemFn for F
where
    F: FnMut(SystemContext) -> Result<(), String> + Send + Sync,
{
    fn run(&mut self, ctx: SystemContext) -> Result<(), String> {
        (self)(ctx)
    }
}

// Allow systems that don't return Result (always succeed)
pub trait InfallibleSystemFn: Send + Sync {
    fn run_infallible(&mut self, ctx: SystemContext);
}

impl<F> InfallibleSystemFn for F
where
    F: FnMut(SystemContext) + Send + Sync,
{
    fn run_infallible(&mut self, ctx: SystemContext) {
        (self)(ctx);
    }
}

// Wrapper to unify both
struct SystemFnWrapper {
    inner: Box<dyn SystemFn>,
}

impl SystemFnWrapper {
    fn from_fallible(f: Box<dyn SystemFn>) -> Self {
        Self { inner: f }
    }
    
    // We can't easily wrap the infallible one without a closure that returns Ok,
    // but Box<dyn Fn> is hard to adapt.
    // Let's change SystemFn to always return Result, and provide a helper for infallible systems.
}

pub struct System {
    pub id: SystemId,
    pub name: String,
    pub phases: HashSet<SystemPhase>,
    pub priority: i32,
    pub run_before: HashSet<SystemId>,
    pub run_after: HashSet<SystemId>,
    pub enabled: bool,
    pub consecutive_failures: u32,
    pub on_error: Option<SystemErrorHandler>,
    fn_ptr: Box<dyn SystemFn>,
}

impl System {
    pub fn new(
        id: SystemId,
        name: String,
        phases: HashSet<SystemPhase>,
        priority: i32,
        run_before: HashSet<SystemId>,
        run_after: HashSet<SystemId>,
        func: Box<dyn SystemFn>,
    ) -> Self {
        Self {
            id,
            name,
            phases,
            priority,
            run_before,
            run_after,
            enabled: true,
            consecutive_failures: 0,
            on_error: None,
            fn_ptr: func,
        }
    }

    pub fn with_error_handler(mut self, handler: SystemErrorHandler) -> Self {
        self.on_error = Some(handler);
        self
    }

    pub fn run(&mut self, ctx: SystemContext) {
        if !self.enabled {
            return;
        }

        match self.fn_ptr.run(ctx) {
            Ok(_) => {
                self.consecutive_failures = 0;
            }
            Err(e) => {
                self.consecutive_failures += 1;
                let phase_str = match self.phases.iter().next().unwrap_or(&SystemPhase::Update) {
                    SystemPhase::Init => "init",
                    SystemPhase::FixedUpdate => "fixedUpdate",
                    SystemPhase::Update => "update",
                    SystemPhase::LateUpdate => "lateUpdate",
                    SystemPhase::Cleanup => "cleanup",
                }.to_string(); // Approximate phase for now

                let error_ctx = SystemErrorContext {
                    system_id: self.id.clone(),
                    error: e,
                    phase: phase_str,
                    consecutive_failures: self.consecutive_failures,
                };

                let strategy = if let Some(handler) = self.on_error {
                    handler(&error_ctx)
                } else {
                    default_error_handler(&error_ctx)
                };

                match strategy {
                    SystemErrorStrategy::Disable => self.enabled = false,
                    SystemErrorStrategy::Ignore => {},
                    SystemErrorStrategy::Retry => {
                        // Retry logic would go here, but requires re-running the function immediately
                        // which is tricky with ownership. For now, we treat Retry as Ignore.
                    }
                }
            }
        }
    }
}

pub struct SystemScheduler {
    systems: HashMap<SystemId, Arc<Mutex<System>>>,
    execution_order: HashMap<SystemPhase, Vec<SystemId>>,
    dirty: bool,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            execution_order: HashMap::new(),
            dirty: true,
        }
    }

    pub fn add(&mut self, system: System) {
        if self.systems.contains_key(&system.id) {
            panic!("System {} already exists", system.id);
        }
        self.systems.insert(system.id.clone(), Arc::new(Mutex::new(system)));
        self.dirty = true;
    }

    pub fn remove(&mut self, system_id: &str) -> bool {
        if self.systems.remove(system_id).is_some() {
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn execute_phase(&mut self, phase: SystemPhase, world: &mut World, delta_time: f64, time: f64) {
        if self.dirty {
            self.recompute_execution_order();
        }

        if let Some(system_ids) = self.execution_order.get(&phase) {
            let ids = system_ids.clone(); 
            
            for system_id in ids {
                if let Some(system_arc) = self.systems.get(&system_id) {
                    let mut system = system_arc.lock().unwrap();
                    let ctx = SystemContext {
                        world,
                        delta_time,
                        time,
                        phase,
                    };
                    system.run(ctx);
                }
            }
        }
    }

    fn recompute_execution_order(&mut self) {
        self.execution_order.clear();
        let phases = [
            SystemPhase::Init,
            SystemPhase::FixedUpdate,
            SystemPhase::Update,
            SystemPhase::LateUpdate,
            SystemPhase::Cleanup,
        ];

        for phase in phases {
            let phase_systems: Vec<Arc<Mutex<System>>> = self.systems.values()
                .filter(|s| s.lock().unwrap().phases.contains(&phase))
                .cloned()
                .collect();
            
            let sorted = self.topological_sort(phase_systems);
            self.execution_order.insert(phase, sorted);
        }

        self.dirty = false;
    }

    fn topological_sort(&self, systems: Vec<Arc<Mutex<System>>>) -> Vec<SystemId> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        let mut systems_by_priority = systems;
        systems_by_priority.sort_by(|a, b| {
            b.lock().unwrap().priority.cmp(&a.lock().unwrap().priority)
        });

        for system_arc in &systems_by_priority {
            let system_id = system_arc.lock().unwrap().id.clone();
            self.visit(&system_id, &mut visited, &mut visiting, &mut sorted);
        }

        sorted
    }

    fn visit(
        &self,
        system_id: &SystemId,
        visited: &mut HashSet<SystemId>,
        visiting: &mut HashSet<SystemId>,
        sorted: &mut Vec<SystemId>,
    ) {
        if visited.contains(system_id) {
            return;
        }

        if visiting.contains(system_id) {
            panic!("Circular dependency detected in system: {}", system_id);
        }

        visiting.insert(system_id.clone());

        let system_arc = self.systems.get(system_id).unwrap();
        let run_after = system_arc.lock().unwrap().run_after.clone();

        for after_id in run_after {
            if self.systems.contains_key(&after_id) {
                self.visit(&after_id, visited, visiting, sorted);
            }
        }

        let run_before = system_arc.lock().unwrap().run_before.clone();
        for before_id in run_before {
             if let Some(_) = self.systems.get(&before_id) {
                 if !visited.contains(&before_id) {
                     visiting.remove(system_id);
                     return;
                 }
             }
        }

        visiting.remove(system_id);
        visited.insert(system_id.clone());
        sorted.push(system_id.clone());
    }
}
