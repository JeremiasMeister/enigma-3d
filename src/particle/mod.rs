pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;
pub mod buffer;
pub mod config;
pub mod system;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
pub use buffer::{ParticleBuffer, ParticleSpawn};
pub use config::{ParticleSystemConfig, RenderStyle, BlendMode, FlipbookConfig, LightEmitConfig};
pub use system::{ParticleSystem, EmissionState, TickResult, DeathEvent};
