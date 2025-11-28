pub mod component;
pub mod entity;
pub mod system;
pub mod world;
pub mod serialization;
pub mod query;
pub mod error;

#[cfg(feature = "native")]
pub mod renderer;

#[cfg(feature = "native")]
pub mod app;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
mod tests;

pub use entity::EntityId;
pub use world::World;
pub use error::TX2Error;

#[cfg(feature = "native")]
pub use app::App;
