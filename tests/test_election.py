"""Tests for cluster_orchestrator.election"""

import time

from cluster_orchestrator.election import ElectionState, LeaderElection


class TestLeaderElection:
    def test_initial_state(self):
        e = LeaderElection(node_id="n1", peers=["n2", "n3"])
        assert e.state == ElectionState.FOLLOWER
        assert e.current_term == 0
        assert e.voted_for is None
        assert e.is_leader is False

    def test_quorum(self):
        e = LeaderElection(node_id="n1", peers=["n2", "n3"])
        assert e.quorum == 2  # 3 nodes total → majority 2

    def test_quorum_five_nodes(self):
        e = LeaderElection(node_id="n1", peers=["n2", "n3", "n4", "n5"])
        assert e.quorum == 3

    def test_start_election(self):
        e = LeaderElection(node_id="n1", peers=["n2", "n3"])
        e._election_deadline = time.time() - 1  # force timeout
        new_state = e.tick()
        assert new_state == ElectionState.CANDIDATE
        assert e.current_term == 1
        assert e.voted_for == "n1"
        assert "n1" in e.votes_received

    def test_election_full_flow(self):
        """Simulate a complete election among 3 nodes."""
        e1 = LeaderElection(node_id="n1", peers=["n2", "n3"], election_timeout_range=(0.1, 0.2))
        e2 = LeaderElection(node_id="n2", peers=["n1", "n3"], election_timeout_range=(0.1, 0.2))
        e3 = LeaderElection(node_id="n3", peers=["n1", "n2"], election_timeout_range=(0.1, 0.2))

        # n1 starts election
        e1._election_deadline = time.time() - 1
        e1.tick()  # becomes candidate

        req = e1.request_vote()
        assert req.term == 1
        assert req.candidate_id == "n1"

        # n2 and n3 grant vote
        resp2 = e2.handle_vote_request(req)
        assert resp2.vote_granted is True
        resp3 = e3.handle_vote_request(req)
        assert resp3.vote_granted is True

        # n1 receives votes — quorum is 2 (self + one peer), so first vote wins
        result = e1.handle_vote_response(resp2, "n2")
        assert result == ElectionState.LEADER
        assert e1.is_leader
        # extra vote is a no-op
        result2 = e1.handle_vote_response(resp3, "n3")
        assert result2 is None  # already leader

    def test_reject_vote_lower_term(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.current_term = 5
        from cluster_orchestrator.election import VoteRequest
        resp = e.handle_vote_request(VoteRequest(term=3, candidate_id="n2"))
        assert resp.vote_granted is False

    def test_reject_double_vote(self):
        e = LeaderElection(node_id="n1", peers=["n2", "n3"])
        from cluster_orchestrator.election import VoteRequest
        req = VoteRequest(term=1, candidate_id="n2")
        e.handle_vote_request(req)  # grants
        resp2 = e.handle_vote_request(VoteRequest(term=1, candidate_id="n3"))
        assert resp2.vote_granted is False  # already voted for n2

    def test_higher_term_forces_follower(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.current_term = 2
        e.state = ElectionState.CANDIDATE
        from cluster_orchestrator.election import VoteRequest
        e.handle_vote_request(VoteRequest(term=5, candidate_id="n2"))
        assert e.state == ElectionState.FOLLOWER
        assert e.current_term == 5

    def test_receive_heartbeat(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.state = ElectionState.CANDIDATE
        e.receive_heartbeat(leader_term=10)
        assert e.state == ElectionState.FOLLOWER
        assert e.current_term == 10

    def test_step_down(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.state = ElectionState.LEADER
        e.step_down()
        assert e.state == ElectionState.FOLLOWER

    def test_leader_tick_no_transition(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.state = ElectionState.LEADER
        result = e.tick()
        assert result is None

    def test_response_with_higher_term(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.state = ElectionState.CANDIDATE
        e.current_term = 1
        from cluster_orchestrator.election import VoteResponse
        result = e.handle_vote_response(VoteResponse(term=5, vote_granted=False), "n2")
        assert result == ElectionState.FOLLOWER
        assert e.current_term == 5

    def test_no_transition_from_response_if_not_candidate(self):
        e = LeaderElection(node_id="n1", peers=["n2"])
        e.state = ElectionState.FOLLOWER
        from cluster_orchestrator.election import VoteResponse
        result = e.handle_vote_response(VoteResponse(term=0, vote_granted=True), "n2")
        assert result is None
