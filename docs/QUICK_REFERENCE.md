# Cluster Orchestrator Test Suite - Quick Reference

## Quick Start

```bash
# Setup
./scripts/setup-test-cluster.sh

# Run all tests
./scripts/run-tests.sh all

# Run specific test types
cargo test --lib                    # Unit tests
cargo test --test '*'               # Integration tests
cargo test --test chaos -- --chaos  # Chaos tests
cargo bench                         # Benchmarks
```

## Test Structure

```
tests/
├── unit/                    # Fast, isolated tests (450+ tests)
│   ├── cluster_tests.rs     # Cluster management
│   ├── scaling_tests.rs     # Autoscaling logic
│   ├── healing_tests.rs     # Self-healing logic
│   └── optimizer_tests.rs   # Resource optimization
│
├── integration/             # End-to-end tests (50+ tests)
│   └── scaling_integration.rs
│
├── chaos/                   # Resilience tests (30+ tests)
│   └── pod_chaos.rs         # Pod failure scenarios
│
└── common/                  # Test utilities
    ├── mod.rs
    ├── cluster_setup.rs
    ├── assertions.rs
    ├── helpers.rs
    └── fixtures.rs
```

## Performance SLAs

| Operation | Target | Test |
|-----------|--------|------|
| Scale-up | < 1 min | `test_scale_up_latency()` |
| Scale-down | < 45 sec | `test_horizontal_scale_down()` |
| Failure detection | < 10 sec | Chaos recovery tests |
| Cluster recovery | < 5 min | `test_multiple_pod_kill_recovery()` |
| Scaling decision | < 100µs | `bench_scaling_decision()` |
| Resource optimization | < 500ms | `bench_optimization_recommendation()` |

## Coverage Targets

- **Cluster Manager**: 85% (95% unit, 80% integration, 60% chaos)
- **Scaling**: 90% (95% unit, 85% integration, 70% chaos)
- **Self-Healing**: 88% (90% unit, 80% integration, 90% chaos)
- **Resource Optimizer**: 80% (90% unit, 75% integration, 50% chaos)

## Test Categories

### Unit Tests (Fast)
- Business logic validation
- Mocked dependencies
- < 100ms per test
- Run on every PR

### Integration Tests (Medium)
- Real Kubernetes clusters (kind/k3d)
- End-to-end workflows
- < 10s per test
- Run before merge

### Chaos Tests (Slow)
- Real failure scenarios
- Chaos Mesh integration
- < 5 min per test
- Run on main branch

### Performance Tests (Measured)
- Benchmark-based
- SLA validation
- Criterion framework
- Run nightly

## Common Commands

```bash
# Specific test file
cargo test --test cluster_tests

# Specific test function
cargo test test_cluster_init

# With output
cargo test -- --nocapture

# With logging
RUST_LOG=debug cargo test

# With backtrace
RUST_BACKTRACE=1 cargo test

# Run ignored tests (integration/chaos)
cargo test -- --ignored

# Benchmarks
cargo bench
cargo bench --bench scaling

# Coverage
cargo tarpaulin --lib --out Html
```

## CI/CD Pipeline

1. **Fast Checks** (5 min) - Every PR
   - `cargo fmt --check`
   - `cargo clippy`
   - `cargo test --lib`

2. **Integration Tests** (20 min) - Pre-merge
   - `cargo test --test '*'`
   - Small chaos tests
   - Performance regression

3. **Chaos Tests** (45 min) - Main branch
   - Full chaos suite
   - Extended tests
   - Stress tests

4. **Nightly Tests** (2 hours)
   - Comprehensive scenarios
   - Long-running stability
   - Multi-scale performance

## Documentation

- `docs/TEST_SUITE.md` - Complete test suite documentation
- `docs/TESTING_STRATEGY.md` - Testing philosophy and approach
- `docs/CHAOS_TESTS.md` - Chaos engineering guide
- `docs/TEST_SUMMARY.md` - Implementation summary

## Test Utilities

### Assertions
```rust
assert_replica_count(&cluster, namespace, deployment, expected)
assert_pods_ready(&cluster, namespace, deployment, expected)
assert_cluster_healthy(&cluster)
assert_scaling_decision(current, desired, threshold, metric)
assert_recovery_time(actual, max_expected, operation)
assert_no_downtime(&cluster, namespace, service, duration)
```

### Helpers
```rust
deploy_and_wait(&cluster, name, replicas)
generate_load(&cluster, namespace, service, duration)
measure_duration(async { operation }).await
retry_exponential(operation, max_retries, delay).await
wait_with_timeout(condition, timeout, interval).await
```

### Fixtures
```rust
TestDeployment::new(name).with_replicas(5)
ScalingPolicyFixture::default()
MetricFixture::cpu_utilization(85.0)
ClusterStateFixture::healthy()
```

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=cluster_orchestrator=debug cargo test
```

### Specific Module Logging
```bash
RUST_LOG=cluster_orchestrator::scaling=trace cargo test
```

### Save Test Output
```bash
RUST_LOG=debug cargo test 2>&1 | tee test-output.log
```

## Test Examples

### Unit Test
```rust
#[test]
fn test_cluster_initialization() {
    let config = ClusterConfig::default();
    let manager = ClusterManager::new(config);

    assert_eq!(manager.node_count(), 0);
    assert_eq!(manager.state(), ClusterState::Initializing);
}
```

### Integration Test
```rust
#[tokio::test]
#[ignore]
async fn test_scale_up_latency() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    deploy_and_wait(&ctx.cluster, "test-app", 3).await?;

    let (result, duration) = measure_duration(
        ctx.cluster.scale_deployment(ns, "test-app", 5)
    ).await;

    result?;
    assert!(duration < Duration::from_secs(60));
    Ok(())
}
```

### Chaos Test
```rust
#[tokio::test]
#[ignore]
async fn test_pod_kill_recovery() -> anyhow::Result<()> {
    let ctx = TestContext::new().await?;
    deploy_test_app(&ctx.cluster, "test-app", 3).await?;

    let chaos = inject_pod_kill(&ctx.cluster, "test-app", 1).await?;
    tokio::time::sleep(Duration::from_secs(10)).await;

    assert_pods_ready(&ctx.cluster, ns, "test-app", 3).await?;
    cleanup_chaos(chaos).await?;

    Ok(())
}
```

### Benchmark
```rust
fn bench_scaling_decision(c: &mut Criterion) {
    let policy = ScalingPolicy::default();

    c.bench_function("scaling_decision", |b| {
        b.iter(|| {
            Scaler::evaluate_scaling(
                black_box(&policy),
                black_box(10),
                black_box(85.0),
            )
        });
    });
}
```

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Project configuration |
| `src/lib.rs` | Library entry point |
| `tests/common/mod.rs` | Test infrastructure |
| `tests/unit/*_tests.rs` | Unit tests |
| `tests/integration/*_integration.rs` | Integration tests |
| `tests/chaos/*_chaos.rs` | Chaos tests |
| `benches/*.rs` | Performance benchmarks |
| `docs/*.md` | Documentation |
| `scripts/*.sh` | Utility scripts |

## Troubleshooting

### Tests Not Running
```bash
# Check test cluster
kubectl cluster-info

# Restart cluster
kind delete cluster --name cluster-orchestrator-test
./scripts/setup-test-cluster.sh
```

### Chaos Tests Failing
```bash
# Verify Chaos Mesh
kubectl get pods -n chaos-testing

# Reinstall Chaos Mesh
./scripts/install-chaos-mesh.sh
```

### Slow Tests
```bash
# Run single test file
cargo test --test cluster_tests

# Run in parallel
cargo test -- --test-threads=4

# Skip slow tests
cargo test -- --skip chaos
```

## Best Practices

1. **Before Committing**
   ```bash
   cargo test --lib
   cargo clippy --all-targets
   cargo fmt --all
   ```

2. **Writing Tests**
   - Keep tests independent
   - Use descriptive names
   - Mock external dependencies in unit tests
   - Use real clusters in integration tests
   - Add assertions for all conditions

3. **Performance Tests**
   - Use `black_box()` to prevent optimization
   - Run multiple iterations
   - Compare against baselines
   - Document targets

4. **Chaos Tests**
   - Start with small failures
   - Gradually increase severity
   - Always cleanup chaos
   - Monitor during execution

## Statistics

- **Total Files**: 25
- **Unit Tests**: 450+
- **Integration Tests**: 50+
- **Chaos Tests**: 30+
- **Benchmarks**: 20+
- **Lines of Documentation**: 2000+
- **Overall Coverage**: 85%+

## Support

- **Documentation**: See `docs/` directory
- **Examples**: See `examples/` directory
- **Issues**: File GitHub issues
- **Discussions**: Use GitHub Discussions

---

**Last Updated**: 2026-01-08
**Version**: 0.1.0
**Status**: Production-Ready
