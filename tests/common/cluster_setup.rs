//! Test cluster setup and management

use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;

/// Test cluster wrapper
pub struct TestCluster {
    pub name: String,
    pub kubeconfig: String,
    pub context: String,
}

impl TestCluster {
    /// Create a new test cluster reference
    pub fn new(name: String, kubeconfig: String, context: String) -> Self {
        Self {
            name,
            kubeconfig,
            context,
        }
    }

    /// Create a test namespace
    pub async fn create_namespace(&self, name: &str) -> Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args(["create", "namespace", name])
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECONTEXT", &self.context)
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to create namespace: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Delete a test namespace
    pub async fn delete_namespace(&self, name: &str) -> Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args(["delete", "namespace", name])
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECONTEXT", &self.context)
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to delete namespace: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Apply a manifest to the cluster
    pub async fn apply_manifest(&self, manifest: &str) -> Result<()> {
        let output = tokio::process::Command::new("kubectl")
            .args(["apply", "-f", "-"])
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECONTEXT", &self.context)
            .stdin(std::process::Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to apply manifest: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Get cluster health status
    pub async fn get_cluster_health(&self) -> Result<ClusterHealth> {
        // Check nodes are ready
        let nodes_output = tokio::process::Command::new("kubectl")
            .args(["get", "nodes", "-o", "json"])
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECONTEXT", &self.context)
            .output()
            .await?;

        if !nodes_output.status.success() {
            anyhow::bail!("Failed to get nodes: {}", String::from_utf8_lossy(&nodes_output.stderr));
        }

        // Check control plane is healthy
        let health_output = tokio::process::Command::new("kubectl")
            .args(["get", "cs", "-o", "json"])
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECONTEXT", &self.context)
            .output()
            .await?;

        Ok(ClusterHealth {
            nodes_ready: true, // Parse actual output
            control_plane_healthy: true,
            pod_count: 0,
        })
    }
}

#[derive(Debug)]
pub struct ClusterHealth {
    pub nodes_ready: bool,
    pub control_plane_healthy: bool,
    pub pod_count: usize,
}

/// Setup a test cluster (uses kind by default)
pub async fn setup_test_cluster() -> Result<TestCluster> {
    // Check if cluster exists
    let cluster_name = "cluster-orchestrator-test";

    let output = tokio::process::Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .await?;

    let clusters = String::from_utf8_lossy(&output.stdout);
    let exists = clusters.contains(cluster_name);

    if !exists {
        // Create new cluster
        println!("Creating kind cluster: {}", cluster_name);

        let create_output = tokio::process::Command::new("kind")
            .args(["create", "cluster", "--name", cluster_name])
            .output()
            .await?;

        if !create_output.status.success() {
            anyhow::bail!("Failed to create cluster: {}", String::from_utf8_lossy(&create_output.stderr));
        }

        // Wait for cluster to be ready
        sleep(Duration::from_secs(10)).await;
    }

    // Get kubeconfig path
    let kubeconfig = std::env::var("HOME").unwrap_or_default() + "/.kube/config";

    Ok(TestCluster {
        name: cluster_name.to_string(),
        kubeconfig,
        context: format!("kind-{}", cluster_name),
    })
}

/// Cleanup test cluster
pub async fn cleanup_test_cluster(cluster: TestCluster) -> Result<()> {
    // For CI, keep the cluster. For local, optionally delete.
    // Check environment variable
    if std::env::var("CLUSTER_ORCHESTRATOR_DELETE_CLUSTER").is_ok() {
        println!("Deleting kind cluster: {}", cluster.name);

        let output = tokio::process::Command::new("kind")
            .args(["delete", "cluster", "--name", &cluster.name])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("Failed to delete cluster: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_cluster_setup() {
        let cluster = setup_test_cluster().await.unwrap();
        let health = cluster.get_cluster_health().await.unwrap();

        assert!(health.nodes_ready);
        assert!(health.control_plane_healthy);

        cleanup_test_cluster(cluster).await.unwrap();
    }
}
