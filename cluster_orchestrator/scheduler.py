"""Scheduler — assign tasks to cluster nodes using various strategies."""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

from .cluster import Cluster
from .node import ClusterNode

logger = logging.getLogger(__name__)


class SchedulingStrategy(Enum):
    BIN_PACK = "bin_pack"       # fill nodes as tightly as possible
    SPREAD = "spread"           # spread evenly across nodes
    ROUND_ROBIN = "round_robin"


@dataclass
class Task:
    """A unit of work to schedule."""
    id: str
    cpu: float
    memory: float
    preferred_node: Optional[str] = None
    labels: dict[str, str] = field(default_factory=dict)


@dataclass
class Placement:
    """Result of scheduling a task."""
    task_id: str
    node_id: str
    cpu: float
    memory: float


class Scheduler:
    """Schedules tasks onto cluster nodes."""

    def __init__(self, cluster: Cluster, strategy: SchedulingStrategy = SchedulingStrategy.BIN_PACK) -> None:
        self.cluster = cluster
        self.strategy = strategy
        self._rr_index: int = 0
        self.placements: dict[str, Placement] = {}  # task_id -> Placement

    # --- public API ---------------------------------------------------------

    def schedule(self, task: Task) -> Optional[Placement]:
        """Schedule a single task. Returns Placement or None if unschedulable."""
        node = self._select_node(task)
        if node is None:
            logger.warning("Cannot schedule task %s — no suitable node", task.id)
            return None
        return self._place(task, node)

    def schedule_many(self, tasks: list[Task]) -> list[Placement]:
        results: list[Placement] = []
        for t in tasks:
            p = self.schedule(t)
            if p:
                results.append(p)
        return results

    def deschedule(self, task_id: str) -> Optional[Placement]:
        """Remove a task, releasing its resources."""
        p = self.placements.pop(task_id, None)
        if p is None:
            return None
        try:
            node = self.cluster.get_node(p.node_id)
            node.release(p.cpu, p.memory)
        except KeyError:
            pass  # node already gone
        return p

    # --- strategy dispatch --------------------------------------------------

    def _select_node(self, task: Task) -> Optional[ClusterNode]:
        # honour preferred_node first
        if task.preferred_node:
            try:
                pref = self.cluster.get_node(task.preferred_node)
                if pref.can_allocate(task.cpu, task.memory):
                    return pref
            except KeyError:
                pass

        candidates = [
            n for n in self.cluster.healthy_nodes
            if n.can_allocate(task.cpu, task.memory)
        ]
        if not candidates:
            return None

        if self.strategy == SchedulingStrategy.BIN_PACK:
            return self._bin_pack(candidates)
        elif self.strategy == SchedulingStrategy.SPREAD:
            return self._spread(candidates)
        elif self.strategy == SchedulingStrategy.ROUND_ROBIN:
            return self._round_robin(candidates)
        return candidates[0]

    @staticmethod
    def _bin_pack(candidates: list[ClusterNode]) -> ClusterNode:
        """Pick the node with the *least* available resources that can still fit."""
        return min(candidates, key=lambda n: n.available_cpu)

    @staticmethod
    def _spread(candidates: list[ClusterNode]) -> ClusterNode:
        """Pick the node with the *most* available resources."""
        return max(candidates, key=lambda n: n.available_cpu)

    def _round_robin(self, candidates: list[ClusterNode]) -> ClusterNode:
        idx = self._rr_index % len(candidates)
        self._rr_index = idx + 1
        return candidates[idx]

    # --- placement ----------------------------------------------------------

    def _place(self, task: Task, node: ClusterNode) -> Placement:
        node.allocate(task.cpu, task.memory)
        p = Placement(task_id=task.id, node_id=node.id, cpu=task.cpu, memory=task.memory)
        self.placements[task.id] = p
        logger.debug("Task %s → node %s", task.id, node.name)
        return p
