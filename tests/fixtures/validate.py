#!/usr/bin/env python3
"""Validate test fixtures against upstream network config schemas.

Auto-discovers fixtures by scanning for well-known filenames:
  - network_data.json  -> OpenStack ConfigDrive schema
  - network-config     -> cloud-init NoCloud v1 or v2 (detected by version field)
"""

import json
import sys
import os
import urllib.request

try:
    import yaml
except ImportError:
    yaml = None

try:
    import jsonschema
except ImportError:
    print("ERROR: python3-jsonschema is required. Install with:")
    print("  dnf install python3-jsonschema python3-pyyaml")
    sys.exit(1)

SCHEMA_URLS = {
    "configdrive": "https://raw.githubusercontent.com/openstack/ironic/master/ironic/api/controllers/v1/network-data-schema.json",
    "nocloud-v1": "https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/schema-network-config-v1.json",
    "nocloud-v2": "https://raw.githubusercontent.com/canonical/cloud-init/main/cloudinit/config/schemas/schema-network-config-v2.json",
}

FIXTURES_DIR = os.path.dirname(os.path.abspath(__file__))


def fetch_schema(url):
    """Fetch a JSON schema from a URL."""
    with urllib.request.urlopen(url) as resp:
        return json.loads(resp.read())


def load_data(path):
    """Load JSON or YAML data file."""
    with open(path) as f:
        content = f.read()
    if content.strip().startswith("{"):
        return json.loads(content)
    if yaml is None:
        print(f"ERROR: {path} is YAML but python3-pyyaml is not installed")
        sys.exit(1)
    return yaml.safe_load(content)


def discover_fixtures():
    """Walk tests/fixtures/ and find network config files by well-known names."""
    fixtures = []
    for dirpath, _, filenames in os.walk(FIXTURES_DIR):
        for filename in filenames:
            full_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(full_path, FIXTURES_DIR)

            if filename == "network_data.json":
                fixtures.append(("configdrive", full_path, rel_path))
            elif filename == "network-config":
                data = load_data(full_path)
                version = data.get("version", 1)
                schema_key = "nocloud-v2" if version == 2 else "nocloud-v1"
                fixtures.append((schema_key, full_path, rel_path))

    fixtures.sort(key=lambda f: f[2])
    return fixtures


def main():
    schemas = {}
    for key, url in SCHEMA_URLS.items():
        print(f"Fetching {key} schema...")
        schemas[key] = fetch_schema(url)

    fixtures = discover_fixtures()
    if not fixtures:
        print("No fixtures found")
        return 1

    failed = 0
    passed = 0

    for schema_key, fixture_path, rel_path in fixtures:
        try:
            data = load_data(fixture_path)
            jsonschema.validate(data, schemas[schema_key])
            print(f"  PASS  {rel_path}")
            passed += 1
        except jsonschema.ValidationError as e:
            print(f"  FAIL  {rel_path}")
            print(f"        {e.message}")
            if e.absolute_path:
                print(
                    f"        at: /{'/'.join(str(p) for p in e.absolute_path)}"
                )
            failed += 1
        except Exception as e:
            print(f"  FAIL  {rel_path}")
            print(f"        {e}")
            failed += 1

    print(f"\n{passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
