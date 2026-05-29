# cluster-orchestrator — Cluster Orchestration

**Distributed cluster management with leader election, scheduling, and rebalancing.**

## What This Gives You

- **Cluster nodes** — register nodes with roles (LEADER, WORKER, STANDBY) and health tracking
- **Leader election** — automatic leader election with failure recovery
- **Task scheduling** — distribute tasks across nodes with configurable strategies (round-robin, least-loaded, capability-match)
- **Rebalancing** — automatically redistribute work when nodes join, leave, or fail
- **Mixed language** — Rust benches + Python orchestration logic

## Quick Start

```bash
pip install cluster-orchestrator
```

```python
from cluster_orchestrator import Cluster, ClusterNode, NodeRole, Scheduler, LeaderElection

# Create a cluster
cluster = Cluster()
cluster.add_node(ClusterNode(id="node-1", role=NodeRole.LEADER, capabilities=["python", "rust"]))
cluster.add_node(ClusterNode(id="node-2", role=NodeRole.WORKER, capabilities=["python"]))
cluster.add_node(ClusterNode(id="node-3", role=NodeRole.WORKER, capabilities=["docs"]))

# Elect a leader
election = LeaderElection(cluster)
leader = election.run()
print(f"Leader: {leader.id}")

# Schedule tasks
scheduler = Scheduler(cluster)
assignment = scheduler.schedule(task="Build Rust library", strategy="capability-match")
print(f"Assigned to: {assignment.node_id}")

# Rebalance on node failure
cluster.mark_unhealthy("node-2")
from cluster_orchestrator import Rebalancer
rebalancer = Rebalancer(cluster)
rebalancer.rebalance()
```

## API Reference

### `Cluster` — `add_node()`, `remove_node()`, `mark_unhealthy()`
### `ClusterNode(id, role, capabilities)` · `NodeRole` — LEADER, WORKER, STANDBY
### `LeaderElection(cluster)` — `run() → ClusterNode`
### `Scheduler(cluster)` — `schedule(task, strategy) → Assignment`
### `SchedulingStrategy` — ROUND_ROBIN, LEAST_LOADED, CAPABILITY_MATCH
### `Rebalancer(cluster)` — `rebalance()`

## How It Fits

The cluster management layer for the [SuperInstance fleet](https://github.com/SuperInstance). Operates above [agent-grid](https://github.com/SuperInstance/agent-grid) (topology) and below [captain](https://github.com/SuperInstance/captain) (fleet command).

- **[captain](https://github.com/SuperInstance/captain)** — Fleet commanding (delegates to cluster orchestrator)
- **[agent-grid](https://github.com/SuperInstance/agent-grid)** — Grid topology (node-level)
- **[fleet-health-monitor](https://github.com/SuperInstance/fleet-health-monitor)** — Health monitoring

## Testing

```bash
pytest tests/
```

## Installation

```bash
pip install cluster-orchestrator
```

Python 3.10+. MIT license.
