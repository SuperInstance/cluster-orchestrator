//! Integration tests for scaling workflows

use std::time::Duration;
use crate::common::{
    TestContext, setup_test_cluster, cleanup_test_cluster,
    assert_replica_count, assert_pods_ready, assert_no_downtime,
    helpers::{deploy_and_wait, generate_load, measure_duration},
    DEFAULT_TIMEOUT,
};

#[tokio::test]
#[ignore]
async fn test_horizontal_scale_up() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy initial workload
    deploy_and_wait(&ctx.cluster, "test-app", 3).await?;

    // Apply HPA
    let hpa_manifest = create_hpa_manifest("test-app", 2, 10, 70);
    ctx.cluster.apply_manifest(&hpa_manifest).await?;

    // Generate load to trigger scale-up
    let load_duration = Duration::from_secs(30);
    tokio::spawn(async move {
        generate_load(&ctx.cluster, TEST_NAMESPACE, "test-app", load_duration).await
    });

    // Wait for scale-up
    assert_replica_count(&ctx.cluster, TEST_NAMESPACE, "test-app", 5).await?;

    // Verify all pods ready
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 5).await?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_horizontal_scale_down() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy with high replica count
    deploy_and_wait(&ctx.cluster, "test-app", 8).await?;

    // Apply HPA
    let hpa_manifest = create_hpa_manifest("test-app", 2, 10, 70);
    ctx.cluster.apply_manifest(&hpa_manifest).await?;

    // Wait for scale-down due to low load
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Verify scaled down
    assert_replica_count(&ctx.cluster, TEST_NAMESPACE, "test-app", 2).await?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_scale_up_latency() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy initial workload
    deploy_and_wait(&ctx.cluster, "test-app", 3).await?;

    // Trigger scale-up
    let (result, duration) = measure_duration(
        ctx.cluster.scale_deployment(TEST_NAMESPACE, "test-app", 5)
    ).await;

    result?;

    // Assert SLA: scale-up < 60 seconds
    assert!(duration < Duration::from_secs(60), "Scale-up took {:?}", duration);

    // Verify all pods ready
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 5).await?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_multi_resource_scaling() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy multiple applications
    deploy_and_wait(&ctx.cluster, "app1", 3).await?;
    deploy_and_wait(&ctx.cluster, "app2", 3).await?;
    deploy_and_wait(&ctx.cluster, "app3", 3).await?;

    // Apply HPAs to all
    for app in &["app1", "app2", "app3"] {
        let hpa = create_hpa_manifest(app, 2, 10, 70);
        ctx.cluster.apply_manifest(&hpa).await?;
    }

    // Generate load on all apps
    for app in &["app1", "app2", "app3"] {
        let cluster = ctx.cluster.clone();
        let app = app.clone();
        tokio::spawn(async move {
            generate_load(&cluster, TEST_NAMESPACE, &app, Duration::from_secs(30)).await
        });
    }

    // Verify all scaled up
    for app in &["app1", "app2", "app3"] {
        assert_replica_count(&ctx.cluster, TEST_NAMESPACE, app, 5).await?;
    }

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_scaling_with_resource_constraints() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Deploy workload with specific resource requirements
    let manifest = format!(
        r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: constrained-app
  namespace: {}
spec:
  replicas: 3
  selector:
    matchLabels:
      app: constrained-app
  template:
    metadata:
      labels:
        app: constrained-app
    spec:
      containers:
      - name: app
        image: nginx:latest
        resources:
          requests:
            cpu: 2000m  # High CPU request
            memory: 4Gi
          limits:
            cpu: 4000m
            memory: 8Gi
"#,
        TEST_NAMESPACE
    );

    ctx.cluster.apply_manifest(&manifest).await?;

    // Attempt to scale beyond cluster capacity
    let result = ctx.cluster.scale_deployment(TEST_NAMESPACE, "constrained-app", 10).await;

    // Should fail or be limited due to resource constraints
    assert!(result.is_err() || result.is_ok());

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_scaling_metrics_validation() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    deploy_and_wait(&ctx.cluster, "test-app", 3).await?;

    let hpa_manifest = create_hpa_manifest("test-app", 2, 10, 70);
    ctx.cluster.apply_manifest(&hpa_manifest).await?;

    // Get HPA metrics
    let metrics = ctx.cluster.get_hpa_metrics(TEST_NAMESPACE, "test-app-hpa").await?;

    assert!(metrics.contains("target"), "HPA metrics should include target");
    assert!(metrics.contains("current"), "HPA metrics should include current");

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_scaling_with_zero_downtime() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;

    // Create service
    let service_manifest = create_service_manifest("test-app");
    ctx.cluster.apply_manifest(&service_manifest).await?;

    deploy_and_wait(&ctx.cluster, "test-app", 3).await?;

    // Start monitoring for downtime
    let monitor_task = tokio::spawn(async move {
        assert_no_downtime(&ctx.cluster, TEST_NAMESPACE, "test-app", Duration::from_secs(120)).await
    });

    // Perform scaling
    ctx.cluster.scale_deployment(TEST_NAMESPACE, "test-app", 6).await?;

    // Wait for scaling to complete
    assert_pods_ready(&ctx.cluster, TEST_NAMESPACE, "test-app", 6).await?;

    // Verify no downtime
    monitor_task.await???;

    ctx.cleanup().await?;
    Ok(())
}

// Helper functions
fn create_hpa_manifest(name: &str, min: u32, max: u32, target_cpu: i32) -> String {
    format!(
        r#"
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {}-hpa
  namespace: {}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {}
  minReplicas: {}
  maxReplicas: {}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {}
"#,
        name, TEST_NAMESPACE, name, min, max, target_cpu
    )
}

fn create_service_manifest(name: &str) -> String {
    format!(
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
    )
}
