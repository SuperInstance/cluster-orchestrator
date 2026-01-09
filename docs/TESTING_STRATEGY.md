# Testing Strategy for Cluster Orchestrator

## Philosophy

Our testing strategy follows the **Testing Pyramid** approach with emphasis on:

1. **Fast Feedback**: Unit tests provide immediate feedback
2. **Confidence**: Integration tests validate real-world scenarios
3. **Resilience**: Chaos tests ensure production readiness
4. **Performance**: Benchmarks guarantee SLA compliance

## Testing Principles

### 1. Test Isolation
- Each test should be independent
- No shared state between tests
- Deterministic outcomes
- Clean setup/teardown

### 2. Test Speed
- Unit tests: < 100ms each
- Integration tests: < 10s each
- Chaos tests: < 5 min each
- Performance tests: Measured, not limited

### 3. Test Reliability
- No flaky tests in CI
- Retry logic for network tests
- Timeout protection
- Clear failure messages

### 4. Test Maintainability
- Clear test names
- Self-documenting code
- Reusable test helpers
- Good fixture organization

## Test Organization

### Directory Structure
```
tests/
├── unit/                    # Fast, isolated unit tests
│   ├── cluster_tests.rs
│   ├── scaling_tests.rs
│   ├── healing_tests.rs
│   ├── optimizer_tests.rs
│   └── multicluster_tests.rs
├── integration/             # Component integration tests
│   ├── cluster_lifecycle.rs
│   ├── scaling_integration.rs
│   ├── healing_integration.rs
│   └── optimizer_integration.rs
├── chaos/                   # Chaos engineering tests
│   ├── pod_chaos.rs
│   ├── network_chaos.rs
│   ├── node_chaos.rs
│   ├── stress_tests.rs
│   └── fault_injection.rs
├── common/                  # Shared test utilities
│   ├── mod.rs
│   ├── cluster_setup.rs
│   ├── assertions.rs
│   └── helpers.rs
└── fixtures/                # Test data and configurations
    ├── clusters/
    ├── deployments/
    └── chaos_scenarios/
```

## Test Categories

### Unit Tests

**Purpose**: Validate business logic in isolation

**Characteristics**:
- Fast execution (< 5s total)
- No external dependencies
- Mocked Kubernetes API
- High code coverage (> 90%)

**What to Test**:
- Pure functions and algorithms
- State machines
- Error handling paths
- Edge cases and boundary conditions
- Data transformations

**What NOT to Test**:
- External API calls (mock them)
- Database operations (mock them)
- Third-party libraries (trust them)
- Trivial getters/setters

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_scaling_decision_increase() {
        let mut mock_client = MockKubeClient::new();
        mock_client
            .expect_get_metrics()
            .returning(|| Ok(vec![metric_at_80_percent()]));

        let scaler = Scaler::new(mock_client);
        let decision = scaler.evaluate_scaling(&policy).unwrap();

        assert_eq!(decision.direction, ScalingDirection::Up);
        assert_eq!(decision.replicas, 5);
    }
}
```

### Integration Tests

**Purpose**: Validate component interactions

**Characteristics**:
- Medium execution time (< 15 min total)
- Real Kubernetes cluster (kind/k3d)
- Real dependencies
- End-to-end workflows

**What to Test**:
- API interactions
- Workflow completion
- State transitions
- Error recovery
- Configuration management

**Example**:
```rust
#[tokio::test]
#[ignore] // Run only with --test-threads=1
async fn test_scale_up_workflow() {
    let cluster = setup_test_cluster().await;
    let deployment = create_test_deployment(&cluster).await;

    // Trigger scale-up
    cluster.apply_scaling("test-app", 5).await.unwrap();

    // Verify new pods
    wait_for_pod_count(&cluster, "test-app", 5).await;

    // Verify all pods ready
    assert_all_pods_ready(&cluster, "test-app").await;

    cleanup_test_cluster(cluster).await;
}
```

### Chaos Engineering Tests

**Purpose**: Validate system resilience

**Characteristics**:
- Longer execution time
- Real failure scenarios
- Production-like conditions
- Automated recovery validation

**What to Test**:
- Pod failures
- Network failures
- Node failures
- Resource exhaustion
- API server failures
- Concurrent failures

**Chaos Levels**:

**Level 1: Component Failures**
- Single pod failures
- Single container failures
- Network latency spikes
- DNS resolution failures

**Level 2: Node Failures**
- Node not ready
- Node resource exhaustion
- Node network failures
- Node daemon failures

**Level 3: Infrastructure Failures**
- API server failures
- Etcd leader loss
- Network partitions
- Multi-node failures

**Level 4: Catastrophic Failures**
- Cluster-wide failures
- Multi-cluster failures
- Network isolation
- Total control plane loss

**Example**:
```rust
#[tokio::test]
#[ignore]
async fn test_pod_kill_recovery() {
    let cluster = setup_test_cluster().await;
    let deployment = create_test_deployment(&cluster, 3).await;

    // Inject chaos: kill pods
    inject_pod_chaos(&cluster, "test-app", PodChaosAction::Kill).await;

    // Verify self-healing
    assert_pods_recovered(&cluster, "test-app", 3, Duration::from_secs(60)).await;

    // Verify no downtime
    assert_no_downtime(&cluster, "test-app").await;

    cleanup_test_cluster(cluster).await;
}
```

### Performance Tests

**Purpose**: Validate SLA compliance

**Characteristics**:
- Measured execution
- Benchmark comparisons
- Load testing
- Stress testing

**Performance SLAs**:
- Scale-up latency: < 1 minute (p95)
- Scale-down latency: < 45 seconds (p95)
- Failure detection: < 10 seconds (p95)
- Cluster recovery: < 5 minutes (p95)
- Resource optimization: < 500ms (p95)

**Example**:
```rust
#[bench]
fn bench_scaling_decision(b: &mut Bencher) {
    let scaler = setup_scaler();
    let policy = create_scaling_policy();

    b.iter(|| {
        scaler.evaluate_scaling(&policy)
    });
}
```

## Test Execution Strategy

### Local Development

**Quick Feedback Loop**:
```bash
# Run unit tests only (fast)
cargo test --lib

# Run unit tests with watch
cargo watch -x 'test --lib'

# Run single test file
cargo test --test cluster_tests
```

**Before Commit**:
```bash
# Run full test suite
cargo test --all

# Run linting
cargo clippy --all-targets

# Check formatting
cargo fmt --all -- --check
```

### Continuous Integration

**Pipeline Stages**:

1. **Fast Checks** (5 min) - Run on every PR
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test --lib
   ```

2. **Integration Tests** (20 min) - Run before merge
   ```bash
   ./scripts/start-test-cluster.sh
   cargo test --test '*'
   ./scripts/stop-test-cluster.sh
   ```

3. **Chaos Tests** (45 min) - Run on main branch
   ```bash
   ./scripts/install-chaos-mesh.sh
   cargo test --test chaos -- --chaos
   ```

4. **Performance Tests** (30 min) - Run nightly
   ```bash
   cargo bench
   ./scripts/compare-benchmarks.sh
   ```

### Test Environments

**Local**:
- kind cluster for development
- Mock Kubernetes API for unit tests
- Real cluster for integration tests

**CI**:
- k3d cluster for speed
- Parallel test execution
- Artifact caching
- Test result persistence

**Staging**:
- Real GKE/AKS cluster
- Production-like configuration
- Full chaos test suite
- Performance validation

## Test Data Management

### Fixtures

**Organization**:
```
tests/fixtures/
├── clusters/
│   ├── small-cluster.yaml
│   ├── medium-cluster.yaml
│   └── large-cluster.yaml
├── deployments/
│   ├── simple-app.yaml
│   ├── stateful-app.yaml
│   └── daemon-app.yaml
└── chaos/
    ├── pod-kill-scenario.yaml
    ├── network-delay-scenario.yaml
    └── stress-scenario.yaml
```

**Guidelines**:
- Use realistic configurations
- Include edge cases
- Document fixture purpose
- Version with code
- Keep small and focused

### Test Data Generation

**Strategies**:
- Factory functions for test objects
- Random generation with proptest
- Fixture builders for complex objects
- Shared test context

**Example**:
```rust
pub struct TestDeploymentBuilder {
    name: String,
    replicas: u32,
    image: String,
    resources: Option<ResourceRequirements>,
}

impl TestDeploymentBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            replicas: 1,
            image: "nginx:latest".to_string(),
            resources: None,
        }
    }

    pub fn replicas(mut self, replicas: u32) -> Self {
        self.replicas = replicas;
        self
    }

    pub fn build(self) -> Deployment {
        // Create deployment spec
    }
}
```

## Mock Strategy

### Kubernetes API Mocking

**Use `mockall` for behavior-based mocking**:
```rust
#[automock]
trait KubeClient {
    async fn get_pods(&self, namespace: &str) -> Result<Vec<Pod>>;
    async fn create_deployment(&self, deployment: &Deployment) -> Result<Deployment>;
    async fn scale_deployment(&self, name: &str, replicas: u32) -> Result<Deployment>;
}
```

**Use `mockito` for HTTP mocking**:
```rust
#[tokio::test]
async fn test_api_error_handling() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/namespaces/default/pods")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    let client = KubeClient::new(server.url());
    let result = client.get_pods("default").await;

    assert!(result.is_err());

    mock.assert();
}
```

### Mock Scenarios

**Success Scenarios**:
- Normal API responses
- Expected resource states
- Successful operations

**Failure Scenarios**:
- Network errors
- API errors (4xx, 5xx)
- Timeout errors
- Resource not found

**Edge Cases**:
- Empty responses
- Malformed responses
- Large response payloads
- Rate limiting

## Test Utilities

### Custom Test Macros

```rust
// Integration test macro
macro_rules! integration_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            let _cluster = setup_test_cluster().await;
            $body
            cleanup_test_cluster(_cluster).await;
        }
    };
}

// Chaos test macro
macro_rules! chaos_test {
    ($name:ident, $chaos_type:expr, $body:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            let cluster = setup_test_cluster().await;
            let chaos = inject_chaos(&cluster, $chaos_type).await;
            $body
            cleanup_chaos(chaos).await;
            cleanup_test_cluster(cluster).await;
        }
    };
}
```

### Assertion Helpers

```rust
pub fn assert_scaling_decision(
    decision: &ScalingDecision,
    direction: ScalingDirection,
    replicas: u32,
) {
    assert_eq!(decision.direction, direction, "Scaling direction mismatch");
    assert_eq!(decision.replicas, replicas, "Replica count mismatch");
}

pub async fn assert_pods_ready(
    cluster: &TestCluster,
    deployment: &str,
    expected: u32,
) {
    let pods = cluster.get_pods(deployment).await.unwrap();
    let ready = pods.iter().filter(|p| p.ready()).count();
    assert_eq!(ready as u32, expected, "Expected {} ready pods", expected);
}
```

## Continuous Improvement

### Test Metrics

**Track**:
- Test execution time trends
- Flaky test rate
- Test failure patterns
- Coverage metrics
- Performance baselines

**Goals**:
- Maintain > 90% code coverage
- Zero flaky tests in CI
- < 10 min total test execution
- < 1% test failure rate

### Regular Review

**Weekly**:
- Review flaky tests
- Update performance baselines
- Audit test coverage

**Monthly**:
- Refresh chaos scenarios
- Update test fixtures
- Review test documentation
- Optimize slow tests

**Quarterly**:
- Major test suite updates
- New chaos scenarios
- Testing strategy review
- Tool upgrades

## Troubleshooting

### Common Issues

**Flaky Integration Tests**:
- Add proper wait conditions
- Increase timeout values
- Fix race conditions
- Improve test isolation

**Slow Tests**:
- Mock expensive operations
- Parallelize independent tests
- Optimize test data setup
- Cache test resources

**Chaos Test Failures**:
- Verify chaos injection worked
- Check recovery timeouts
- Review healing logic
- Update test expectations

### Debugging Tips

**Enable Logging**:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

**Single Test Execution**:
```bash
cargo test test_name -- --exact --nocapture
```

**Backtrace**:
```bash
RUST_BACKTRACE=1 cargo test
```

**Test Output Capture**:
```bash
cargo test -- --show-output
```
