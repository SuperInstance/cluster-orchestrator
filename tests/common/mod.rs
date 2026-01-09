//! Common test utilities and helpers

pub mod cluster_setup;
pub mod assertions;
pub mod helpers;
pub mod fixtures;

pub use cluster_setup::{TestCluster, setup_test_cluster, cleanup_test_cluster};
pub use assertions::*;
pub use helpers::*;
pub use fixtures::*;

use std::time::Duration;
use tokio::time::sleep;

/// Default timeout for test operations
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Default number of replicas for testing
pub const DEFAULT_REPLICAS: u32 = 3;

/// Default namespace for test resources
pub const TEST_NAMESPACE: &str = "test-cluster-orchestrator";

/// Test context for managing test lifecycle
pub struct TestContext {
    pub cluster: TestCluster,
    pub namespace: String,
    pub cleanup_resources: Vec<String>,
}

impl TestContext {
    pub async fn new() -> anyhow::Result<Self> {
        let cluster = setup_test_cluster().await?;
        let namespace = TEST_NAMESPACE.to_string();

        // Create test namespace
        cluster.create_namespace(&namespace).await?;

        Ok(Self {
            cluster,
            namespace,
            cleanup_resources: Vec::new(),
        })
    }

    pub async fn cleanup(self) -> anyhow::Result<()> {
        // Delete namespace (cascades to all resources)
        self.cluster.delete_namespace(&self.namespace).await?;
        cleanup_test_cluster(self.cluster).await?;
        Ok(())
    }
}

/// Wait helper with timeout
pub async fn wait_for<F, Fut>(condition: F, timeout: Duration) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        interval.tick().await;

        if condition().await {
            return Ok(());
        }

        if start.elapsed() > timeout {
            anyhow::bail!("Condition not met within {:?}", timeout);
        }
    }
}

/// Retry helper for transient failures
pub async fn retry<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
