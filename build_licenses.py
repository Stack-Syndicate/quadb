#!/usr/bin/python
import os
import json
from pathlib import Path
from shutil import copyfile

with open("licenses.json", "r") as f:
    data = json.load(f)

output = Path("THIRD-PARTY-LICENSES.md")
output.write_text("# Third-Party Licenses\n\n")

for crate in data:
    name = crate["name"]
    version = crate["version"]
    license = crate["license"]
    repository = crate.get("repository", "")
    source_path = crate.get("repository", "") or crate.get("repository", "")
    license_file = None

    # Try to find the license file in the local Cargo registry
    registry_path = Path.home() / ".cargo/registry/src"
    for folder in registry_path.glob("*"):
        candidate = folder / f"{name}-{version}"
        if candidate.exists():
            for fname in ["LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "COPYING"]:
                path = candidate / fname
                if path.exists():
                    license_file = path
                    break
        if license_file:
            break

    if not license_file:
        print(f"⚠️  No license file found for {name} {version}")
        continue

    license_text = license_file.read_text(errors="ignore")

    output.write_text(
        output.read_text()
        + f"\n---\n\n## {name} {version}\n"
        + f"**License:** {license}\n\n"
        + (f"**Repository:** {repository}\n" if repository else "")
        + "\n```text\n"
        + license_text.strip()
        + "\n```\n"
    )

print(f"\n✅ License bundle written to: {output}")
