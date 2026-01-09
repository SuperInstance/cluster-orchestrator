//! Chaos engineering tests for pod failures

use std::time::Duration;
use crate::common::{
    TestContext, setup_test_cluster, cleanup_test_cluster,
    assert_pods_ready, assert_cluster_healthy,
    DEFAULT_TIMEOUT,
};

const TEST_NAMESPACE: &str = "chaos-testing";

#[tokio::test]
#[ignore]
async fn test_pod_kill_recovery() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy workload
    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    // Inject pod kill chaos
    let chaos = inject_pod_kill(&ctx.cluster, "test-app", 1).await?;

    // Wait for recovery
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Verify all pods recovered
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 3).await?;

    // Cleanup chaos
    cleanup_chaos(chaos).await?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_multiple_pod_kill_recovery() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    deploy_test_app(&ctx.cluster, "test-app", 5).await?;

    // Kill multiple pods
    let chaos = inject_pod_kill(&ctx.cluster, "test-app", 3).await?;

    // Wait for recovery
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Verify all pods recovered
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 5).await?;

    cleanup_chaos(chaos).await?;
    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pod_kill_during_scale_up() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    // Start scale-up
    let scale_task = tokio::spawn(async move {
        ctx.cluster.scale_deployment(TEST_NAMESPACE, "test-app", 8).await
    });

    // Wait a bit, then kill pods
    tokio::time::sleep(Duration::from_secs(2)).await;
    let chaos = inject_pod_kill(&ctx.cluster, "test-app", 2).await?;

    // Wait for scale-up to complete
    scale_task.await???;

    // Verify final state
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 8).await?;

    cleanup_chaos(chaos).await?;
    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pod_failure_continuous() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    // Continuous pod failure for 30 seconds
    let chaos = inject_continuous_pod_kill(&ctx.cluster, "test-app", Duration::from_secs(30)).await?;

    // Monitor during chaos
    for _ in 0..6 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        // Cluster should remain healthy despite chaos
        assert_cluster_healthy(&ctx.cluster).await?;
    }

    // Stop chaos
    cleanup_chaos(chaos).await?;

    // Verify recovery
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 3).await?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_container_failure_recovery() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    // Inject container failure
    let chaos = inject_container_kill(&ctx.cluster, "test-app", "app", 1).await?;

    // Wait for recovery
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Verify all pods recovered
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 3).await?;

    cleanup_chaos(chaos).await?;
    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pod_crash_loop_recovery() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy with crash loop image
    deploy_crashing_app(&ctx.cluster, "crash-app", 3).await?;

    // Wait for crash loop backoff
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Verify cluster handles crash loops
    let pod_status = get_pod_status(&ctx.cluster, TEST_NAMESPACE, "crash-app").await?;
    assert!(pod_status.restart_count > 0, "Pod should have restarted");

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_pod_kill_with_service_disruption() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Create service
    create_service(&ctx.cluster, "test-app").await?;
    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    // Start monitoring service availability
    let monitor_task = tokio::spawn(async move {
        monitor_service_availability(&ctx.cluster, "test-app", Duration::from_secs(60)).await
    });

    // Kill pods
    let chaos = inject_pod_kill(&ctx.cluster, "test-app", 2).await?;

    // Wait and verify no disruption
    let available = monitor_task.await???;
    assert!(available > 0.8, "Service should be available > 80% of the time");

    cleanup_chaos(chaos).await?;
    ctx.cleanup().await?;
    Ok(())
}

// Helper functions

async fn deploy_test_app(
    cluster: &TestCluster,
    name: &str,
    replicas: u32,
) -> anyhow::Result<()> {
    let manifest = format!(
        r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  namespace: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: app
        image: nginx:latest
        ports:
        - containerPort: 8080
"#,
        name, TEST_NAMESPACE, replicas, name, name
    );

    cluster.apply_manifest(&manifest).await?;
    Ok(())
}

async fn deploy_crashing_app(
    cluster: &TestCluster,
    name: &str,
    replicas: u32,
) -> anyhow::Result<()> {
    let manifest = format!(
        r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  namespace: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: app
        image: nginx:latest
        command: ["/bin/sh", "-c"]
        args: ["exit 1"]
"#,
        name, TEST_NAMESPACE, replicas, name, name
    );

    cluster.apply_manifest(&manifest).await?;
    Ok(())
}

async fn inject_pod_kill(
    cluster: &TestCluster,
    app: &str,
    count: usize,
) -> anyhow::Result<ChaosInjection> {
    let chaos_name = format!("pod-kill-{}", app);

    let manifest = format!(
        r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: {}
  namespace: {}
spec:
  action: pod-kill
  mode: fixed
  value: "{}"
  selector:
    namespaces:
      - {}
    labelSelectors:
      app: {}
"#,
        chaos_name, TEST_NAMESPACE, count, TEST_NAMESPACE, app
    );

    cluster.apply_manifest(&manifest).await?;

    Ok(ChaosInjection {
        name: chaos_name,
        namespace: TEST_NAMESPACE.to_string(),
    })
}

async fn inject_continuous_pod_kill(
    cluster: &TestCluster,
    app: &str,
    duration: Duration,
) -> anyhow::Result<ChaosInjection> {
    let chaos_name = format!("continuous-pod-kill-{}", app);
    let interval = "10s";

    let manifest = format!(
        r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: {}
  namespace: {}
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - {}
    labelSelectors:
      app: {}
  scheduler:
    cron: "*/{} * * * *"
"#,
        chaos_name, TEST_NAMESPACE, TEST_NAMESPACE, app, interval
    );

    cluster.apply_manifest(&manifest).await?;

    Ok(ChaosInjection {
        name: chaos_name,
        namespace: TEST_NAMESPACE.to_string(),
    })
}

async fn inject_container_kill(
    cluster: &TestCluster,
    app: &str,
    container: &str,
    count: usize,
) -> anyhow::Result<ChaosInjection> {
    let chaos_name = format!("container-kill-{}", app);

    let manifest = format!(
        r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: {}
  namespace: {}
spec:
  action: container-kill
  mode: fixed
  value: "{}"
  containerNames:
    - {}
  selector:
    namespaces:
      - {}
    labelSelectors:
      app: {}
"#,
        chaos_name, TEST_NAMESPACE, count, container, TEST_NAMESPACE, app
    );

    cluster.apply_manifest(&manifest).await?;

    Ok(ChaosInjection {
        name: chaos_name,
        namespace: TEST_NAMESPACE.to_string(),
    })
}

async fn cleanup_chaos(chaos: ChaosInjection) -> anyhow::Result<()> {
    let output = tokio::process::Command::new("kubectl")
        .args([
            "delete",
            "podchaos",
            &chaos.name,
            "-n",
            &chaos.namespace,
        ])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to cleanup chaos: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

async fn create_service(cluster: &TestCluster, name: &str) -> anyhow::Result<()> {
    let manifest = format!(
        r#"
apiVersion: v1
kind: Service
metadata:
  name: {}
  namespace: {}
spec:
  selector:
    app: {}
  ports:
  - port: 80
    targetPort: 8080
"#,
        name, TEST_NAMESPACE, name
    );

    cluster.apply_manifest(&manifest).await?;
    Ok(())
}

async fn monitor_service_availability(
    cluster: &TestCluster,
    service: &str,
    duration: Duration,
) -> anyhow::Result<f64> {
    let start = std::time::Instant::now();
    let mut available_count = 0;
    let mut total_checks = 0;

    while start.elapsed() < duration {
        // Check if service endpoints are available
        let output = tokio::process::Command::new("kubectl")
            .args([
                "get",
                "endpoints",
                service,
                "-n",
                TEST_NAMESPACE,
                "-o",
                "json",
            ])
            .output()
            .await?;

        if output.status.success() {
            let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
            if json["subsets"].as_array().map_or(false, |s| !s.is_empty()) {
                available_count += 1;
            }
        }

        total_checks += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(available_count as f64 / total_checks as f64)
}

async fn get_pod_status(
    cluster: &TestCluster,
    namespace: &str,
    app: &str,
) -> anyhow::Result<PodStatus> {
    let output = tokio::process::Command::new("kubectl")
        .args([
            "get",
            "pods",
            "-n",
            namespace,
            "-l",
            &format!("app={}", app),
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

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let pod = json["items"].as_array()
        .and_then(|pods| pods.first())
        .ok_or_else(|| anyhow::anyhow!("No pods found"))?;

    let restart_count = pod["status"]["containerStatuses"].as_array()
        .and_then(|statuses| statuses.first())
        .and_then(|status| status["restartCount"].as_u64())
        .unwrap_or(0) as u32;

    Ok(PodStatus {
        name: pod["metadata"]["name"].as_str().unwrap_or("unknown").to_string(),
        restart_count,
    })
}

struct ChaosInjection {
    name: String,
    namespace: String,
}

struct PodStatus {
    name: String,
    restart_count: u32,
}
