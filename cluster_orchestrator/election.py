"""LeaderElection — RAFT-style leader election for cluster nodes."""

from __future__ import annotations

import logging
import random
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

logger = logging.getLogger(__name__)


class ElectionState(Enum):
    FOLLOWER = "follower"
    CANDIDATE = "candidate"
    LEADER = "leader"


@dataclass
class VoteRequest:
    term: int
    candidate_id: str


@dataclass
class VoteResponse:
    term: int
    vote_granted: bool


@dataclass
class LeaderElection:
    """RAFT-style leader election tracker.

    Call ``tick()`` periodically to drive timeouts.  Use ``request_vote()``
    and ``handle_vote()`` to simulate RPCs between nodes.
    """

    node_id: str
    peers: list[str] = field(default_factory=list)
    election_timeout_range: tuple[float, float] = (1.5, 3.0)
    heartbeat_interval: float = 0.5

    # internal state
    current_term: int = 0
    voted_for: Optional[str] = None
    state: ElectionState = ElectionState.FOLLOWER
    votes_received: set[str] = field(default_factory=set)
    last_heartbeat: float = field(default_factory=time.time)
    _election_deadline: float = 0.0
    _random: random.Random = field(default_factory=random.Random)

    def __post_init__(self) -> None:
        self._reset_election_timeout()

    # --- helpers ------------------------------------------------------------

    def _reset_election_timeout(self) -> None:
        lo, hi = self.election_timeout_range
        self._election_deadline = time.time() + self._random.uniform(lo, hi)

    @property
    def is_leader(self) -> bool:
        return self.state == ElectionState.LEADER

    @property
    def quorum(self) -> int:
        total = 1 + len(self.peers)  # self + peers
        return total // 2 + 1

    # --- tick (timeout driver) ----------------------------------------------

    def tick(self) -> Optional[ElectionState]:
        """Drive election timeouts.  Returns new state if a transition occurred."""
        now = time.time()

        if self.state == ElectionState.LEADER:
            # leaders don't time out
            return None

        if now >= self._election_deadline:
            return self._start_election()

        return None

    def _start_election(self) -> ElectionState:
        self.current_term += 1
        self.state = ElectionState.CANDIDATE
        self.voted_for = self.node_id
        self.votes_received = {self.node_id}
        self._reset_election_timeout()
        logger.info("Node %s starting election for term %d", self.node_id, self.current_term)
        return self.state

    # --- RPC simulation -----------------------------------------------------

    def request_vote(self) -> VoteRequest:
        """Create a vote request to send to peers."""
        return VoteRequest(term=self.current_term, candidate_id=self.node_id)

    def handle_vote_request(self, req: VoteRequest) -> VoteResponse:
        """Process an incoming vote request."""
        if req.term > self.current_term:
            # higher term — become follower
            self.current_term = req.term
            self.state = ElectionState.FOLLOWER
            self.voted_for = None

        if req.term < self.current_term:
            return VoteResponse(term=self.current_term, vote_granted=False)

        if self.voted_for is None or self.voted_for == req.candidate_id:
            self.voted_for = req.candidate_id
            self.last_heartbeat = time.time()
            self._reset_election_timeout()
            return VoteResponse(term=self.current_term, vote_granted=True)

        return VoteResponse(term=self.current_term, vote_granted=False)

    def handle_vote_response(self, resp: VoteResponse, voter_id: str) -> Optional[ElectionState]:
        """Process a vote response.  Returns new state if elected leader."""
        if resp.term > self.current_term:
            self.current_term = resp.term
            self.state = ElectionState.FOLLOWER
            self.voted_for = None
            return self.state

        if self.state != ElectionState.CANDIDATE:
            return None

        if resp.vote_granted:
            self.votes_received.add(voter_id)
            if len(self.votes_received) >= self.quorum:
                self.state = ElectionState.LEADER
                self.last_heartbeat = time.time()
                logger.info(
                    "Node %s elected leader for term %d", self.node_id, self.current_term
                )
                return self.state

        return None

    def receive_heartbeat(self, leader_term: int) -> None:
        """Handle a heartbeat from a leader."""
        if leader_term >= self.current_term:
            self.current_term = leader_term
            self.state = ElectionState.FOLLOWER
            self.voted_for = None
            self.last_heartbeat = time.time()
            self._reset_election_timeout()

    def step_down(self) -> None:
        """Force this node to become a follower (e.g. on higher-term discovery)."""
        self.state = ElectionState.FOLLOWER
        self.voted_for = None
        self._reset_election_timeout()
