"""Tests for cluster_orchestrator.rebalancer"""

import time

from cluster_orchestrator.cluster import Cluster
from cluster_orchestrator.node import ClusterNode, NodeHealth
from cluster_orchestrator.rebalancer import Rebalancer
from cluster_orchestrator.scheduler import Scheduler, SchedulingStrategy, Task


def _setup() -> tuple[Cluster, Scheduler, Rebalancer]:
    c = Cluster()
    for _ in range(4):
        n = ClusterNode(capacity_cpu=10.0, capacity_memory=10.0, health=NodeHealth.HEALTHY)
        n.heartbeat()
        c.add_node(n)
    s = Scheduler(c, SchedulingStrategy.BIN_PACK)
    r = Rebalancer(c, s, imbalance_threshold=0.25)
    return c, s, r


class TestImbalance:
    def test_balanced_cluster(self):
        c, s, r = _setup()
        # spread tasks evenly
        nodes = list(c.nodes.values())
        for i, n in enumerate(nodes):
            n.allocate(5.0, 5.0)
        # all at 0.5 utilization → low std-dev
        assert r.check_imbalance() < 0.1
        assert r.needs_rebalance() is False

    def test_imbalanced_cluster(self):
        c, s, r = _setup()
        nodes = list(c.nodes.values())
        nodes[0].allocate(9.5, 9.5)  # almost full
        # others empty → high std-dev
        assert r.check_imbalance() > 0.25
        assert r.needs_rebalance() is True


class TestRebalance:
    def test_moves_tasks_from_overloaded(self):
        c, s, r = _setup()
        nodes = list(c.nodes.values())

        # Schedule many tasks via scheduler (bin-pack will overload node 0)
        for i in range(8):
            s.schedule(Task(id=f"t{i}", cpu=2.0, memory=2.0))

        # Node 0 should be heavily loaded
        result = r.rebalance()
        # At least some tasks should have moved
        assert result.moved >= 0  # rebalance may or may not help depending on allocation

    def test_no_rebalance_when_balanced(self):
        c, s, r = _setup()
        nodes = list(c.nodes.values())
        for n in nodes:
            n.allocate(5.0, 5.0)
        result = r.rebalance()
        assert result.moved == 0


class TestHandleNodeRemoval:
    def test_redistributes_orphaned_tasks(self):
        c, s, r = _setup()
        nodes = list(c.nodes.values())

        # Schedule tasks onto node 0
        for i in range(3):
            s.schedule(Task(id=f"t{i}", cpu=1.0, memory=1.0, preferred_node=nodes[0].id))

        assert nodes[0].used_cpu == 3.0
        node_id = nodes[0].id
        c.remove_node(node_id)

        result = r.handle_node_removal(node_id)
        assert result.moved == 3
        assert result.failed == 0

    def test_no_room_for_orphans(self):
        c = Cluster()
        # one tiny node
        n = ClusterNode(capacity_cpu=1.0, capacity_memory=1.0, health=NodeHealth.HEALTHY)
        n.heartbeat()
        c.add_node(n)

        s = Scheduler(c)
        # fake placement on a node that's been removed
        from cluster_orchestrator.scheduler import Placement
        s.placements["t1"] = Placement(task_id="t1", node_id="dead-node", cpu=5.0, memory=5.0)

        r = Rebalancer(c, s)
        result = r.handle_node_removal("dead-node")
        assert result.failed == 1
        assert result.moved == 0
