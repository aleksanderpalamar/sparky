import unittest

from ci.policy import CHECKS, missing_checks, size_problem


class SizePolicyTest(unittest.TestCase):
    def test_new_source_respects_two_hundred_lines(self):
        self.assertIsNone(size_problem("new.rs", None, "x\n" * 200))
        self.assertIsNotNone(size_problem("new.rs", None, "x\n" * 201))

    def test_old_large_file_can_shrink_but_cannot_grow(self):
        original = "x\n" * 260
        self.assertIsNone(size_problem("old.py", original, "x\n" * 259))
        self.assertIsNone(size_problem("old.py", original, original))
        self.assertIsNotNone(size_problem("old.py", original, "x\n" * 261))

    def test_small_file_cannot_become_large(self):
        self.assertIsNotNone(size_problem("file.sh", "x\n" * 190, "x\n" * 201))

    def test_documentation_and_data_are_not_source(self):
        for path in ["README.md", "Cargo.lock", "image.svg"]:
            self.assertIsNone(size_problem(path, None, "x\n" * 400))

    def test_pkgbuild_without_extension_is_source(self):
        self.assertIsNotNone(size_problem("packaging/arch/PKGBUILD", None, "x\n" * 201))


class PullRequestPolicyTest(unittest.TestCase):
    def test_empty_or_unchecked_description_fails(self):
        self.assertEqual(missing_checks(""), list(CHECKS))
        unchecked = "\n".join(f"- [ ] {label}: conferido" for label in CHECKS)
        self.assertEqual(missing_checks(unchecked), list(CHECKS))

    def test_all_acknowledgements_are_required(self):
        checked = "\n".join(f"- [X] {label}: conferido" for label in CHECKS)
        self.assertEqual(missing_checks(checked), [])
        self.assertEqual(missing_checks(checked.replace("[X]", "[ ]", 1)), [CHECKS[0]])

    def test_checked_text_inside_code_is_not_an_acknowledgement(self):
        checked = "\n".join(f"- [x] {label}: conferido" for label in CHECKS)
        self.assertEqual(missing_checks(f"```markdown\n{checked}\n```"), list(CHECKS))

    def test_hidden_or_indented_examples_do_not_complete_the_checklist(self):
        checked = "\n".join(f"- [x] {label}: conferido" for label in CHECKS)
        self.assertEqual(missing_checks(f"<!--\n{checked}\n-->"), list(CHECKS))
        indented = "\n".join(f"    {line}" for line in checked.splitlines())
        self.assertEqual(missing_checks(indented), list(CHECKS))
