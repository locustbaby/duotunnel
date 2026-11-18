import argparse
import contextlib
import importlib.util
import io
import pathlib
import tempfile
import types
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("bench_tool", pathlib.Path(__file__).with_name("bench-tool.py"))
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


class BenchmarkInputTests(unittest.TestCase):
    def test_collector_restarts_with_the_original_argument_boundaries(self):
        argv = ["bench-tool.py", "collect", "--output", "path with spaces.jsonl"]
        executable = "/test path/bin/python3"
        with (
            patch.dict(bench.sys.modules, {"psutil": None}),
            patch.object(bench.sys, "argv", argv),
            patch.object(bench.sys, "executable", executable),
            patch.object(bench.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)) as install,
            patch.object(bench.os, "execv", side_effect=SystemExit) as restart,
            contextlib.redirect_stderr(io.StringIO()),
        ):
            with self.assertRaises(SystemExit):
                bench.run_collect(None)
        install.assert_called_once_with([executable, "-m", "pip", "install", "-q", "psutil"], check=False)
        restart.assert_called_once_with(executable, [executable, *argv])

    def test_systemctl_receives_a_single_validated_unit_operand(self):
        with self.assertRaises(argparse.ArgumentTypeError):
            bench._scope_name("--all")
        with self.assertRaises(argparse.ArgumentTypeError):
            bench._scope_name("duotunnel-*.scope")
        scope = bench._scope_name("duotunnel-server.scope")
        with patch.object(bench, "_run_capture", return_value={"status": 0, "output": "TasksCurrent=2"}) as run:
            result = bench._systemd_scope_snapshot(scope)
        self.assertEqual(run.call_args.args[0][-3:], ["show", "--", scope])
        self.assertEqual(result["properties"], {"TasksCurrent": "2"})

    def test_redaction_preserves_secret_keys_and_handles_long_inputs(self):
        large_key = 'DUOTUNNEL_' * 1000
        value = f"  token: secret\nDUOTUNNEL_CLIENT__AUTH_TOKEN=secret\n{large_key}\n"
        expected = f"  token: <redacted>\nDUOTUNNEL_CLIENT__AUTH_TOKEN=<redacted>\n{large_key}\n"
        self.assertEqual(bench._redact_text(value), expected)
        self.assertEqual(bench._redact_text("OTHER_DUOTUNNEL_SECRET=discard"), "OTHER_DUOTUNNEL_SECRET=<redacted>")
        spaces = " " * 10000
        self.assertEqual(bench._redact_text(f"DUOTUNNEL_TOKEN={spaces}"), f"DUOTUNNEL_TOKEN={spaces}")
        self.assertEqual(
            bench._redact_text(f"DUOTUNNEL_TOKEN={spaces}discard DUOTUNNEL_PASSWORD: other"),
            f"DUOTUNNEL_TOKEN={spaces}<redacted> DUOTUNNEL_PASSWORD: <redacted>",
        )

    def test_invalid_publish_ids_are_rejected_before_any_file_access(self):
        for sha, run_id in [("../invalid", "1"), ("abcdef1", "../invalid")]:
            args = types.SimpleNamespace(sha=sha, run_id=run_id)
            with self.subTest(sha=sha, run_id=run_id), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(bench.run_publish(args), 2)

    def test_publish_writes_details_under_the_index_with_a_numeric_run_id(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            result = root / "result.json"
            result.write_text("{}")
            data = root / "data.js"
            args = types.SimpleNamespace(
                sha="abcdef123", run_id="", run_url="https://example.test/actions/runs/123",
                result=result, data=data, msg="", url="", trace_cases_file="", max_entries=2, max_traces=1,
            )
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertIsNone(bench.run_publish(args))
            self.assertTrue((root / "data" / "abcdef1-123.json").is_file())
            self.assertIn("abcdef1-123", data.read_text())


if __name__ == "__main__":
    unittest.main()
