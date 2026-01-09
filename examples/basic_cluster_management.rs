//! Example: Basic cluster management with Cluster Orchestrator

use cluster_orchestrator::{
    ClusterManager, Scaler, ScalingPolicy, Healer,
    config::ClusterConfig,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Cluster Orchestrator Example");
    println!("============================\n");

    // 1. Initialize cluster manager
    println!("1. Initializing cluster manager...");
    let config = ClusterConfig::from_file("examples/cluster-config.yaml")?;
    let mut cluster = ClusterManager::new(config);

    // 2. Connect to cluster
    println!("2. Connecting to cluster...");
    cluster.connect().await?;
    println!("   Connected to cluster: {}", cluster.name());

    // 3. Check cluster health
    println!("3. Checking cluster health...");
    let health = cluster.check_health().await?;
    println!("   Cluster health: {:?}", health);
    println!("   Nodes: {}/{}", health.ready_nodes, health.total_nodes);
    println!("   Pods: {}/{}", health.ready_pods, health.total_pods);

    // 4. Deploy application
    println!("4. Deploying application...");
    cluster.deploy_application(
        "examples/nginx-deployment.yaml",
        "default",
    ).await?;
    println!("   Application deployed successfully");

    // 5. Setup auto-scaling
    println!("5. Setting up auto-scaling...");
    let scaling_policy = ScalingPolicy {
        min_replicas: 2,
        max_replicas: 10,
        target_cpu_utilization: 70,
        stabilization_window: Duration::from_secs(300),
    };

    let scaler = Scaler::new(cluster.clone(), scaling_policy);
    scaler.enable("my-app", "default").await?;
    println!("   Auto-scaling enabled for my-app");

    // 6. Monitor scaling
    println!("6. Monitoring scaling decisions (30s)...");
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;
        if let Some(decision) = scaler.last_decision("my-app").await? {
            println!("   [{}] Replicas: {}, Direction: {:?}",
                i, decision.current_replicas, decision.direction);
        }
    }

    // 7. Enable self-healing
    println!("7. Enabling self-healing...");
    let healer = Healer::new(cluster.clone());
    healer.enable("default").await?;
    println!("   Self-healing enabled for default namespace");

    // 8. Manual scaling example
    println!("8. Manual scaling example...");
    cluster.scale_deployment("my-app", "default", 5).await?;
    println!("   Scaled my-app to 5 replicas");

    // 9. Wait and verify
    println!("9. Waiting for scaling to complete...");
    sleep(Duration::from_secs(10)).await;
    let deployment = cluster.get_deployment("my-app", "default").await?;
    println!("   Current replicas: {}", deployment.spec.replicas.unwrap_or(0));

    // 10. Resource optimization
    println!("10. Running resource optimization...");
    let recommendations = cluster.optimize_resources("default").await?;
    println!("    Found {} optimization recommendations", recommendations.len());

    for rec in &recommendations {
        println!("    - {}: {} (potential savings: {}%)",
            rec.resource_name, rec.action, rec.savings_percentage);
    }

    println!("\nExample completed successfully!");
    Ok(())
}
