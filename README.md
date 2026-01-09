# Cluster Orchestrator

Production-grade Kubernetes cluster orchestration and management library for Rust.

## Features

- **Cluster Management**: Full lifecycle management of Kubernetes clusters
- **Auto-Scaling**: Horizontal and vertical autoscaling with custom policies
- **Self-Healing**: Automatic failure detection and recovery
- **Resource Optimization**: Intelligent rightsizing and cost optimization
- **Multi-Cluster**: Federation and management across multiple clusters
- **Chaos Engineering**: Built-in chaos testing support

## Documentation

- [Test Suite](docs/TEST_SUITE.md) - Comprehensive testing overview
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Testing philosophy and approach
- [Chaos Tests](docs/CHAOS_TESTS.md) - Chaos engineering guide

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cluster-orchestrator = "0.1"
```

### Basic Usage

```rust
use cluster_orchestrator::{ClusterManager, Scaler, ScalingPolicy};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize cluster manager
    let config = ClusterConfig::from_file("config.yaml")?;
    let cluster = ClusterManager::new(config).connect().await?;

    // Deploy application
    cluster.deploy_application("app.yaml", "default").await?;

    // Setup autoscaling
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 10,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    let scaler = Scaler::new(cluster.clone(), policy);
    scaler.enable("my-app", "default").await?;

    Ok(())
}
```

## Testing

### Prerequisites

1. **kind** (Kubernetes in Docker) - for test clusters
2. **kubectl** - for cluster operations
3. **Chaos Mesh** - for chaos engineering tests (optional)

### Setup

```bash
# Install test cluster
./scripts/setup-test-cluster.sh

# Install Chaos Mesh (optional, for chaos tests)
./scripts/install-chaos-mesh.sh
```

### Running Tests

```bash
# Run all tests
./scripts/run-tests.sh all

# Run specific test types
./scripts/run-tests.sh unit          # Unit tests only
./scripts/run-tests.sh integration   # Integration tests only
./scripts/run-tests.sh chaos         # Chaos engineering tests
./scripts/run-tests.sh bench         # Performance benchmarks

# Or use cargo directly
cargo test --lib                     # Unit tests
cargo test --test '*'                # Integration tests
cargo bench                          # Benchmarks
```

### Test Coverage

| Component | Unit | Integration | Chaos |
|-----------|------|-------------|-------|
| Cluster Manager | 95% | 80% | 60% |
| Scaling | 95% | 85% | 70% |
| Self-Healing | 90% | 80% | 90% |
| Resource Optimizer | 90% | 75% | 50% |
| Multi-Cluster | 85% | 80% | 60% |

## Performance SLAs

The library is designed to meet the following performance targets:

- **Scale-up latency**: < 1 minute (p95)
- **Scale-down latency**: < 45 seconds (p95)
- **Failure detection**: < 10 seconds (p95)
- **Cluster recovery**: < 5 minutes (p95)
- **Resource optimization**: < 500ms (p95)

Run benchmarks to verify performance:

```bash
cargo bench
```

## Architecture

```
cluster-orchestrator/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── cluster.rs          # Cluster management
│   ├── scaling.rs          # Autoscaling logic
│   ├── healing.rs          # Self-healing logic
│   ├── optimizer.rs        # Resource optimization
│   ├── multicluster.rs     # Multi-cluster management
│   ├── metrics.rs          # Metrics collection
│   └── config.rs           # Configuration
├── tests/
│   ├── unit/               # Unit tests
│   ├── integration/        # Integration tests
│   ├── chaos/              # Chaos engineering tests
│   └── common/             # Test utilities
├── benches/                # Performance benchmarks
├── docs/                   # Documentation
├── examples/               # Usage examples
└── scripts/                # Utility scripts
```

## Configuration

Example cluster configuration (`config.yaml`):

```yaml
cluster:
  name: "my-cluster"
  kubeconfig: "~/.kube/config"
  context: "default"

scaling:
  enabled: true
  default_policy:
    min_replicas: 2
    max_replicas: 10
    target_cpu_utilization: 70
    target_memory_utilization: 80
    stabilization_window_seconds: 300

healing:
  enabled: true
  check_interval_seconds: 10
  max_retries: 3
  circuit_breaker_threshold: 5

optimization:
  enabled: true
  strategy: "balanced"  # performance | cost | balanced
  check_interval_seconds: 300
```

## Examples

See the [examples](examples/) directory for complete examples:

- `basic_cluster_management.rs` - Basic cluster operations
- `autoscaling.rs` - Autoscaling setup
- `self_healing.rs` - Self-healing configuration
- `multi_cluster.rs` - Multi-cluster management

## Development

### Building

```bash
cargo build
cargo build --release
```

### Linting

```bash
cargo clippy --all-targets -- -D warnings
```

### Formatting

```bash
cargo fmt --all
```

### Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --lib --out Html
```

## CI/CD

The project uses GitHub Actions with the following pipeline:

1. **Fast Checks** (5 min) - Run on every PR
   - Formatting checks
   - Linting
   - Unit tests

2. **Full Tests** (20 min) - Run before merge
   - Integration tests
   - Small-scale chaos tests
   - Performance regression tests

3. **Chaos Tests** (45 min) - Run on main branch
   - Full chaos engineering suite
   - Extended performance tests
   - Stress tests

4. **Nightly Tests** (2 hours)
   - Comprehensive chaos scenarios
   - Long-running stability tests
   - Multi-scale performance tests

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests (unit, integration, chaos as appropriate)
5. Ensure all tests pass
6. Submit a pull request

### Testing Guidelines

- Maintain > 90% code coverage
- Write unit tests for all business logic
- Add integration tests for workflows
- Include chaos tests for resilience
- Update benchmarks for performance changes

## License

MIT License - see LICENSE file for details

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions

## Performance Benchmarks

Latest benchmarks (run on `main` branch):

```
scaling_decision              time:   [45.23 µs 46.12 µs 47.05 µs]
replica_calculation           time:   [12.45 µs 12.67 µs 12.91 µs]
health_check                  time:   [8.234 µs 8.345 µs 8.467 µs]
optimization_recommendation   time:   [234.5 µs 236.7 µs 239.1 µs]
bin_packing_10_workloads     time:   [1.2345 ms 1.2456 ms 1.2567 ms]
```

## Roadmap

- [ ] Advanced scheduling algorithms
- [ ] Machine learning-based optimization
- [ ] Multi-cloud provider support
- [ ] Web UI for cluster management
- [ ] Custom resource definitions (CRDs)
- [ ] Event-driven scaling
- [ ] Predictive autoscaling
