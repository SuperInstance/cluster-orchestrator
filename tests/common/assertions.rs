//! Custom assertion helpers for testing

use std::time::Duration;
use crate::common::{TestCluster, wait_for, DEFAULT_TIMEOUT};

/// Assert that a deployment has the expected number of replicas
pub async fn assert_replica_count(
    cluster: &TestCluster,
    namespace: &str,
    deployment: &str,
    expected: u32,
) -> anyhow::Result<()> {
    wait_for(|| async {
        match get_deployment_replicas(cluster, namespace, deployment).await {
            Ok(count) => count == expected,
            Err(_) => false,
        }
    }, DEFAULT_TIMEOUT).await?;

    Ok(())
}

/// Assert that all pods in a deployment are ready
pub async fn assert_pods_ready(
    cluster: &TestCluster,
    namespace: &str,
    deployment: &str,
    expected: u32,
) -> anyhow::Result<()> {
    wait_for(|| async {
        match get_ready_pod_count(cluster, namespace, deployment).await {
            Ok(count) => count == expected,
            Err(_) => false,
        }
    }, DEFAULT_TIMEOUT).await?;

    Ok(())
}

/// Assert cluster health
pub async fn assert_cluster_healthy(cluster: &TestCluster) -> anyhow::Result<()> {
    let health = cluster.get_cluster_health().await?;

    assert!(health.nodes_ready, "Cluster nodes are not ready");
    assert!(health.control_plane_healthy, "Control plane is not healthy");

    Ok(())
}

/// Assert scaling decision
#[derive(Debug)]
pub struct ScalingAssertion {
    pub direction: String,
    pub current_replicas: u32,
    pub desired_replicas: u32,
    pub reason: String,
}

pub fn assert_scaling_decision(
    current: u32,
    desired: u32,
    threshold: f64,
    metric_value: f64,
) -> ScalingAssertion {
    let direction = if metric_value > threshold {
        "up".to_string()
    } else if metric_value < threshold * 0.5 {
        "down".to_string()
    } else {
        "none".to_string()
    };

    let desired_replicas = match direction.as_str() {
        "up" => current + 1,
        "down" => (current - 1).max(1),
        _ => current,
    };

    ScalingAssertion {
        direction,
        current_replicas: current,
        desired_replicas,
        reason: format!("Metric value {:.2} exceeds threshold {:.2}", metric_value, threshold),
    }
}

/// Assert recovery time
pub fn assert_recovery_time(
    actual: Duration,
    max_expected: Duration,
    operation: &str,
) -> anyhow::Result<()> {
    if actual > max_expected {
        anyhow::bail!(
            "{} took {:?} which exceeds maximum expected of {:?}",
            operation,
            actual,
            max_expected
        );
    }
    Ok(())
}

/// Assert no service disruption
pub async fn assert_no_downtime(
    cluster: &TestCluster,
    namespace: &str,
    service: &str,
    duration: Duration,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    while start.elapsed() < duration {
        match check_service_available(cluster, namespace, service).await {
            Ok(true) => {
                // Service is available, wait a bit
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(false) => {
                anyhow::bail!("Service {} was unavailable during monitoring period", service);
            }
            Err(e) => {
                anyhow::bail!("Failed to check service availability: {}", e);
            }
        }
    }

    Ok(())
}

/// Helper: Get deployment replica count
async fn get_deployment_replicas(
    cluster: &TestCluster,
    namespace: &str,
    deployment: &str,
) -> anyhow::Result<u32> {
    let output = tokio::process::Command::new("kubectl")
        .args([
            "get",
            "deployment",
            deployment,
            "-n",
            namespace,
            "-o",
            "jsonpath='{.spec.replicas}'",
        ])
        .env("KUBECONFIG", &cluster.kubeconfig)
        .env("KUBECONTEXT", &cluster.context)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to get deployment: {}", String::from_utf8_lossy(&output.stderr));
    }

    let count_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_matches('\'')
        .to_string();

    Ok(count_str.parse::<u32>().unwrap_or(0))
}

/// Helper: Get ready pod count
async fn get_ready_pod_count(
    cluster: &TestCluster,
    namespace: &str,
    deployment: &str,
) -> anyhow::Result<u32> {
    let output = tokio::process::Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            namespace,
            "-l",
            &format!("app={}", deployment),
            "-o",
            "json",
        ])
        .env("KUBECONFIG", &cluster.kubeconfig)
        .env("KUBECONTEXT", &cluster.context)
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to get pods: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Parse JSON and count ready pods
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let items = json["items"].as_array().ok_or_else(|| anyhow::anyhow!("Invalid JSON"))?;

    let ready_count = items
        .iter()
        .filter(|pod| {
            pod["status"]["phase"].as_str() == Some("Running")
                && pod["status"].get("containerStatuses").is_some()
        })
        .count();

    Ok(ready_count as u32)
}

/// Helper: Check if service is available
async fn check_service_available(
    cluster: &TestCluster,
    namespace: &str,
    service: &str,
) -> anyhow::Result<bool> {
    let output = tokio::process::Command::new("kubectl")
        .args([
            "get",
            "endpoints",
            service,
            "-n",
            namespace,
            "-o",
            "json",
        ])
        .env("KUBECONFIG", &cluster.kubeconfig)
        .env("KUBECONTEXT", &cluster.context)
        .output()
        .await?;

    if !output.status.success() {
        return Ok(false);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let subsets = json["subsets"].as_array();

    match subsets {
        Some(subs) if !subs.is_empty() => Ok(true),
        _ => Ok(false),
    }
}
