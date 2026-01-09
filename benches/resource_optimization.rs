// Criterion benchmark for resource optimization performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cluster_orchestrator::optimizer::{ResourceOptimizer, OptimizationStrategy};
use std::time::Duration;

fn bench_optimization_recommendation(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let usage = ResourceUsage {
        cpu_cores: 2.5,
        memory_gb: 10.0,
        request_cpu: 4.0,
        request_memory: 16.0,
        limit_cpu: 8.0,
        limit_memory: 32.0,
    };

    let mut group = c.benchmark_group("optimization_strategies");

    group.bench_function("performance", |b| {
        b.iter(|| {
            optimizer.recommend(
                black_box(&usage),
                black_box(OptimizationStrategy::Performance),
            )
        });
    });

    group.bench_function("cost", |b| {
        b.iter(|| {
            optimizer.recommend(
                black_box(&usage),
                black_box(OptimizationStrategy::Cost),
            )
        });
    });

    group.bench_function("balanced", |b| {
        b.iter(|| {
            optimizer.recommend(
                black_box(&usage),
                black_box(OptimizationStrategy::Balanced),
            )
        });
    });

    group.finish();
}

fn bench_right_sizing(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let current = ResourceLimits {
        cpu_request: 4.0,
        cpu_limit: 8.0,
        memory_request: 16.0,
        memory_limit: 32.0,
    };

    let usage = ResourceUsage {
        cpu_cores: 2.5,
        memory_gb: 10.0,
        request_cpu: 4.0,
        request_memory: 16.0,
        limit_cpu: 8.0,
        limit_memory: 32.0,
    };

    c.bench_function("right_sizing", |b| {
        b.iter(|| {
            optimizer.right_size(
                black_box(current.clone()),
                black_box(usage.clone()),
            )
        });
    });
}

fn bench_bin_packing(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let node_capacity = NodeCapacity {
        cpu_cores: 16.0,
        memory_gb: 64.0,
    };

    let workloads: Vec<Workload> = (0..10).map(|i| Workload {
        name: format!("app{}", i),
        cpu_request: 2.0,
        memory_request: 8.0,
    }).collect();

    c.bench_function("bin_packing_10_workloads", |b| {
        b.iter(|| {
            optimizer.bin_pack(
                black_box(node_capacity),
                black_box(workloads.clone()),
            )
        });
    });
}

fn bench_bin_packing_scalability(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let node_capacity = NodeCapacity {
        cpu_cores: 64.0,
        memory_gb: 256.0,
    };

    let mut group = c.benchmark_group("bin_packing_scalability");
    group.throughput(criterion::Throughput::Elements(1));

    for count in [10, 20, 50, 100].iter() {
        let workloads: Vec<Workload> = (0..*count).map(|i| Workload {
            name: format!("app{}", i),
            cpu_request: 2.0,
            memory_request: 8.0,
        }).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, _| {
                b.iter(|| {
                    optimizer.bin_pack(
                        black_box(node_capacity),
                        black_box(workloads.clone()),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_cost_calculation(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let current = ResourceLimits {
        cpu_request: 4.0,
        cpu_limit: 8.0,
        memory_request: 16.0,
        memory_limit: 32.0,
    };

    let optimized = ResourceLimits {
        cpu_request: 2.0,
        cpu_limit: 4.0,
        memory_request: 8.0,
        memory_limit: 16.0,
    };

    c.bench_function("cost_savings", |b| {
        b.iter(|| {
            optimizer.calculate_cost_savings(
                black_box(current.clone()),
                black_box(optimized.clone()),
            )
        });
    });
}

fn bench_headroom_calculation(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let usage = ResourceUsage {
        cpu_cores: 2.5,
        memory_gb: 10.0,
        request_cpu: 4.0,
        request_memory: 16.0,
        limit_cpu: 8.0,
        limit_memory: 32.0,
    };

    c.bench_function("headroom_calculation", |b| {
        b.iter(|| {
            optimizer.calculate_headroom(black_box(&usage))
        });
    });
}

fn bench_batch_optimization(c: &mut Criterion) {
    let optimizer = ResourceOptimizer::new();

    let usages: Vec<ResourceUsage> = (0..100).map(|_| ResourceUsage {
        cpu_cores: 2.5,
        memory_gb: 10.0,
        request_cpu: 4.0,
        request_memory: 16.0,
        limit_cpu: 8.0,
        limit_memory: 32.0,
    }).collect();

    let mut group = c.benchmark_group("batch_optimization");
    group.throughput(criterion::Throughput::Elements(100));

    group.bench_function("batch_100", |b| {
        b.iter(|| {
            black_box(&usages).iter()
                .map(|usage| optimizer.recommend(usage, OptimizationStrategy::Balanced))
                .collect::<Vec<_>>()
        });
    });

    group.finish();
}

// Stub implementations
#[derive(Debug, Clone, Copy)]
pub enum OptimizationStrategy {
    Performance,
    Cost,
    Balanced,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OptimizationAction {
    Increase,
    Decrease,
    Maintain,
}

#[derive(Clone)]
struct ResourceUsage {
    cpu_cores: f64,
    memory_gb: f64,
    request_cpu: f64,
    request_memory: f64,
    limit_cpu: f64,
    limit_memory: f64,
}

#[derive(Clone)]
struct ResourceLimits {
    cpu_request: f64,
    cpu_limit: f64,
    memory_request: f64,
    memory_limit: f64,
}

#[derive(Clone)]
struct NodeCapacity {
    cpu_cores: f64,
    memory_gb: f64,
}

#[derive(Clone)]
struct Workload {
    name: String,
    cpu_request: f64,
    memory_request: f64,
}

pub struct OptimizationRecommendation {
    pub action: OptimizationAction,
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub reason: String,
}

struct ResourceHeadroom {
    cpu_headroom: f64,
    memory_headroom: f64,
    cpu_headroom_pct: f64,
    memory_headroom_pct: f64,
}

struct ResourceOptimizer;

impl ResourceOptimizer {
    fn new() -> Self {
        Self
    }

    fn recommend(&self, usage: &ResourceUsage, strategy: OptimizationStrategy) -> OptimizationRecommendation {
        match strategy {
            OptimizationStrategy::Performance => {
                if usage.cpu_cores / usage.request_cpu > 0.9 {
                    OptimizationRecommendation {
                        action: OptimizationAction::Increase,
                        cpu_cores: usage.request_cpu * 1.5,
                        memory_gb: usage.request_memory * 1.5,
                        reason: "High utilization, recommend scale-up for performance".to_string(),
                    }
                } else {
                    OptimizationRecommendation {
                        action: OptimizationAction::Maintain,
                        cpu_cores: usage.request_cpu,
                        memory_gb: usage.request_memory,
                        reason: "Utilization within acceptable range".to_string(),
                    }
                }
            }
            OptimizationStrategy::Cost => {
                if usage.cpu_cores / usage.request_cpu < 0.5 {
                    OptimizationRecommendation {
                        action: OptimizationAction::Decrease,
                        cpu_cores: usage.cpu_cores * 1.2,
                        memory_gb: usage.memory_gb * 1.2,
                        reason: "Over-provisioned, recommend rightsizing for cost savings".to_string(),
                    }
                } else {
                    OptimizationRecommendation {
                        action: OptimizationAction::Maintain,
                        cpu_cores: usage.request_cpu,
                        memory_gb: usage.request_memory,
                        reason: "Reasonable utilization".to_string(),
                    }
                }
            }
            OptimizationStrategy::Balanced => {
                OptimizationRecommendation {
                    action: OptimizationAction::Maintain,
                    cpu_cores: usage.request_cpu,
                    memory_gb: usage.request_memory,
                    reason: "Balanced configuration".to_string(),
                }
            }
        }
    }

    fn right_size(&self, current: ResourceLimits, usage: ResourceUsage) -> ResourceLimits {
        let cpu_request = (usage.cpu_cores * 1.2).min(current.cpu_limit);
        let memory_request = (usage.memory_gb * 1.2).min(current.memory_limit);

        ResourceLimits {
            cpu_request,
            cpu_limit: (cpu_request * 2.0).min(current.cpu_limit),
            memory_request,
            memory_limit: (memory_request * 2.0).min(current.memory_limit),
        }
    }

    fn bin_pack(&self, capacity: NodeCapacity, workloads: Vec<Workload>) -> Option<Vec<Workload>> {
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        let mut packed = Vec::new();

        for workload in workloads {
            if total_cpu + workload.cpu_request <= capacity.cpu_cores
                && total_memory + workload.memory_request <= capacity.memory_gb
            {
                total_cpu += workload.cpu_request;
                total_memory += workload.memory_request;
                packed.push(workload);
            } else {
                return None;
            }
        }

        Some(packed)
    }

    fn calculate_cost_savings(&self, current: ResourceLimits, optimized: ResourceLimits) -> f64 {
        let current_total = current.cpu_limit + current.memory_limit;
        let optimized_total = optimized.cpu_limit + optimized.memory_limit;

        if current_total == 0.0 {
            0.0
        } else {
            (current_total - optimized_total) / current_total
        }
    }

    fn calculate_headroom(&self, usage: &ResourceUsage) -> ResourceHeadroom {
        let cpu_headroom = usage.limit_cpu - usage.cpu_cores;
        let memory_headroom = usage.limit_memory - usage.memory_gb;

        ResourceHeadroom {
            cpu_headroom,
            memory_headroom,
            cpu_headroom_pct: (cpu_headroom / usage.limit_cpu) * 100.0,
            memory_headroom_pct: (memory_headroom / usage.limit_memory) * 100.0,
        }
    }
}

criterion_group!(
    benches,
    bench_optimization_recommendation,
    bench_right_sizing,
    bench_bin_packing,
    bench_bin_packing_scalability,
    bench_cost_calculation,
    bench_headroom_calculation,
    bench_batch_optimization
);

criterion_main!(benches);
