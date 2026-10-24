# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
import unittest
from scripts import coverage_correspondence as coverage


class CorrespondenceTests(unittest.TestCase):
    def test_rejection_branch_is_missing_then_observed_without_test_attribution(self):
        path="crates/example/src/lib.rs"
        file={"filename":str(coverage.ROOT/path),"segments":[[3,1,0,True,True,False],[4,1,0,False,False,False]],"branches":[[3,1,3,8,0,1,0,0,4]]}
        report={"data":[{"files":[file]}]}
        missing=coverage.correspondence(report,{path:[{"start":3,"lines":1}]})[0]
        self.assertEqual(missing["missing_lines"],[3])
        self.assertEqual(missing["branches"][0]["true_count"],0)
        file["segments"][0][2]=1
        file["branches"][0][4]=1
        reached=coverage.correspondence(report,{path:[{"start":3,"lines":1}]})[0]
        self.assertEqual(reached["covered_lines"],[3])
        self.assertEqual(reached["missing_lines"],[])
        self.assertEqual(coverage.correspondence(report,{"crates/example/tests/a.rs":[{"start":1,"lines":1}]}),[])
