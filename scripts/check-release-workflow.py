#!/usr/bin/env python3
"""Fail closed when the first-party release provenance policy drifts."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ACTION_SHA = "4d101475d8b20a2381f78447822ac1eab6504dd8"
SUBJECT = "${{ inputs.game }}/build/${{ inputs.game }}.tcade"


def check(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    failures: list[str] = []

    if "\npermissions: {}\n" not in text:
        failures.append("workflow-level permissions must default to none")

    permission_match = re.search(
        r"(?m)^    permissions:\n((?:^      [a-z-]+: [^\n]+\n)+)", text
    )
    permissions: dict[str, str] = {}
    if permission_match:
        for line in permission_match.group(1).splitlines():
            key, value = line.strip().split(":", 1)
            permissions[key] = value.strip().split(" #", 1)[0]
    expected_permissions = {
        "attestations": "write",
        "contents": "write",
        "id-token": "write",
    }
    if permissions != expected_permissions:
        failures.append(
            f"release job permissions must be exactly {expected_permissions}, got {permissions}"
        )

    external_actions = re.findall(r"(?m)^\s*- uses: ([^\s@]+)@([^\s]+)", text)
    for action, revision in external_actions:
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            failures.append(f"{action} must be pinned to a full 40-character SHA")

    attestation_use = (
        "uses: actions/attest-build-provenance@" + ACTION_SHA + " # v4.2.2"
    )
    if text.count(attestation_use) != 1:
        failures.append("attestation action must occur once at the approved v4.2.2 SHA")

    subject_line = f"          subject-path: {SUBJECT}"
    if text.count(subject_line) != 1:
        failures.append(f"attestation subject must be exactly {SUBJECT}")

    ordered_steps = [
        "      - name: Build the package",
        "      - name: Attest package provenance",
        "      - name: Verify package provenance policy",
        "      - name: Tag and release",
        "      - name: Publish to the marketplace",
    ]
    positions = [text.find(step) for step in ordered_steps]
    if -1 in positions or positions != sorted(positions) or len(set(positions)) != len(positions):
        failures.append("release steps must remain build, attest, verify, release, publish")

    required_fragments = [
        'sha256sum "$GAME/build/$GAME.tcade" | tee "$GAME/build/$GAME.tcade.sha256"',
        '"$GAME/build/$GAME.tcade.sha256" \\',
        '--signer-workflow "$GITHUB_REPOSITORY/.github/workflows/release.yml"',
        "--source-ref refs/heads/main",
        '--source-digest "$GITHUB_SHA"',
        '--signer-digest "$GITHUB_SHA"',
        "--deny-self-hosted-runners",
        "--predicate-type https://slsa.dev/provenance/v1",
        'if [ -z "${TERMCADE_TOKEN:-}" ]; then',
        "exit 1",
        "go run github.com/aviorstudio/termcade@v0.0.5 publish",
    ]
    for fragment in required_fragments:
        if fragment not in text:
            failures.append(f"required release control is missing: {fragment}")

    return failures


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} RELEASE_WORKFLOW", file=sys.stderr)
        return 2

    failures = check(Path(sys.argv[1]))
    if failures:
        print("release workflow policy failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("release workflow policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
