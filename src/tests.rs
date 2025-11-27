#[cfg(test)]
mod tests {
    use crate::world::World;
    use crate::component::Component;
    use crate::serialization::DeltaCompressor;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Position {
        x: f64,
        y: f64,
    }

    #[test]
    fn test_ecs_flow() {
        let mut world = World::new();
        let entity = world.create_entity();
        
        let pos = Position { x: 10.0, y: 20.0 };
        println!("Component ID: {}", std::any::type_name::<Position>());
        world.add_component(entity.id, Box::new(pos));

        assert!(world.has_component(entity.id, std::any::type_name::<Position>()));
        
        let stored_pos: &Position = world.get_component(entity.id).unwrap();
        assert_eq!(stored_pos.x, 10.0);
    }

    #[test]
    fn test_serialization_delta() {
        let mut world = World::new();
        let mut compressor = DeltaCompressor::new();

        // Frame 1: Create entity with Position
        let entity = world.create_entity();
        world.add_component(entity.id, Box::new(Position { x: 0.0, y: 0.0 }));

        let delta1 = compressor.create_delta(&world);
        assert_eq!(delta1.changes.len(), 2); // EntityAdded, ComponentAdded

        // Frame 2: Update Position
        // Note: In our current implementation, we need to remove and add to update, or use interior mutability.
        // For this test, let's remove and add.
        world.remove_component(entity.id, &std::any::type_name::<Position>());
        world.add_component(entity.id, Box::new(Position { x: 1.0, y: 0.0 }));

        let delta2 = compressor.create_delta(&world);
        // Should detect update because we compare snapshots
        // Wait, if I remove and add, the snapshot just sees the new value.
        // The delta compressor compares data.
        // So it should be ComponentUpdated.
        
        let changes = delta2.changes;
        println!("{:?}", changes);
        assert!(changes.iter().any(|c| matches!(c, crate::serialization::DeltaChange::ComponentUpdated { .. })));
    }

    #[test]
    fn test_query_system() {
        let mut world = World::new();
        
        // Entity 1: Position
        let e1 = world.create_entity();
        world.add_component(e1.id, Box::new(Position { x: 0.0, y: 0.0 }));

        // Entity 2: Position
        let e2 = world.create_entity();
        world.add_component(e2.id, Box::new(Position { x: 10.0, y: 10.0 }));

        // Entity 3: No Position
        let _e3 = world.create_entity();

        // We need to manually execute the query against the store because World::query takes a descriptor,
        // but we built a Query object.
        // Let's use World::query instead.
        let results = world.query(crate::query::QueryDescriptor {
            all: vec![std::any::type_name::<Position>().to_string()],
            ..Default::default()
        });

        assert_eq!(results.len(), 2);
        assert!(results.contains(&e1.id));
        assert!(results.contains(&e2.id));
    }

    #[test]
    fn test_system_error_handling() {
        use crate::system::{System, SystemPhase};
        use crate::error::SystemErrorStrategy;
        use std::collections::HashSet;

        let mut world = World::new();
        let mut scheduler = crate::system::SystemScheduler::new();

        let failing_system = System::new(
            "failing_system".to_string(),
            "Failing System".to_string(),
            HashSet::from([SystemPhase::Update]),
            0,
            HashSet::new(),
            HashSet::new(),
            Box::new(|_ctx: crate::system::SystemContext| Err("Something went wrong".to_string())),
        );

        scheduler.add(failing_system);

        // Run 4 times. Default handler disables after 3 failures.
        for _ in 0..4 {
            scheduler.execute_phase(SystemPhase::Update, &mut world, 0.16, 0.0);
        }

        // We can't easily inspect the system state from here because it's wrapped in Arc<Mutex>.
        // But we can verify it doesn't panic.
    }
}
