//! Helper functions for tests

use std::time::Duration;
use tokio::time::sleep;

/// Create a test deployment manifest
pub fn create_deployment_manifest(
    name: &str,
    image: &str,
    replicas: u32,
) -> String {
    format!(
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
      - name: {}
        image: {}
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
"#,
        name, crate::common::TEST_NAMESPACE, replicas, name, name, name, image
    )
}

/// Create a test service manifest
pub fn create_service_manifest(name: &str) -> String {
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
  type: ClusterIP
"#,
        name, crate::common::TEST_NAMESPACE, name
    )
}

/// Create a HorizontalPodAutoscaler manifest
pub fn create_hpa_manifest(
    name: &str,
    min_replicas: u32,
    max_replicas: u32,
    target_cpu: i32,
) -> String {
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
        name, crate::common::TEST_NAMESPACE, name, min_replicas, max_replicas, target_cpu
    )
}

/// Apply a deployment and wait for it to be ready
pub async fn deploy_and_wait(
    cluster: &crate::common::TestCluster,
    name: &str,
    replicas: u32,
) -> anyhow::Result<()> {
    let manifest = create_deployment_manifest(name, "nginx:latest", replicas);
    cluster.apply_manifest(&manifest).await?;

    // Wait for deployment to be ready
    crate::common::assertions::assert_pods_ready(
        cluster,
        crate::common::TEST_NAMESPACE,
        name,
        replicas,
    ).await?;

    Ok(())
}

/// Generate load against a deployment
pub async fn generate_load(
    cluster: &crate::common::TestCluster,
    namespace: &str,
    service: &str,
    duration: Duration,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    while start.elapsed() < duration {
        // Use kubectl to port-forward and make requests
        let output = tokio::process::Command::new("kubectl")
            .args([
                "exec",
                "-n",
                namespace,
                "deploy/" /// service,
                "--",
                "curl",
                "-s",
                &format!("http://{}", service),
            ])
            .env("KUBECONFIG", &cluster.kubeconfig)
            .output()
            .await?;

        // Sleep a bit between requests
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Measure operation duration
pub async fn measure_duration<F, Fut, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (result, duration)
}

/// Retry with exponential backoff
pub async fn retry_exponential<F, Fut, T, E>(
    operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

/// Wait for condition with custom timeout
pub async fn wait_with_timeout<F, Fut>(
    condition: F,
    timeout: Duration,
    check_interval: Duration,
) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        sleep(check_interval).await;
    }

    anyhow::bail!("Condition not met within {:?}", timeout);
}
