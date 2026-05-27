"""ClusterNode — represents a single node in the cluster."""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class NodeRole(Enum):
    WORKER = "worker"
    MANAGER = "manager"
    LEADER = "leader"


class NodeHealth(Enum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    UNKNOWN = "unknown"


@dataclass
class ClusterNode:
    """A node in the cluster with capacity tracking and health."""

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    name: str = ""
    role: NodeRole = NodeRole.WORKER
    health: NodeHealth = NodeHealth.UNKNOWN
    capacity_cpu: float = 1.0  # total CPU units
    capacity_memory: float = 1.0  # total memory units
    used_cpu: float = 0.0
    used_memory: float = 0.0
    labels: dict[str, str] = field(default_factory=dict)
    metadata: dict[str, Any] = field(default_factory=dict)
    joined_at: float = field(default_factory=time.time)
    last_heartbeat: float = field(default_factory=time.time)

    def __post_init__(self) -> None:
        if not self.name:
            self.name = f"node-{self.id[:8]}"

    # --- capacity helpers ---------------------------------------------------

    @property
    def available_cpu(self) -> float:
        return max(0.0, self.capacity_cpu - self.used_cpu)

    @property
    def available_memory(self) -> float:
        return max(0.0, self.capacity_memory - self.used_memory)

    @property
    def cpu_utilization(self) -> float:
        """Return CPU utilization as a fraction 0..1."""
        if self.capacity_cpu == 0:
            return 0.0
        return self.used_cpu / self.capacity_cpu

    @property
    def memory_utilization(self) -> float:
        if self.capacity_memory == 0:
            return 0.0
        return self.used_memory / self.capacity_memory

    def can_allocate(self, cpu: float, memory: float) -> bool:
        return self.available_cpu >= cpu and self.available_memory >= memory

    def allocate(self, cpu: float, memory: float) -> bool:
        """Try to allocate resources. Returns True on success."""
        if not self.can_allocate(cpu, memory):
            return False
        self.used_cpu += cpu
        self.used_memory += memory
        return True

    def release(self, cpu: float, memory: float) -> None:
        """Release previously allocated resources."""
        self.used_cpu = max(0.0, self.used_cpu - cpu)
        self.used_memory = max(0.0, self.used_memory - memory)

    # --- health helpers -----------------------------------------------------

    def heartbeat(self) -> None:
        self.last_heartbeat = time.time()
        if self.health == NodeHealth.UNKNOWN:
            self.health = NodeHealth.HEALTHY

    def is_alive(self, timeout: float = 30.0) -> bool:
        """Check if the node has sent a heartbeat within *timeout* seconds."""
        return (time.time() - self.last_heartbeat) < timeout

    def __repr__(self) -> str:  # pragma: no cover
        return (
            f"ClusterNode(id={self.id!r}, name={self.name!r}, "
            f"role={self.role.value}, health={self.health.value})"
        )
