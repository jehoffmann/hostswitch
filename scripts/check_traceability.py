#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Checks the chain OpenSpec scenario -> Gherkin scenario -> Rust test.

Rules (exit status 1 on any violation, gaps are only reported):
  1. Every `#### Scenario: <title>` of openspec/specs/<cap>/spec.md has a Gherkin scenario with the
     same title tagged @spec-<cap> (in crates/*/tests/features/*.feature).
  2. Every Gherkin scenario has a level (@unit @integration @system), a @spec-<cap> whose spec has
     that title, a @req-<ID>, and exactly one way of being verified: @bdd (run by cucumber), at least
     one @test:<crate>/<path>::<fn> (an existing Rust test), @manual (needs real hardware, run by a
     person) or @gap (not verified yet; listed below).
  3. Every @test: tag points to an existing file that contains `fn <name>`.
  4. Every ADR in docs/adr/NNNN-slug.md has the title `# ADR-NNNN: ...`, a status, a `## Requirements`
     section with at least one reference, and every reference exists:
       spec:<capability>/<ID>    a `### Requirement: <ID>` in openspec/specs/<capability>/spec.md
       change:<name>[/<ID>]      an open change (and, with ID, a requirement of its delta specs)
       constraint:<ID>           a row of the constraints table in docs/requirements.md
     docs/adr/README.md lists every ADR.
Usage: scripts/check_traceability.py [--list-gaps] [--list-unreferenced]
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def spec_scenarios():
    found = {}
    for spec in sorted((ROOT / "openspec/specs").glob("*/spec.md")):
        cap = spec.parent.name
        for m in re.finditer(r"^#### Scenario: (.+)$", spec.read_text(encoding="utf-8"), re.M):
            found[(cap, m.group(1).strip())] = spec
    return found


def gherkin_scenarios():
    scenarios = []
    for feature in sorted((ROOT / "crates").glob("*/tests/features/*.feature")):
        tags = []
        for number, line in enumerate(feature.read_text(encoding="utf-8").splitlines(), 1):
            stripped = line.strip()
            if stripped.startswith("@"):
                tags += stripped.split()
            elif stripped.startswith(("Scenario:", "Scenario Outline:")):
                title = stripped.split(":", 1)[1].strip()
                scenarios.append((feature, number, title, tags))
                tags = []
            elif stripped and not stripped.startswith("#"):
                tags = []
    return scenarios


def requirement_ids(spec_file):
    return set(re.findall(r"^### Requirement: (\S+)", spec_file.read_text(encoding="utf-8"), re.M))


def check_adrs(errors):
    """Returns the set of (capability, ID) pairs that at least one ADR references."""
    adr_dir = ROOT / "docs/adr"
    spec_ids = {spec.parent.name: requirement_ids(spec) for spec in (ROOT / "openspec/specs").glob("*/spec.md")}
    change_ids = {}
    for change in (ROOT / "openspec/changes").iterdir():
        if change.is_dir() and change.name != "archive":
            ids = set()
            for spec in change.glob("specs/*/spec.md"):
                ids |= requirement_ids(spec)
            change_ids[change.name] = ids
    constraints = set(re.findall(r"^\| (N-\d+) \|", (ROOT / "docs/requirements.md").read_text(encoding="utf-8"), re.M))
    index = (adr_dir / "README.md").read_text(encoding="utf-8") if (adr_dir / "README.md").is_file() else ""
    referenced = set()
    files = sorted(adr_dir.glob("[0-9][0-9][0-9][0-9]-*.md"))
    for adr in files:
        where = str(adr.relative_to(ROOT))
        text = adr.read_text(encoding="utf-8")
        number = adr.name[:4]
        if not re.match(rf"# ADR-{number}: \S", text):
            errors.append(f"{where}: first line must be '# ADR-{number}: <title>'")
        if not re.search(r"^- Status: (Proposed|Accepted|Superseded by ADR-\d{4})$", text, re.M):
            errors.append(f"{where}: missing '- Status: Proposed|Accepted|Superseded by ADR-NNNN'")
        if f"./{adr.name}" not in index:
            errors.append(f"{where}: not listed in docs/adr/README.md")
        section = re.search(r"^## Requirements\n(.*?)(?=^## )", text, re.M | re.S)
        refs = re.findall(r"^- ((?:spec|change|constraint):\S+)", section.group(1), re.M) if section else []
        if not refs:
            errors.append(f"{where}: '## Requirements' needs at least one reference")
        for ref in refs:
            kind, _, target = ref.partition(":")
            if kind == "spec":
                cap, _, req = target.partition("/")
                if req not in spec_ids.get(cap, set()):
                    errors.append(f"{where}: {ref}: no such requirement in openspec/specs/{cap}")
                referenced.add((cap, req))
            elif kind == "change":
                name, _, req = target.partition("/")
                if name not in change_ids:
                    errors.append(f"{where}: {ref}: no open change '{name}'")
                elif req and req not in change_ids[name]:
                    errors.append(f"{where}: {ref}: change '{name}' has no requirement {req}")
            elif target not in constraints:
                errors.append(f"{where}: {ref}: no such constraint in docs/requirements.md")
    for line in re.findall(r"\[(\d{4})\]\(\./", index):
        if not any(f.name.startswith(line) for f in files):
            errors.append(f"docs/adr/README.md lists ADR {line} but the file does not exist")
    return len(files), referenced


def main():
    errors, gaps = [], []
    specs = spec_scenarios()
    gherkin = gherkin_scenarios()
    covered = set()
    for feature, number, title, tags in gherkin:
        where = f"{feature.relative_to(ROOT)}:{number} '{title}'"
        caps = [t[len("@spec-"):] for t in tags if t.startswith("@spec-")]
        if len(caps) != 1:
            errors.append(f"{where}: needs exactly one @spec-<capability> tag")
            continue
        if (caps[0], title) not in specs:
            errors.append(f"{where}: no OpenSpec scenario '{title}' in capability '{caps[0]}'")
        covered.add((caps[0], title))
        if not any(t in tags for t in ("@unit", "@integration", "@system")):
            errors.append(f"{where}: missing level tag (@unit, @integration or @system)")
        if not any(t.startswith("@req-") for t in tags):
            errors.append(f"{where}: missing @req-<ID> tag")
        tests = [t for t in tags if t.startswith("@test:")]
        ways = [bool(tests), "@bdd" in tags, "@manual" in tags, "@gap" in tags]
        if "@gap" in tags:
            gaps.append(where)
        if not any(ways):
            errors.append(f"{where}: not verified (@bdd, @test:..., @manual or @gap)")
        if "@bdd" in tags and "hostswitchctl/tests/features" not in str(feature):
            errors.append(f"{where}: @bdd scenarios must live in a crate that runs cucumber (hostswitchctl)")
        for tag in tests:
            m = re.fullmatch(r"@test:([\w-]+)/(.+?)::(\w+)", tag)
            if not m:
                errors.append(f"{where}: malformed tag {tag}")
                continue
            crate, rel, fn = m.groups()
            path = ROOT / "crates" / crate / rel
            if not path.is_file():
                errors.append(f"{where}: {tag}: file not found")
            elif not re.search(rf"\bfn {re.escape(fn)}\b", path.read_text(encoding="utf-8")):
                errors.append(f"{where}: {tag}: no fn {fn}")
    for (cap, title), spec in sorted(specs.items()):
        if (cap, title) not in covered:
            errors.append(f"{spec.relative_to(ROOT)}: scenario '{title}' has no Gherkin scenario")
    adr_count, referenced = check_adrs(errors)
    all_reqs = {(cap, req) for cap in {c for c, _ in specs} for req in requirement_ids(ROOT / f"openspec/specs/{cap}/spec.md")}
    print(f"{len(specs)} OpenSpec scenarios, {len(gherkin)} Gherkin scenarios, {len(gaps)} gaps")
    print(f"{adr_count} ADRs, {len(referenced & all_reqs)} of {len(all_reqs)} spec requirements referenced by an ADR")
    if "--list-unreferenced" in sys.argv:
        for cap, req in sorted(all_reqs - referenced):
            print(f"  not in any ADR: {cap}/{req}")
    print(f"{len(errors)} errors")
    if "--list-gaps" in sys.argv or errors:
        for gap in gaps:
            print(f"  gap: {gap}")
    for error in errors:
        print(f"  ERROR {error}")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
