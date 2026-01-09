// Criterion benchmark for scaling performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use cluster_orchestrator::scaling::{Scaler, ScalingPolicy};
use std::time::Duration;

fn bench_scaling_decision(c: &mut Criterion) {
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 100,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

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

fn bench_scaling_decision_various_loads(c: &mut Criterion) {
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 100,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    let mut group = c.benchmark_group("scaling_decision_by_load");

    for cpu_util in [20.0, 50.0, 70.0, 85.0, 95.0].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(cpu_util),
            cpu_util,
            |b, &cpu| {
                b.iter(|| {
                    Scaler::evaluate_scaling(
                        black_box(&policy),
                        black_box(10),
                        black_box(cpu),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_scaling_replica_calculation(c: &mut Criterion) {
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 100,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    c.bench_function("replica_calculation", |b| {
        b.iter(|| {
            Scaler::calculate_replicas(
                black_box(&policy),
                black_box(10),
                black_box(8.5),
                black_box(85.0),
            )
        });
    });
}

fn bench_batch_scaling_decisions(c: &mut Criterion) {
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 100,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    let mut group = c.benchmark_group("batch_scaling");
    group.throughput(Throughput::Elements(100));

    group.bench_function("batch_100", |b| {
        b.iter(|| {
            (0..100).map(|i| {
                Scaler::evaluate_scaling(
                    black_box(&policy),
                    black_box(10),
                    black_box(50.0 + (i as f64 * 0.5)),
                )
            }).collect::<Vec<_>>()
        });
    });

    group.finish();
}

fn bench_multi_metric_scaling(c: &mut Criterion) {
    let policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 100,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    c.bench_function("multi_metric_scaling", |b| {
        b.iter(|| {
            Scaler::evaluate_scaling_multi(
                black_box(&policy),
                black_box(10),
                black_box(85.0),
                black_box(75.0),
            )
        });
    });
}

criterion_group!(
    benches,
    bench_scaling_decision,
    bench_scaling_decision_various_loads,
    bench_scaling_replica_calculation,
    bench_batch_scaling_decisions,
    bench_multi_metric_scaling
);

criterion_main!(benches);
