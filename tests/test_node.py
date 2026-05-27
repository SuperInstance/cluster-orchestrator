"""Tests for cluster_orchestrator.node"""

import time

from cluster_orchestrator.node import ClusterNode, NodeHealth, NodeRole


class TestClusterNode:
    def test_defaults(self):
        node = ClusterNode()
        assert node.role == NodeRole.WORKER
        assert node.health == NodeHealth.UNKNOWN
        assert node.capacity_cpu == 1.0
        assert node.used_cpu == 0.0
        assert node.id  # auto-generated
        assert node.name.startswith("node-")

    def test_custom_name(self):
        node = ClusterNode(name="my-node")
        assert node.name == "my-node"

    def test_available_resources(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0, used_cpu=1.0, used_memory=2.0)
        assert node.available_cpu == 3.0
        assert node.available_memory == 6.0

    def test_utilization(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0, used_cpu=2.0, used_memory=4.0)
        assert node.cpu_utilization == 0.5
        assert node.memory_utilization == 0.5

    def test_utilization_zero_capacity(self):
        node = ClusterNode(capacity_cpu=0.0, capacity_memory=0.0)
        assert node.cpu_utilization == 0.0
        assert node.memory_utilization == 0.0

    def test_can_allocate(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0)
        assert node.can_allocate(2.0, 4.0) is True
        assert node.can_allocate(5.0, 1.0) is False
        assert node.can_allocate(1.0, 9.0) is False

    def test_allocate_success(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0)
        assert node.allocate(2.0, 3.0) is True
        assert node.used_cpu == 2.0
        assert node.used_memory == 3.0

    def test_allocate_fail(self):
        node = ClusterNode(capacity_cpu=1.0, capacity_memory=1.0)
        assert node.allocate(2.0, 0.5) is False
        assert node.used_cpu == 0.0

    def test_release(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0)
        node.allocate(2.0, 4.0)
        node.release(1.0, 2.0)
        assert node.used_cpu == 1.0
        assert node.used_memory == 2.0

    def test_release_clamps_to_zero(self):
        node = ClusterNode(capacity_cpu=4.0, capacity_memory=8.0)
        node.release(10.0, 10.0)  # release more than used
        assert node.used_cpu == 0.0
        assert node.used_memory == 0.0

    def test_heartbeat(self):
        node = ClusterNode()
        before = node.last_heartbeat
        time.sleep(0.01)
        node.heartbeat()
        assert node.last_heartbeat > before
        assert node.health == NodeHealth.HEALTHY

    def test_is_alive(self):
        node = ClusterNode(last_heartbeat=time.time())
        assert node.is_alive(timeout=30) is True

    def test_is_dead(self):
        node = ClusterNode(last_heartbeat=time.time() - 100)
        assert node.is_alive(timeout=30) is False

    def test_labels_and_metadata(self):
        node = ClusterNode(labels={"zone": "us-east"}, metadata={"version": "1.0"})
        assert node.labels["zone"] == "us-east"
        assert node.metadata["version"] == "1.0"

    def test_roles(self):
        for role in NodeRole:
            node = ClusterNode(role=role)
            assert node.role == role
