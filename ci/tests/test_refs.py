import unittest

from ci.refs import event_base, pull_request_problem


class EventRefsTest(unittest.TestCase):
    def test_pull_request_uses_the_target_commit(self):
        event = {"pull_request": {"base": {"sha": "base"}, "head": {"ref": "fix/test"}}}
        self.assertEqual(event_base("pull_request", event), "base")

    def test_merge_queue_uses_the_queue_base(self):
        self.assertEqual(event_base("merge_group", {"merge_group": {"base_sha": "base"}}), "base")

    def test_push_uses_previous_tip_and_rejects_null_sha(self):
        self.assertEqual(event_base("push", {"before": "abc"}), "abc")
        self.assertIsNone(event_base("push", {"before": "0" * 40}))
        self.assertIsNone(event_base("workflow_dispatch", {}))

    def test_pull_request_requires_a_separate_branch(self):
        event = {"pull_request": {"base": {"ref": "main"}, "head": {"ref": "main"}}}
        self.assertIsNotNone(pull_request_problem(event))
        event["pull_request"]["head"]["ref"] = "ci/gate"
        self.assertIsNone(pull_request_problem(event))
