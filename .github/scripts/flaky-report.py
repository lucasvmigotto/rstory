"""Report flaky tests from a cargo-nextest JUnit file (test-strategy §8: reported, never silently green).

Usage: python3 -I flaky-report.py <junit.xml>
Emits one GitHub warning annotation per flaky test and a job-summary table. Test names only;
nextest output (stdout/stderr) is never copied, so no test data reaches the summary.
"""

import os
import sys
import xml.etree.ElementTree as ET


def main() -> int:
    path = sys.argv[1]
    if not os.path.exists(path):
        print(f"no JUnit report at {path}; nothing to report")
        return 0
    flaky = []
    for case in ET.parse(path).getroot().iter("testcase"):
        reruns = len(case.findall("flakyFailure")) + len(case.findall("flakyError"))
        if reruns:
            flaky.append((case.get("classname", ""), case.get("name", ""), reruns))
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    lines = []
    if flaky:
        lines += ["### Flaky tests (passed on rerun)", "", "| Binary | Test | Failed attempts |", "|---|---|---|"]
        for binary, name, n in flaky:
            print(f"::warning title=Flaky test::{binary} {name} failed {n} time(s) before passing")
            lines.append(f"| `{binary}` | `{name}` | {n} |")
        lines += ["", "Twice in 14 days means quarantine: see docs/release.md, Flaky-test policy."]
    else:
        lines.append("No flaky tests in this run.")
    print("\n".join(lines))
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
