#!/usr/bin/env python3
"""
Version management script for Telelog.
Keeps Cargo.toml and pyproject.toml versions in sync.
"""

import re
import sys
from pathlib import Path

def update_version(new_version: str) -> None:
    """Update version in both Cargo.toml and pyproject.toml"""
    
    # Validate version format (semantic versioning)
    if not re.match(r'^\d+\.\d+\.\d+([-\w\d\.]+)?(\+[\w\d\.]+)?$', new_version):
        print(f"Error: Invalid version format '{new_version}'. Use semantic versioning (e.g., 1.0.0)")
        sys.exit(1)
    
    project_root = Path(__file__).parent.parent
    
    # Update Cargo.toml
    cargo_toml = project_root / "Cargo.toml"
    if cargo_toml.exists():
        content = cargo_toml.read_text()
        updated = re.sub(r'^version = "[\d\.\w-]+"', f'version = "{new_version}"', content, flags=re.MULTILINE)
        cargo_toml.write_text(updated)
        print(f"✅ Updated Cargo.toml to version {new_version}")
    else:
        print("❌ Cargo.toml not found")
        sys.exit(1)
    
    # Update pyproject.toml
    pyproject_toml = project_root / "pyproject.toml"
    if pyproject_toml.exists():
        content = pyproject_toml.read_text()
        updated = re.sub(r'^version = "[\d\.\w-]+"', f'version = "{new_version}"', content, flags=re.MULTILINE)
        pyproject_toml.write_text(updated)
        print(f"✅ Updated pyproject.toml to version {new_version}")
    else:
        print("❌ pyproject.toml not found")
        sys.exit(1)
    
    print(f"\n🎉 Successfully updated version to {new_version}")
    print("\nNext steps:")
    print("1. Review changes: git diff")
    print("2. Test: cargo test && maturin develop && python -m pytest")
    print(f"3. Commit: git add -A && git commit -m 'Bump version to {new_version}'")
    print(f"4. Tag: git tag v{new_version}")
    print("5. Push: git push && git push --tags")

def get_current_version() -> str:
    """Get current version from Cargo.toml"""
    cargo_toml = Path(__file__).parent.parent / "Cargo.toml"
    if not cargo_toml.exists():
        return "unknown"
    
    content = cargo_toml.read_text()
    match = re.search(r'^version = "([\d\.\w-]+)"', content, flags=re.MULTILINE)
    return match.group(1) if match else "unknown"

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python scripts/version.py <new_version>")
        print(f"Current version: {get_current_version()}")
        print("\nExamples:")
        print("  python scripts/version.py 0.1.1    # patch release")
        print("  python scripts/version.py 0.2.0    # minor release")
        print("  python scripts/version.py 1.0.0    # major release")
        print("  python scripts/version.py 0.1.1-beta.1  # pre-release")
        sys.exit(1)
    
    new_version = sys.argv[1]
    update_version(new_version)