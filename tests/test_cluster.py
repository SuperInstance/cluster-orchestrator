"""Tests for cluster_orchestrator.cluster"""

import time

from cluster_orchestrator.cluster import Cluster
from cluster_orchestrator.node import ClusterNode, NodeHealth, NodeRole


def _make_node(**kwargs) -> ClusterNode:
    defaults = {"capacity_cpu": 4.0, "capacity_memory": 8.0, "health": NodeHealth.HEALTHY}
    defaults.update(kwargs)
    return ClusterNode(**defaults)


class TestCluster:
    def test_add_node(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        assert c.size == 1

    def test_add_duplicate_raises(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        import pytest
        with pytest.raises(ValueError):
            c.add_node(n)

    def test_remove_node(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        removed = c.remove_node(n.id)
        assert removed.id == n.id
        assert c.size == 0

    def test_remove_missing_raises(self):
        import pytest
        c = Cluster()
        with pytest.raises(KeyError):
            c.remove_node("nonexistent")

    def test_get_node(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        assert c.get_node(n.id) is n

    def test_healthy_nodes(self):
        c = Cluster()
        h = _make_node(health=NodeHealth.HEALTHY)
        u = _make_node(health=NodeHealth.UNHEALTHY)
        c.add_node(h)
        c.add_node(u)
        assert len(c.healthy_nodes) == 1

    def test_workers_and_managers(self):
        c = Cluster()
        w = _make_node(role=NodeRole.WORKER)
        m = _make_node(role=NodeRole.MANAGER)
        c.add_node(w)
        c.add_node(m)
        assert len(c.workers) == 1
        assert len(c.managers) == 1

    def test_leader(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        c.set_leader(n.id)
        assert c.leader is n
        assert n.role == NodeRole.LEADER

    def test_set_leader_demotes_previous(self):
        c = Cluster()
        n1 = _make_node()
        n2 = _make_node()
        c.add_node(n1)
        c.add_node(n2)
        c.set_leader(n1.id)
        assert n1.role == NodeRole.LEADER
        c.set_leader(n2.id)
        assert n1.role == NodeRole.MANAGER
        assert n2.role == NodeRole.LEADER

    def test_remove_leader_clears(self):
        c = Cluster()
        n = _make_node()
        c.add_node(n)
        c.set_leader(n.id)
        c.remove_node(n.id)
        assert c.leader is None

    def test_cluster_resources(self):
        c = Cluster()
        c.add_node(_make_node(capacity_cpu=4.0, capacity_memory=8.0, used_cpu=1.0, used_memory=2.0))
        c.add_node(_make_node(capacity_cpu=2.0, capacity_memory=4.0, used_cpu=0.0, used_memory=0.0))
        assert c.total_cpu == 6.0
        assert c.total_memory == 12.0
        assert c.used_cpu == 1.0
        assert c.used_memory == 2.0
        assert c.available_cpu == 5.0
        assert c.available_memory == 10.0

    def test_prune_dead_nodes(self):
        c = Cluster(heartbeat_timeout=5.0)
        alive = _make_node(last_heartbeat=time.time())
        dead = _make_node(last_heartbeat=time.time() - 100)
        c.add_node(alive)
        c.add_node(dead)
        pruned = c.prune_dead_nodes()
        assert len(pruned) == 1
        assert pruned[0].id == dead.id
        assert c.size == 1

    def test_active_nodes(self):
        c = Cluster(heartbeat_timeout=5.0)
        alive = _make_node(last_heartbeat=time.time())
        dead = _make_node(last_heartbeat=time.time() - 100)
        c.add_node(alive)
        c.add_node(dead)
        assert len(c.active_nodes) == 1
