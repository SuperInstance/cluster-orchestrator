"""Tests for cluster_orchestrator.scheduler"""

import pytest

from cluster_orchestrator.cluster import Cluster
from cluster_orchestrator.node import ClusterNode, NodeHealth
from cluster_orchestrator.scheduler import Scheduler, SchedulingStrategy, Task


def _cluster_with_nodes(*capacities: tuple[float, float]) -> tuple[Cluster, list[ClusterNode]]:
    c = Cluster()
    nodes = []
    for cpu, mem in capacities:
        n = ClusterNode(capacity_cpu=cpu, capacity_memory=mem, health=NodeHealth.HEALTHY)
        n.heartbeat()
        c.add_node(n)
        nodes.append(n)
    return c, nodes


class TestBinPack:
    def test_fills_tightest(self):
        c, nodes = _cluster_with_nodes((4, 8), (2, 4))
        s = Scheduler(c, SchedulingStrategy.BIN_PACK)
        # small node has less available → bin-pack prefers it
        t = Task(id="t1", cpu=1, memory=1)
        p = s.schedule(t)
        assert p is not None
        assert p.node_id == nodes[1].id  # smaller node

    def test_no_fit(self):
        c, _ = _cluster_with_nodes((1, 1))
        s = Scheduler(c, SchedulingStrategy.BIN_PACK)
        p = s.schedule(Task(id="t1", cpu=5, memory=5))
        assert p is None

    def test_fill_up(self):
        c, nodes = _cluster_with_nodes((2, 4), (2, 4))
        s = Scheduler(c, SchedulingStrategy.BIN_PACK)
        s.schedule(Task(id="t1", cpu=1, memory=2))
        # node 0 now has less available
        p2 = s.schedule(Task(id="t2", cpu=1, memory=2))
        assert p2 is not None
        # should still prefer node 0 (less available)
        assert p2.node_id == nodes[0].id


class TestSpread:
    def test_spreads_to_most_available(self):
        c, nodes = _cluster_with_nodes((2, 4), (8, 16))
        s = Scheduler(c, SchedulingStrategy.SPREAD)
        p = s.schedule(Task(id="t1", cpu=1, memory=2))
        assert p is not None
        assert p.node_id == nodes[1].id  # bigger node


class TestRoundRobin:
    def test_rotates(self):
        c, nodes = _cluster_with_nodes((10, 10), (10, 10), (10, 10))
        s = Scheduler(c, SchedulingStrategy.ROUND_ROBIN)
        p1 = s.schedule(Task(id="t1", cpu=1, memory=1))
        p2 = s.schedule(Task(id="t2", cpu=1, memory=1))
        p3 = s.schedule(Task(id="t3", cpu=1, memory=1))
        assert p1 is not None and p2 is not None and p3 is not None
        ids = {p1.node_id, p2.node_id, p3.node_id}
        assert len(ids) == 3


class TestPreferredNode:
    def test_honours_preference(self):
        c, nodes = _cluster_with_nodes((10, 10), (10, 10))
        s = Scheduler(c, SchedulingStrategy.BIN_PACK)
        p = s.schedule(Task(id="t1", cpu=1, memory=1, preferred_node=nodes[1].id))
        assert p is not None
        assert p.node_id == nodes[1].id

    def test_falls_back_if_preferred_full(self):
        c, nodes = _cluster_with_nodes((1, 1), (10, 10))
        nodes[0].allocate(1, 1)  # fill node 0
        s = Scheduler(c)
        p = s.schedule(Task(id="t1", cpu=1, memory=1, preferred_node=nodes[0].id))
        assert p is not None
        assert p.node_id == nodes[1].id


class TestDeschedule:
    def test_deschedule_releases(self):
        c, nodes = _cluster_with_nodes((10, 10))
        s = Scheduler(c)
        s.schedule(Task(id="t1", cpu=3, memory=3))
        assert nodes[0].used_cpu == 3
        s.deschedule("t1")
        assert nodes[0].used_cpu == 0

    def test_deschedule_nonexistent(self):
        c, _ = _cluster_with_nodes((10, 10))
        s = Scheduler(c)
        assert s.deschedule("nope") is None


class TestScheduleMany:
    def test_all_fit(self):
        c, _ = _cluster_with_nodes((10, 10))
        s = Scheduler(c)
        tasks = [Task(id=f"t{i}", cpu=1, memory=1) for i in range(5)]
        results = s.schedule_many(tasks)
        assert len(results) == 5

    def test_some_dont_fit(self):
        c, _ = _cluster_with_nodes((2, 2))
        s = Scheduler(c)
        tasks = [Task(id=f"t{i}", cpu=1, memory=1) for i in range(5)]
        results = s.schedule_many(tasks)
        assert len(results) == 2
