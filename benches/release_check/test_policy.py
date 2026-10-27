from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_release import validate, verdict


class RegressionPolicyTests(unittest.TestCase):
    def test_thresholds_are_strict_and_require_every_pair(self):
        cases = [
            (1.05, [1.05] * 3, "pass"),
            (1.051, [1.051] * 3, "warning"),
            (1.10, [1.10] * 3, "warning"),
            (1.101, [1.101] * 3, "fail"),
            (1.20, [1.20, 1.20, 1.01], "pass"),
            (1.20, [1.20, 1.20, 1.06], "warning"),
        ]
        for ratio, pairs, expected in cases:
            with self.subTest(ratio=ratio, pairs=pairs):
                self.assertEqual(verdict({"ratio": ratio, "pair_ratios": pairs}), expected)

    def test_missing_duplicate_and_invalid_samples_fail(self):
        row = {"case": "character/2/insert", "samples_ms": [1.0] * 7}
        validate([row], {row["case"]})
        validate([row], [row["case"]])
        invalid = [
            [],
            [row, row],
            [{**row, "case": "unexpected"}],
            [{**row, "samples_ms": [1.0]}],
            *[[{**row, "samples_ms": [value] * 7}] for value in [0, -1, float("nan"), float("inf")]],
        ]
        for rows in invalid:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                validate(rows, {row["case"]})


if __name__ == "__main__":
    unittest.main()
