# Chaos Engineering Test Suite

## Overview

Chaos engineering tests validate the resilience and self-healing capabilities of the cluster orchestrator under real-world failure conditions.

## Infrastructure Requirements

### Tools
- **Chaos Mesh**: Primary chaos engineering platform
- **kind/k3d**: Local Kubernetes clusters for testing
- **Prometheus**: Metrics collection and monitoring
- **Grafana**: Visualization and dashboards

### Installation
```bash
# Install Chaos Mesh
curl -sSL https://mirrors.chaos-mesh.org/v0.25.0/install.sh | bash

# Install to test cluster
kubectl apply -f chaos-mesh/manifests.yaml

# Verify installation
kubectl get pods -n chaos-testing
```

## Chaos Test Categories

### 1. Pod Chaos Tests

#### 1.1 Pod Kill Tests
**File**: `tests/chaos/pod_chaos.rs`

**Purpose**: Validate self-healing when pods are terminated

**Scenarios**:
```rust
#[tokio::test]
#[ignore]
async fn test_pod_kill_single() {
    // Kill single pod, verify recovery
}

#[tokio::test]
#[ignore]
async fn test_pod_kill_multiple() {
    // Kill multiple pods simultaneously
}

#[tokio::test]
#[ignore]
async fn test_pod_kill_continuous() {
    // Continuously kill pods over time
}

#[tokio::test]
#[ignore]
async fn test_pod_kill_all_replicas() {
    // Kill all replicas (catastrophic failure)
}
```

**Validation**:
- Pods are recreated within SLA
- No service disruption
- Correct replica count maintained
- Recovery time < 60s per pod

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-kill-test
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - default
    labelSelectors:
      app: test-app
  scheduler:
    cron: "@every 10s"
```

#### 1.2 Pod Failure Tests
**Purpose**: Validate handling of container failures

**Scenarios**:
- Container exit with error code
- Container OOM killed
- Container crash loop
- Image pull failures

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-failure-test
spec:
  action: container-kill
  mode: fixed-percent
  value: "50"
  containerNames:
    - app
  selector:
    labelSelectors:
      app: test-app
```

### 2. Network Chaos Tests

#### 2.1 Network Partition Tests
**File**: `tests/chaos/network_chaos.rs`

**Purpose**: Validate behavior during network failures

**Scenarios**:
```rust
#[tokio::test]
#[ignore]
async fn test_network_partition_pod_to_pod() {
    // Isolate pods from each other
}

#[tokio::test]
#[ignore]
async fn test_network_partition_pod_to_service() {
    // Isolate pods from services
}

#[tokio::test]
#[ignore]
async fn test_network_dns_failure() {
    // Inject DNS resolution failures
}

#[tokio::test]
#[ignore]
async fn test_network_partial_loss() {
    // Inject packet loss
}
```

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: network-partition
spec:
  action: partition
  mode: all
  selector:
    labelSelectors:
      app: test-app
  direction: to
  target:
    selector:
      labelSelectors:
        app: database
```

#### 2.2 Network Delay Tests
**Purpose**: Validate handling of high latency

**Scenarios**:
- Fixed latency injection (100ms, 500ms, 1s)
- Jitter injection (latency variance)
- Correlation preservation (maintain flow patterns)

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: network-delay
spec:
  action: delay
  mode: fixed-percent
  value: "100"
  delay:
    latency: "500ms"
    jitter: "100ms"
    correlation: "50"
  selector:
    labelSelectors:
      app: test-app
```

#### 2.3 Packet Loss Tests
**Purpose**: Validate handling of unreliable networks

**Scenarios**:
- Low packet loss (1%)
- Medium packet loss (5%)
- High packet loss (10%)
- Extreme packet loss (25%)

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: packet-loss
spec:
  action: loss
  mode: all
  loss:
    loss: "10"
    correlation: "25"
  selector:
    labelSelectors:
      app: test-app
```

### 3. Node Chaos Tests

#### 3.1 Node Failure Tests
**File**: `tests/chaos/node_chaos.rs`

**Purpose**: Validate cluster behavior during node failures

**Scenarios**:
```rust
#[tokio::test]
#[ignore]
async fn test_node_not_ready() {
    // Simulate node NotReady state
}

#[tokio::test]
#[ignore]
async fn test_node_resource_exhaustion() {
    // Simulate node resource exhaustion
}

#[tokio::test]
#[ignore]
async fn test_node_network_failure() {
    // Simulate node network isolation
}

#[tokio::test]
#[ignore]
async fn test_node_multiple_failures() {
    // Simulate multiple node failures
}
```

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NodeChaos
metadata:
  name: node-failure
spec:
  action: node-failure
  mode: one
  selector:
    nodes:
      - worker-1
```

#### 3.2 Node Stress Tests
**Purpose**: Validate behavior under node stress

**Scenarios**:
- CPU stress (100% utilization)
- Memory stress (90% utilization)
- Disk I/O stress (high IOPS)
- Network stress (bandwidth saturation)

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: StressChaos
metadata:
  name: node-stress
spec:
  mode: one
  stressors:
    cpu:
      workers: 4
      load: 100
    memory:
      size: "1GB"
  selector:
    labelSelectors:
      node-role: worker
```

### 4. IO Chaos Tests

#### 4.1 Disk Failure Tests
**File**: `tests/chaos/io_chaos.rs`

**Purpose**: Validate handling of disk failures

**Scenarios**:
- Disk read errors
- Disk write errors
- Disk latency spikes
- Disk full scenarios

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: IOChaos
metadata:
  name: disk-failure
spec:
  action: diskFailure
  mode: all
  delay: "100ms"
  errno: 5
  selector:
    labelSelectors:
      app: database
  volumePath: /data
```

### 5. Fault Injection Tests

#### 5.1 HTTP Fault Tests
**File**: `tests/chaos/fault_injection.rs`

**Purpose**: Validate handling of API failures

**Scenarios**:
```rust
#[tokio::test]
#[ignore]
async fn test_api_server_500_errors() {
    // Inject 500 errors from API server
}

#[tokio::test]
#[ignore]
async fn test_api_server_abort() {
    // Abort API connections
}

#[tokio::test]
#[ignore]
async fn test_api_rate_limit() {
    // Inject rate limiting
}

#[tokio::test]
#[ignore]
async fn test_api_timeout() {
    // Inject timeouts
}
```

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: HTTPChaos
metadata:
  name: api-abort
spec:
  mode: all
  target: Request
  port: 8080
  path: "/api/v1/*"
  method: GET
  abort: true
  selector:
    labelSelectors:
      app: api-server
```

### 6. Stress Tests

#### 6.1 High Load Tests
**File**: `tests/chaos/stress_tests.rs`

**Purpose**: Validate system under extreme load

**Scenarios**:
```rust
#[tokio::test]
#[ignore]
async fn test_concurrent_scaling_operations() {
    // Trigger 100 scaling operations concurrently
}

#[tokio::test]
#[ignore]
async fn test_rapid_pod_churn() {
    // Create and delete pods rapidly
}

#[tokio::test]
#[ignore]
async fn test_memory_pressure() {
    // Consume all available memory
}

#[tokio::test]
#[ignore]
async fn test_cpu_saturation() {
    // Max out CPU usage
}
```

**Validation**:
- System remains responsive
- No deadlocks or crashes
- Graceful degradation
- Recovery when stress stops

### 7. Time Chaos Tests

#### 7.1 Clock Skew Tests
**Purpose**: Validate handling of time synchronization issues

**Scenarios**:
- Clock skew forward (+10s)
- Clock skew backward (-10s)
- Clock drift over time

**Chaos Spec**:
```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: TimeChaos
metadata:
  name: clock-skew
spec:
  mode: all
  timeOffset: "-10s"
  selector:
    labelSelectors:
      app: time-sensitive
```

## Test Execution Framework

### Test Wrapper

```rust
use std::time::Duration;
use tokio::time::sleep;

pub struct ChaosTest {
    cluster: TestCluster,
    chaos_spec: ChaosSpec,
    timeout: Duration,
}

impl ChaosTest {
    pub async fn run<F, Fut>(test_name: &str, test_fn: F) -> TestResult
    where
        F: FnOnce(TestCluster) -> Fut,
        Fut: Future<Output = anyhow::Result<()>>,
    {
        let cluster = setup_test_cluster().await?;

        // Inject chaos
        let chaos = inject_chaos(&cluster, &chaos_spec).await?;

        // Run test
        let result = tokio::time::timeout(timeout, test_fn(cluster.clone())).await;

        // Cleanup chaos
        cleanup_chaos(chaos).await?;

        // Cleanup cluster
        cleanup_test_cluster(cluster).await?;

        result?
    }
}
```

### Chaos Test Macro

```rust
macro_rules! chaos_test {
    ($name:ident, $chaos_spec:expr, $timeout:expr, $test_body:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() -> anyhow::Result<()> {
            let test = ChaosTest::new($chaos_spec, $timeout);
            test.run(stringify!($name), $test_body).await
        }
    };
}

// Usage
chaos_test!(
    test_pod_kill_recovery,
    ChaosSpec::pod_kill("test-app", 1),
    Duration::from_secs(60),
    |cluster| async move {
        assert_pods_recovered(&cluster, "test-app", 3).await?;
        Ok(())
    }
);
```

## Metrics and Validation

### Recovery Time Measurement

```rust
pub struct RecoveryMetrics {
    pub detection_time: Duration,
    pub action_time: Duration,
    pub total_recovery_time: Duration,
    pub success: bool,
}

pub async fn measure_recovery(
    cluster: &TestCluster,
    deployment: &str,
    expected_replicas: u32,
) -> RecoveryMetrics {
    let start = Instant::now();

    // Measure detection time
    let detection_start = Instant::now();
    wait_for_failure_detection(cluster, deployment).await;
    let detection_time = detection_start.elapsed();

    // Measure action time
    let action_start = Instant::now();
    wait_for_pods_ready(cluster, deployment, expected_replicas).await;
    let action_time = action_start.elapsed();

    RecoveryMetrics {
        detection_time,
        action_time,
        total_recovery_time: start.elapsed(),
        success: true,
    }
}
```

### SLA Validation

```rust
pub fn validate_sla(metrics: &RecoveryMetrics) -> Result<(), String> {
    if metrics.total_recovery_time > Duration::from_secs(300) {
        return Err(format!(
            "Recovery time {:?} exceeds SLA of 300s",
            metrics.total_recovery_time
        ));
    }

    if metrics.detection_time > Duration::from_secs(10) {
        return Err(format!(
            "Detection time {:?} exceeds SLA of 10s",
            metrics.detection_time
        ));
    }

    Ok(())
}
```

## Chaos Test Scenarios

### Scenario 1: Rolling Pod Kill During Scale-Up

**Objective**: Validate scaling continues despite pod failures

**Steps**:
1. Start with 3 replicas
2. Trigger scale-up to 10 replicas
3. Kill 2 pods during scale-up
4. Verify all 10 replicas eventually ready

**Expected**: 10 replicas ready within 90s

### Scenario 2: Network Partition During Healing

**Objective**: Validate healing works with partial network

**Steps**:
1. Create deployment with 5 replicas
2. Kill 3 pods
3. Network partition remaining pods
4. Verify healing continues despite partition

**Expected**: All pods recovered, system recovers when partition ends

### Scenario 3: Node Failure During Optimization

**Objective**: Validate resource optimization handles node loss

**Steps**:
1. Deploy workloads across cluster
2. Trigger resource optimization
3. Fail a node during optimization
3. Verify optimization completes, pods rescheduled

**Expected**: Optimization completes, pods placed on remaining nodes

### Scenario 4: Multi-Cluster Failover

**Objective**: Validate multi-cluster failover mechanism

**Steps**:
1. Deploy to two clusters
2. Configure failover policy
3. Fail primary cluster
4. Verify traffic shifts to secondary

**Expected**: Traffic fails over within 30s, zero data loss

### Scenario 5: Cascading Failures

**Objective**: Validate system doesn't fail catastrophically

**Steps**:
1. Deploy critical services
2. Kill database pods
3. Kill cache pods (depends on DB)
4. Kill app pods (depends on cache)
5. Verify graceful degradation

**Expected**: Circuit breakers prevent cascade, graceful recovery

## Continuous Chaos Testing

### Scheduled Chaos

**CI Integration**:
```yaml
# .github/workflows/chaos.yml
name: Chaos Tests

on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily
  workflow_dispatch:

jobs:
  chaos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup kind cluster
        run: ./scripts/setup-kind-cluster.sh
      - name: Install Chaos Mesh
        run: ./scripts/install-chaos-mesh.sh
      - name: Run chaos tests
        run: cargo test --test chaos -- --chaos
```

### Chaos in Production

**Controlled Chaos**:
- Run during low-traffic windows
- Use blast radius (single namespace/cluster)
- Gradual rollout of chaos scenarios
- Always have manual abort

**Monitoring During Chaos**:
- Real-time dashboards
- Alert on SLA violations
- Auto-abort on error rate spike
- Detailed logging

## Best Practices

### 1. Start Small
- Begin with single pod failures
- Progress to multiple failures
- End with catastrophic scenarios

### 2. Measure Everything
- Recovery time
- Error rates
- Resource utilization
- User impact

### 3. Document Findings
- Record each chaos experiment
- Document recovery behavior
- Track improvements over time
- Share lessons learned

### 4. Gradual Ramp-Up
- Start with low chaos intensity
- Increase gradually
- Monitor system health
- Stop on degradation

### 5. Blast Radius Control
- Limit chaos to test namespaces
- Use percentage-based chaos
- Implement circuit breakers
- Always have abort mechanism

## Troubleshooting

### Chaos Tests Not Failing

**Issue**: Tests pass but chaos doesn't execute

**Solutions**:
- Verify Chaos Mesh is installed
- Check chaos CRD status
- Verify pod selectors match
- Check chaos mesh logs

### Tests Are Too Flaky

**Issue**: Chaos tests have inconsistent results

**Solutions**:
- Increase timeout values
- Add proper wait conditions
- Improve test isolation
- Fix race conditions

### Recovery Too Slow

**Issue**: SLA violations during chaos

**Solutions**:
- Tune healing parameters
- Increase replica counts
- Optimize pod startup time
- Review resource limits

## Safety Measures

### Pre-Flight Checks
```rust
pub async fn pre_flight_checks(cluster: &TestCluster) -> Result<()> {
    // Ensure cluster is healthy
    assert_cluster_healthy(cluster).await?;

    // Verify no existing chaos
    assert_no_active_chaos(cluster).await?;

    // Check sufficient resources
    assert_sufficient_resources(cluster).await?;

    Ok(())
}
```

### Abort Conditions
- Error rate > 10%
- Recovery time > 10 minutes
- More than 50% pods down
- Control plane unresponsive

### Recovery Mechanisms
```bash
# Emergency chaos cleanup
kubectl delete chaos --all -n chaos-testing

# Restore cluster state
./scripts/restore-cluster.sh

# Emergency scaling
kubectl scale deployment --all --replicas=3
```
