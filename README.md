# tx2-core

**A high-performance native ECS engine for cross-platform applications, games, and simulations.**

tx2-core is the native Rust implementation of the TX-2 world runtime, providing an authoritative Entity-Component-System architecture that powers desktop apps, game-like experiences, and high-performance servers. It's designed to work seamlessly with the broader TX-2 ecosystem for isomorphic state synchronization across web, native, and CLI environments.

## Features

### Core ECS Architecture
- **Type-safe entity-component system** with compile-time guarantees
- **Flexible component registration** supporting any serializable Rust type
- **Efficient query system** for component iteration and filtering
- **System scheduling** with dependency tracking and execution ordering
- **World snapshots** for save/load, time-travel debugging, and checkpointing

### Rendering & Graphics
- **GPU-accelerated rendering** via `wgpu` (WebGPU API)
- **Automatic render batching** by component type for optimal performance
- **Instanced rendering** support for drawing thousands of entities efficiently
- **Cross-platform windowing** using `winit` (Windows, macOS, Linux)

### Serialization & Sync
- **Delta compression** via `tx2-link` for efficient state synchronization
- **Field-level change detection** to minimize network bandwidth
- **Multiple serialization formats** (MessagePack, Bincode, JSON)
- **Schema versioning** for backward-compatible protocol evolution

### Application Framework
- **Event-driven architecture** with platform event integration
- **Fixed timestep simulation** for deterministic physics and gameplay
- **Hot-reload support** for rapid development iteration
- **Extensible system pipeline** for custom game logic

## Quick Start

```rust
use tx2_core::{World, Entity, Component, System, App};
use serde::{Serialize, Deserialize};

// Define components
#[derive(Component, Serialize, Deserialize, Clone)]
struct Position { x: f32, y: f32 }

#[derive(Component, Serialize, Deserialize, Clone)]
struct Velocity { dx: f32, dy: f32 }

// Create a world
let mut world = World::new();
world.register_component::<Position>();
world.register_component::<Velocity>();

// Spawn entities
let player = world.create_entity();
world.add_component(player, Position { x: 0.0, y: 0.0 });
world.add_component(player, Velocity { dx: 1.0, dy: 0.5 });

// Query and iterate
for (entity, (pos, vel)) in world.query::<(&Position, &Velocity)>() {
    println!("Entity {:?} at ({}, {})", entity, pos.x, pos.y);
}
```

## Running the Example

```bash
cargo run --example hello_window
```

This demonstrates:
- Window creation with `winit`
- Basic rendering setup with `wgpu`
- Entity creation and component management
- Event loop integration

## Architecture

tx2-core follows a pure ECS architecture where:

- **Entities** are unique identifiers (UUIDs) representing game objects
- **Components** are pure data structs attached to entities
- **Systems** are functions that operate on queries of components
- **World** manages all entities, components, and systems

### Component Storage

Components are stored in type-erased hashmaps, allowing:
- Dynamic component registration at runtime
- Zero-cost abstractions via generics
- Efficient batch operations on component types

### System Execution

Systems are executed in a deterministic order based on:
- Explicit dependencies between systems
- Read/write access patterns to prevent data races
- Fixed timestep scheduling for consistent simulation

## Integration with TX-2 Ecosystem

tx2-core is designed to work as part of the broader TX-2 stack:

- **tx2-ecs** (TypeScript/Node): Web runtime with reactive UI rendering
- **tx2-link**: Protocol for syncing worlds between runtimes (WebSocket, IPC, stdio)
- **tx2-pack**: Binary snapshot format for save/load and time-travel replay
- **tx2-core**: Native engine (this crate)

The same world state can be:
- Rendered as a 3D game in tx2-core
- Displayed as a web dashboard in tx2-ecs
- Synchronized in realtime via tx2-link
- Saved/loaded using tx2-pack

One world, many views.

## Performance

tx2-core is built for high-performance applications:

- **Batch rendering**: Components of the same type are rendered together
- **GPU instancing**: Draw thousands of entities in a single draw call
- **Delta compression**: 11.9× reduction in network traffic (2.28MB → 196KB)
- **Memory efficiency**: Struct-of-arrays layout for cache-friendly iteration
- **Zero-copy serialization**: Direct memory mapping for fast snapshots

## Serialization & Networking

World state can be serialized for:

- **Save/load**: Checkpoint entire worlds to disk
- **Network sync**: Delta-compressed updates to clients
- **Time-travel**: Replay simulation from any point in time
- **Hot-reload**: Preserve state across code changes

Supported formats:
- **MessagePack**: Compact binary format (default)
- **Bincode**: Fast Rust-native serialization
- **JSON**: Human-readable debugging format

## Examples

### Creating Entities

```rust
let entity = world.create_entity();
world.add_component(entity, Position { x: 10.0, y: 20.0 });
world.add_component(entity, Health { current: 100, max: 100 });
```

### Querying Components

```rust
// Iterate over all entities with Position and Velocity
for (entity, (pos, vel)) in world.query::<(&mut Position, &Velocity)>() {
    pos.x += vel.dx;
    pos.y += vel.dy;
}
```

### World Snapshots

```rust
// Create a snapshot of the current world state
let snapshot = world.create_snapshot();

// Restore from snapshot
world.restore_from_snapshot(&snapshot)?;
```

### Delta Compression

```rust
use tx2_core::DeltaCompressor;

let mut compressor = DeltaCompressor::new();

// Generate delta between two snapshots
let delta = compressor.create_delta(&previous_snapshot, &current_snapshot)?;

// Apply delta to reconstruct current state
let reconstructed = compressor.apply_delta(&previous_snapshot, &delta)?;
```

## Rendering

tx2-core includes a rendering abstraction built on `wgpu`:

```rust
use tx2_core::{Renderer, RenderBatch};

// Create renderer
let mut renderer = Renderer::new(&window).await;

// Prepare render batches from world
let batches = renderer.prepare_batches(&world);

// Render frame
renderer.render(&batches)?;
```

## Application Framework

Build complete applications with the App framework:

```rust
use tx2_core::App;

let mut app = App::new("My Game", 800, 600)?;

app.run(|world, delta_time| {
    // Update game logic
    update_physics(world, delta_time);
    update_ai(world, delta_time);
    update_animations(world, delta_time);
});
```

## Development Status

tx2-core is in active development. Current status:

- [x] Core ECS implementation
- [x] Component registration and queries
- [x] World serialization
- [x] Delta compression integration
- [x] Basic rendering with wgpu
- [x] Window and event handling
- [x] Render batching
- [ ] Advanced physics integration
- [ ] Audio system
- [ ] Asset loading pipeline
- [ ] Scripting integration

## Dependencies

- `serde` - Serialization framework
- `uuid` - Entity identifiers
- `winit` - Cross-platform windowing
- `wgpu` - WebGPU rendering API
- `tx2-link` - World synchronization protocol

## License

MIT

## Contributing

Contributions are welcome! This is part of the broader TX-2 project for building isomorphic applications with a unified world model.

## Learn More

- [TX-2 Framework Outline](../frameworkoutline.md)
- [tx2-link Protocol](../tx2-link)
- [tx2-pack Format](../tx2-pack)
- [tx2-ecs TypeScript Runtime](https://github.com/IreGaddr/tx2-ecs)
