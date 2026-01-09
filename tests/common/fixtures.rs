//! Test fixtures and data

use serde::{Deserialize, Serialize};

/// Fixture: Sample cluster configuration
pub const SMALL_CLUSTER_CONFIG: &str = r#"
apiVersion: kind.x-k8s.io/v1alpha4
kind: Cluster
nodes:
- role: control-plane
- role: worker
- role: worker
"#;

/// Fixture: Sample deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDeployment {
    pub name: String,
    pub image: String,
    pub replicas: u32,
    pub port: u32,
}

impl TestDeployment {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            image: "nginx:latest".to_string(),
            replicas: 3,
            port: 8080,
        }
    }

    pub fn with_image(mut self, image: &str) -> Self {
        self.image = image.to_string();
        self
    }

    pub fn with_replicas(mut self, replicas: u32) -> Self {
        self.replicas = replicas;
        self
    }

    pub fn to_yaml(&self) -> String {
        format!(
            r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
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
        - containerPort: {}
"#,
            self.name, self.replicas, self.name, self.name, self.name, self.image, self.port
        )
    }
}

/// Fixture: Sample scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicyFixture {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: i32,
    pub target_memory_utilization: i32,
    pub stabilization_window_seconds: u32,
}

impl Default for ScalingPolicyFixture {
    fn default() -> Self {
        Self {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
            stabilization_window_seconds: 300,
        }
    }
}

/// Fixture: Sample metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFixture {
    pub name: String,
    pub value: f64,
    pub timestamp: i64,
}

impl MetricFixture {
    pub fn cpu_utilization(value: f64) -> Self {
        Self {
            name: "cpu_utilization".to_string(),
            value,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn memory_utilization(value: f64) -> Self {
        Self {
            name: "memory_utilization".to_string(),
            value,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn request_rate(value: f64) -> Self {
        Self {
            name: "request_rate".to_string(),
            value,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Fixture: Sample cluster state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStateFixture {
    pub total_nodes: u32,
    pub ready_nodes: u32,
    pub total_pods: u32,
    pub running_pods: u32,
    pub cpu_capacity: f64,
    pub cpu_used: f64,
    pub memory_capacity: f64,
    pub memory_used: f64,
}

impl ClusterStateFixture {
    pub fn healthy() -> Self {
        Self {
            total_nodes: 3,
            ready_nodes: 3,
            total_pods: 30,
            running_pods: 30,
            cpu_capacity: 12.0,
            cpu_used: 6.0,
            memory_capacity: 48.0,
            memory_used: 24.0,
        }
    }

    pub fn degraded() -> Self {
        Self {
            total_nodes: 3,
            ready_nodes: 2,
            total_pods: 30,
            running_pods: 25,
            cpu_capacity: 12.0,
            cpu_used: 8.0,
            memory_capacity: 48.0,
            memory_used: 32.0,
        }
    }

    pub fn overloaded() -> Self {
        Self {
            total_nodes: 3,
            ready_nodes: 3,
            total_pods: 50,
            running_pods: 50,
            cpu_capacity: 12.0,
            cpu_used: 11.5,
            memory_capacity: 48.0,
            memory_used: 46.0,
        }
    }
}

/// Fixture: Sample chaos scenarios
pub mod chaos {
    pub const POD_KILL_SCENARIO: &str = r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-kill-test
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - test-cluster-orchestrator
    labelSelectors:
      app: test-app
  scheduler:
    cron: "@every 10s"
"#;

    pub const NETWORK_DELAY_SCENARIO: &str = r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: network-delay
spec:
  action: delay
  mode: all
  delay:
    latency: "500ms"
    jitter: "100ms"
  selector:
    namespaces:
      - test-cluster-orchestrator
    labelSelectors:
      app: test-app
"#;

    pub const POD_FAILURE_SCENARIO: &str = r#"
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-failure
spec:
  action: container-kill
  mode: fixed-percent
  value: "50"
  containerNames:
    - app
  selector:
    namespaces:
      - test-cluster-orchestrator
    labelSelectors:
      app: test-app
"#;
}

/// Create mock Kubernetes client responses
pub mod mock_responses {
    use serde_json::json;

    pub fn pod_list(count: usize) -> serde_json::Value {
        let pods: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                json!({
                    "metadata": {
                        "name": format!("pod-{}", i),
                        "namespace": "default",
                        "labels": {
                            "app": "test-app"
                        }
                    },
                    "status": {
                        "phase": "Running",
                        "podIP": format!("10.244.{}.{}", i / 256, i % 256)
                    }
                })
            })
            .collect();

        json!({
            "apiVersion": "v1",
            "kind": "PodList",
            "items": pods
        })
    }

    pub fn deployment(replicas: u32) -> serde_json::Value {
        json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "test-app",
                "namespace": "default"
            },
            "spec": {
                "replicas": replicas,
                "selector": {
                    "matchLabels": {
                        "app": "test-app"
                    }
                }
            },
            "status": {
                "readyReplicas": replicas,
                "replicas": replicas,
                "updatedReplicas": replicas,
                "availableReplicas": replicas
            }
        })
    }

    pub fn node_list(count: usize) -> serde_json::Value {
        let nodes: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                json!({
                    "metadata": {
                        "name": format!("node-{}", i),
                    },
                    "status": {
                        "conditions": [
                            {
                                "type": "Ready",
                                "status": "True"
                            }
                        ]
                    }
                })
            })
            .collect();

        json!({
            "apiVersion": "v1",
            "kind": "NodeList",
            "items": nodes
        })
    }
}
