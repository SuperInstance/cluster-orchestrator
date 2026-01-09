//! Unit tests for resource optimization logic

use cluster_orchestrator::optimizer::{ResourceOptimizer, OptimizationStrategy, OptimizationRecommendation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_recommendation_scale_up() {
        let optimizer = ResourceOptimizer::new();
        let usage = ResourceUsage {
            cpu_cores: 2.0,
            memory_gb: 8.0,
            request_cpu: 1.0,
            request_memory: 4.0,
            limit_cpu: 2.0,
            limit_memory: 8.0,
        };

        let recommendation = optimizer.recommend(&usage, OptimizationStrategy::Performance);

        assert_eq!(recommendation.action, OptimizationAction::Increase);
        assert!(recommendation.cpu_cores > usage.cpu_cores);
    }

    #[test]
    fn test_optimization_recommendation_scale_down() {
        let optimizer = ResourceOptimizer::new();
        let usage = ResourceUsage {
            cpu_cores: 2.0,
            memory_gb: 8.0,
            request_cpu: 4.0,
            request_memory: 16.0,
            limit_cpu: 8.0,
            limit_memory: 32.0,
        };

        let recommendation = optimizer.recommend(&usage, OptimizationStrategy::Cost);

        assert_eq!(recommendation.action, OptimizationAction::Decrease);
        assert!(recommendation.cpu_cores < usage.request_cpu);
    }

    #[test]
    fn test_optimization_right_sizing() {
        let optimizer = ResourceOptimizer::new();

        let current = ResourceLimits {
            cpu_request: 4.0,
            cpu_limit: 8.0,
            memory_request: 16.0,
            memory_limit: 32.0,
        };

        let usage = ResourceUsage {
            cpu_cores: 2.5,
            memory_gb: 12.0,
            request_cpu: 4.0,
            request_memory: 16.0,
            limit_cpu: 8.0,
            limit_memory: 32.0,
        };

        let right_sized = optimizer.right_size(current, usage);

        // Right-sized requests should be slightly above usage
        assert!(right_sized.cpu_request >= usage.cpu_cores);
        assert!(right_sized.cpu_request < current.cpu_request);

        // Limits should be reasonable multiple of requests
        assert!(right_sized.cpu_limit >= right_sized.cpu_request * 1.5);
    }

    #[test]
    fn test_optimization_bin_packing() {
        let optimizer = ResourceOptimizer::new();

        let node_capacity = NodeCapacity {
            cpu_cores: 16.0,
            memory_gb: 64.0,
        };

        let workloads = vec![
            Workload {
                name: "app1".to_string(),
                cpu_request: 2.0,
                memory_request: 8.0,
            },
            Workload {
                name: "app2".to_string(),
                cpu_request: 4.0,
                memory_request: 16.0,
            },
            Workload {
                name: "app3".to_string(),
                cpu_request: 3.0,
                memory_request: 12.0,
            },
        ];

        let packing = optimizer.bin_pack(node_capacity, workloads);

        // Should fit all workloads
        assert!(packing.is_some());
        let packed = packing.unwrap();
        assert_eq!(packed.len(), 3);
    }

    #[test]
    fn test_optimization_bin_packing_insufficient_capacity() {
        let optimizer = ResourceOptimizer::new();

        let node_capacity = NodeCapacity {
            cpu_cores: 4.0,
            memory_gb: 16.0,
        };

        let workloads = vec![
            Workload {
                name: "app1".to_string(),
                cpu_request: 3.0,
                memory_request: 12.0,
            },
            Workload {
                name: "app2".to_string(),
                cpu_request: 3.0,
                memory_request: 12.0,
            },
        ];

        let packing = optimizer.bin_pack(node_capacity, workloads);

        // Cannot fit both workloads
        assert!(packing.is_none());
    }

    #[test]
    fn test_optimization_cost_calculation() {
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

        let savings = optimizer.calculate_cost_savings(current, optimized);

        assert!(savings > 0.0);
        assert!(savings < 1.0); // Should be percentage
    }

    #[test]
    fn test_optimization_strategy_balanced() {
        let optimizer = ResourceOptimizer::new();

        let usage = ResourceUsage {
            cpu_cores: 2.0,
            memory_gb: 8.0,
            request_cpu: 3.0,
            request_memory: 12.0,
            limit_cpu: 6.0,
            limit_memory: 24.0,
        };

        let recommendation = optimizer.recommend(&usage, OptimizationStrategy::Balanced);

        // Balanced strategy should be moderate
        assert_eq!(recommendation.action, OptimizationAction::Maintain);
    }

    #[test]
    fn test_optimization_headroom_calculation() {
        let optimizer = ResourceOptimizer::new();

        let usage = ResourceUsage {
            cpu_cores: 2.0,
            memory_gb: 8.0,
            request_cpu: 3.0,
            request_memory: 12.0,
            limit_cpu: 6.0,
            limit_memory: 24.0,
        };

        let headroom = optimizer.calculate_headroom(&usage);

        assert!(headroom.cpu_headroom > 0.0);
        assert!(headroom.memory_headroom > 0.0);

        // Headroom percentage should be reasonable
        assert!(headroom.cpu_headroom_pct > 10.0);
        assert!(headroom.cpu_headroom_pct < 100.0);
    }
}

// Stub types
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    Performance,
    Cost,
    Balanced,
}

#[derive(Debug, PartialEq)]
pub enum OptimizationAction {
    Increase,
    Decrease,
    Maintain,
}

struct ResourceUsage {
    cpu_cores: f64,
    memory_gb: f64,
    request_cpu: f64,
    request_memory: f64,
    limit_cpu: f64,
    limit_memory: f64,
}

struct ResourceLimits {
    cpu_request: f64,
    cpu_limit: f64,
    memory_request: f64,
    memory_limit: f64,
}

struct NodeCapacity {
    cpu_cores: f64,
    memory_gb: f64,
}

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
        // Add 20% headroom to usage
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
                return None; // Cannot fit
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
