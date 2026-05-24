pub mod types;
pub mod curves;
pub mod emitter;
pub mod forces;
pub mod collision;

pub use types::{Range, BurstSpec};
pub use curves::{Curve, Gradient};
pub use emitter::{EmitterShape, InitialVelocity, ColorRange};
pub use forces::{VortexConfig, AttractorConfig, CurlNoiseConfig};
pub use collision::{CollisionConfig, CollisionEvent};
