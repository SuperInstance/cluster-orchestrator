"""Rebalancer — redistribute tasks when cluster membership changes."""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Optional

from .cluster import Cluster
from .node import ClusterNode
from .scheduler import Placement, Scheduler, SchedulingStrategy, Task

logger = logging.getLogger(__name__)


@dataclass
class RebalanceResult:
    """Summary of a rebalance operation."""
    moved: int = 0
    failed: int = 0
    details: list[str] = None  # type: ignore[assignment]

    def __post_init__(self) -> None:
        if self.details is None:
            self.details = []


class Rebalancer:
    """Monitors cluster imbalance and redistributes tasks."""

    def __init__(
        self,
        cluster: Cluster,
        scheduler: Scheduler,
        imbalance_threshold: float = 0.30,
    ) -> None:
        """
        Args:
            imbalance_threshold: max allowed std-dev of CPU utilisation across
                                 active nodes before triggering rebalance.
        """
        self.cluster = cluster
        self.scheduler = scheduler
        self.imbalance_threshold = imbalance_threshold

    # --- public API ---------------------------------------------------------

    def check_imbalance(self) -> float:
        """Return the standard deviation of CPU utilisation across active nodes."""
        nodes = self.cluster.active_nodes
        if len(nodes) < 2:
            return 0.0
        utils = [n.cpu_utilization for n in nodes]
        mean = sum(utils) / len(utils)
        variance = sum((u - mean) ** 2 for u in utils) / len(utils)
        return variance ** 0.5

    def needs_rebalance(self) -> bool:
        return self.check_imbalance() > self.imbalance_threshold

    def rebalance(self) -> RebalanceResult:
        """Attempt to rebalance tasks across active nodes."""
        result = RebalanceResult()
        if not self.needs_rebalance():
            return result

        # Collect tasks on overloaded nodes
        active = self.cluster.active_nodes
        if len(active) < 2:
            return result

        avg_util = sum(n.cpu_utilization for n in active) / len(active)

        # Find overloaded nodes (above avg + threshold)
        overloaded = [n for n in active if n.cpu_utilization > avg_util + self.imbalance_threshold / 2]

        for node in overloaded:
            # Collect placements on this node
            tasks_here = [
                (tid, p) for tid, p in self.scheduler.placements.items()
                if p.node_id == node.id
            ]
            # Move tasks until node is near average
            for tid, placement in tasks_here:
                if node.cpu_utilization <= avg_util + 0.05:
                    break
                # Deschedule and reschedule
                removed = self.scheduler.deschedule(tid)
                if removed is None:
                    continue
                task = Task(id=tid, cpu=removed.cpu, memory=removed.memory)
                new_placement = self.scheduler.schedule(task)
                if new_placement and new_placement.node_id != node.id:
                    result.moved += 1
                    result.details.append(f"{tid}: {node.name} → {new_placement.node_id}")
                    logger.info("Rebalanced task %s from %s to %s", tid, node.name, new_placement.node_id)
                else:
                    # couldn't move — put it back
                    if new_placement:
                        self.scheduler.deschedule(tid)
                    node.allocate(removed.cpu, removed.memory)
                    fake = Placement(task_id=tid, node_id=node.id, cpu=removed.cpu, memory=removed.memory)
                    self.scheduler.placements[tid] = fake
                    result.failed += 1

        return result

    def handle_node_removal(self, node_id: str) -> RebalanceResult:
        """Redistribute all tasks that were on a removed node."""
        result = RebalanceResult()
        orphaned = [
            (tid, p) for tid, p in list(self.scheduler.placements.items())
            if p.node_id == node_id
        ]
        for tid, placement in orphaned:
            self.scheduler.placements.pop(tid, None)
            task = Task(id=tid, cpu=placement.cpu, memory=placement.memory)
            new_p = self.scheduler.schedule(task)
            if new_p:
                result.moved += 1
                result.details.append(f"{tid}: {node_id} → {new_p.node_id}")
            else:
                result.failed += 1
                result.details.append(f"{tid}: could not reschedule")
        return result
