import unittest

from ci.changes import parse_changes


class ChangedPathsTest(unittest.TestCase):
    def test_modified_new_and_renamed_paths_keep_the_correct_baseline(self):
        output = "M\0old.rs\0A\0new.py\0R100\0before.rs\0after.rs\0"
        self.assertEqual(parse_changes(output), [("old.rs", "old.rs"), ("new.py", None), ("after.rs", "before.rs")])

    def test_paths_with_spaces_or_newlines_are_not_split(self):
        path = "a folder/a\nfile.rs"
        self.assertEqual(parse_changes(f"M\0{path}\0"), [(path, path)])

    def test_empty_diff_has_no_paths(self):
        self.assertEqual(parse_changes(""), [])
