"""Cluster — manages a set of nodes and tracks membership."""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass, field
from typing import Optional

from .node import ClusterNode, NodeHealth, NodeRole

logger = logging.getLogger(__name__)


@dataclass
class Cluster:
    """A cluster of nodes with membership management."""

    name: str = "default"
    nodes: dict[str, ClusterNode] = field(default_factory=dict)
    heartbeat_timeout: float = 30.0
    _leader_id: Optional[str] = field(default=None, repr=False)

    # --- membership ---------------------------------------------------------

    def add_node(self, node: ClusterNode) -> None:
        if node.id in self.nodes:
            raise ValueError(f"Node {node.id} already exists in cluster")
        self.nodes[node.id] = node
        logger.info("Node %s (%s) joined cluster '%s'", node.name, node.id, self.name)

    def remove_node(self, node_id: str) -> ClusterNode:
        if node_id not in self.nodes:
            raise KeyError(f"Node {node_id} not found")
        node = self.nodes.pop(node_id)
        if self._leader_id == node_id:
            self._leader_id = None
        logger.info("Node %s left cluster '%s'", node.name, self.name)
        return node

    def get_node(self, node_id: str) -> ClusterNode:
        if node_id not in self.nodes:
            raise KeyError(f"Node {node_id} not found")
        return self.nodes[node_id]

    # --- queries ------------------------------------------------------------

    @property
    def size(self) -> int:
        return len(self.nodes)

    @property
    def healthy_nodes(self) -> list[ClusterNode]:
        return [n for n in self.nodes.values() if n.health == NodeHealth.HEALTHY]

    @property
    def active_nodes(self) -> list[ClusterNode]:
        """Nodes that are alive (recent heartbeat)."""
        now = time.time()
        return [n for n in self.nodes.values() if (now - n.last_heartbeat) < self.heartbeat_timeout]

    @property
    def workers(self) -> list[ClusterNode]:
        return [n for n in self.nodes.values() if n.role == NodeRole.WORKER]

    @property
    def managers(self) -> list[ClusterNode]:
        return [n for n in self.nodes.values() if n.role in (NodeRole.MANAGER, NodeRole.LEADER)]

    @property
    def leader(self) -> Optional[ClusterNode]:
        if self._leader_id and self._leader_id in self.nodes:
            return self.nodes[self._leader_id]
        return None

    def set_leader(self, node_id: str) -> None:
        if node_id not in self.nodes:
            raise KeyError(f"Node {node_id} not found")
        # demote previous leader
        if self._leader_id and self._leader_id in self.nodes:
            self.nodes[self._leader_id].role = NodeRole.MANAGER
        self._leader_id = node_id
        self.nodes[node_id].role = NodeRole.LEADER

    # --- cluster-level resource view ----------------------------------------

    @property
    def total_cpu(self) -> float:
        return sum(n.capacity_cpu for n in self.nodes.values())

    @property
    def total_memory(self) -> float:
        return sum(n.capacity_memory for n in self.nodes.values())

    @property
    def used_cpu(self) -> float:
        return sum(n.used_cpu for n in self.nodes.values())

    @property
    def used_memory(self) -> float:
        return sum(n.used_memory for n in self.nodes.values())

    @property
    def available_cpu(self) -> float:
        return self.total_cpu - self.used_cpu

    @property
    def available_memory(self) -> float:
        return self.total_memory - self.used_memory

    # --- pruning ------------------------------------------------------------

    def prune_dead_nodes(self) -> list[ClusterNode]:
        """Remove nodes that haven't heartbeated within timeout."""
        now = time.time()
        dead = [
            nid for nid, n in self.nodes.items()
            if (now - n.last_heartbeat) >= self.heartbeat_timeout
        ]
        removed = []
        for nid in dead:
            removed.append(self.remove_node(nid))
        if removed:
            logger.info("Pruned %d dead node(s)", len(removed))
        return removed
