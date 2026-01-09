//! # Cluster Orchestrator
//!
//! Production-grade Kubernetes cluster orchestration and management library.
//!
//! ## Features
//!
//! - Cluster management and lifecycle
//! - Auto-scaling (horizontal and vertical)
//! - Self-healing and fault tolerance
//! - Resource optimization
//! - Multi-cluster management
//! - Chaos engineering integration

pub mod cluster;
pub mod scaling;
pub mod healing;
pub mod optimizer;
pub mod multicluster;
pub mod metrics;
pub mod config;

pub use cluster::{ClusterManager, ClusterState};
pub use scaling::{Scaler, ScalingPolicy, ScalingDirection};
pub use healing::{Healer, HealingAction, HealthChecker};
pub use optimizer::{ResourceOptimizer, OptimizationStrategy};
pub use multicluster::{MultiClusterManager, ClusterRegistry};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default timeout for operations (seconds)
pub const DEFAULT_TIMEOUT: u64 = 300;

/// Maximum retry attempts for transient failures
pub const MAX_RETRIES: u32 = 3;
