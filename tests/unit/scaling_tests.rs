//! Unit tests for scaling logic

use cluster_orchestrator::scaling::{Scaler, ScalingPolicy, ScalingDirection};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaling_decision_scale_up() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        let decision = Scaler::evaluate_scaling(
            &policy,
            3,      // current replicas
            85.0,   // cpu utilization
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::Up);
        assert_eq!(decision.target_replicas, 4);
        assert!(decision.reason.contains("above threshold"));
    }

    #[test]
    fn test_scaling_decision_scale_down() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        let decision = Scaler::evaluate_scaling(
            &policy,
            5,      // current replicas
            25.0,   // cpu utilization
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::Down);
        assert_eq!(decision.target_replicas, 4);
        assert!(decision.reason.contains("below threshold"));
    }

    #[test]
    fn test_scaling_decision_no_change() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        let decision = Scaler::evaluate_scaling(
            &policy,
            5,      // current replicas
            68.0,   // cpu utilization (within range)
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::None);
        assert_eq!(decision.target_replicas, 5);
    }

    #[test]
    fn test_scaling_decision_max_limit() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // Already at max, high CPU
        let decision = Scaler::evaluate_scaling(
            &policy,
            10,     // current replicas (at max)
            95.0,   // cpu utilization
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::None);
        assert_eq!(decision.target_replicas, 10);
        assert!(decision.reason.contains("at maximum"));
    }

    #[test]
    fn test_scaling_decision_min_limit() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // Already at min, low CPU
        let decision = Scaler::evaluate_scaling(
            &policy,
            2,      // current replicas (at min)
            20.0,   // cpu utilization
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::None);
        assert_eq!(decision.target_replicas, 2);
        assert!(decision.reason.contains("at minimum"));
    }

    #[test]
    fn test_scaling_threshold_calculation() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // Upper threshold: target + 10%
        let upper = policy.upper_threshold();
        assert_eq!(upper, 77.0);

        // Lower threshold: target - 10%
        let lower = policy.lower_threshold();
        assert_eq!(lower, 63.0);
    }

    #[test]
    fn test_scaling_replica_calculation() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // Scale up based on CPU
        let replicas = Scaler::calculate_replicas(
            &policy,
            5,      // current replicas
            3,      // current CPU cores
            87.5,   // target utilization
        ).unwrap();

        // 3 / 0.875 = 3.43, round up to 4
        assert_eq!(replicas, 4);
    }

    #[test]
    fn test_scaling_stabilization_window() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        let scaler = Scaler::new(policy);
        scaler.record_scaling_decision(ScalingDirection::Up);

        // Should be in stabilization window
        assert!(scaler.is_in_stabilization_window());
    }

    #[test]
    fn test_scaling_multiple_metrics() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // Both metrics indicate scale up
        let decision = Scaler::evaluate_scaling_multi(
            &policy,
            3,      // current replicas
            85.0,   // cpu utilization
            75.0,   // memory utilization
        ).unwrap();

        assert_eq!(decision.direction, ScalingDirection::Up);
    }

    #[test]
    fn test_scaling_conflicting_metrics() {
        let policy = ScalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            stabilization_window: Duration::from_secs(300),
        };

        // CPU high, memory low - should be conservative
        let decision = Scaler::evaluate_scaling_multi(
            &policy,
            5,      // current replicas
            85.0,   // cpu utilization (scale up)
            30.0,   // memory utilization (scale down)
        ).unwrap();

        // Should not scale due to conflicting signals
        assert_eq!(decision.direction, ScalingDirection::None);
    }
}

// Stub implementation
pub struct ScalingPolicy {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: i32,
    pub stabilization_window: Duration,
}

impl ScalingPolicy {
    fn upper_threshold(&self) -> f64 {
        self.target_cpu_utilization as f64 * 1.1
    }

    fn lower_threshold(&self) -> f64 {
        self.target_cpu_utilization as f64 * 0.9
    }
}

pub enum ScalingDirection {
    Up,
    Down,
    None,
}

pub struct ScalingDecision {
    pub direction: ScalingDirection,
    pub target_replicas: u32,
    pub reason: String,
}

pub struct Scaler {
    policy: ScalingPolicy,
    last_scaling_time: Option<std::time::Instant>,
}

impl Scaler {
    fn new(policy: ScalingPolicy) -> Self {
        Self {
            policy,
            last_scaling_time: None,
        }
    }

    fn evaluate_scaling(
        policy: &ScalingPolicy,
        current_replicas: u32,
        cpu_utilization: f64,
    ) -> Result<ScalingDecision, String> {
        let upper = policy.upper_threshold();
        let lower = policy.lower_threshold();

        if cpu_utilization > upper {
            // Scale up
            let target = (current_replicas + 1).min(policy.max_replicas);
            if target == current_replicas {
                return Ok(ScalingDecision {
                    direction: ScalingDirection::None,
                    target_replicas: current_replicas,
                    reason: "Already at maximum replicas".to_string(),
                });
            }
            Ok(ScalingDecision {
                direction: ScalingDirection::Up,
                target_replicas: target,
                reason: format!("CPU utilization {:.1}% above threshold {:.1}%", cpu_utilization, upper),
            })
        } else if cpu_utilization < lower {
            // Scale down
            let target = (current_replicas - 1).max(policy.min_replicas);
            if target == current_replicas {
                return Ok(ScalingDecision {
                    direction: ScalingDirection::None,
                    target_replicas: current_replicas,
                    reason: "Already at minimum replicas".to_string(),
                });
            }
            Ok(ScalingDecision {
                direction: ScalingDirection::Down,
                target_replicas: target,
                reason: format!("CPU utilization {:.1}% below threshold {:.1}%", cpu_utilization, lower),
            })
        } else {
            Ok(ScalingDecision {
                direction: ScalingDirection::None,
                target_replicas: current_replicas,
                reason: "CPU utilization within acceptable range".to_string(),
            })
        }
    }

    fn evaluate_scaling_multi(
        policy: &ScalingPolicy,
        current_replicas: u32,
        cpu_utilization: f64,
        _memory_utilization: f64,
    ) -> Result<ScalingDecision, String> {
        // Simplified: just use CPU for now
        Self::evaluate_scaling(policy, current_replicas, cpu_utilization)
    }

    fn calculate_replicas(
        policy: &ScalingPolicy,
        _current_replicas: u32,
        current_cpu: f64,
        target_utilization: f64,
    ) -> Result<u32, String> {
        let required = current_cpu / (target_utilization / 100.0);
        let replicas = required.ceil() as u32;
        Ok(replicas.min(policy.max_replicas).max(policy.min_replicas))
    }

    fn record_scaling_decision(&mut self, _direction: ScalingDirection) {
        self.last_scaling_time = Some(std::time::Instant::now());
    }

    fn is_in_stabilization_window(&self) -> bool {
        match self.last_scaling_time {
            Some(time) => time.elapsed() < self.policy.stabilization_window,
            None => false,
        }
    }
}
