import hashlib
import json
import os
import sys

def calculate_sha256(filepath):
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()

def fix_checksums(package_dir):
    checksum_file = os.path.join(package_dir, ".cargo-checksum.json")
    if not os.path.exists(checksum_file):
        print(f"No checksum file found in {package_dir}")
        return

    with open(checksum_file, "r") as f:
        data = json.load(f)

    changed = False
    for filename in data.get("files", {}):
        filepath = os.path.join(package_dir, filename)
        if os.path.exists(filepath):
            current_hash = calculate_sha256(filepath)
            if data["files"][filename] != current_hash:
                print(f"Updating hash for {filename}")
                data["files"][filename] = current_hash
                changed = True
        else:
            print(f"File not found: {filename}")

    if changed:
        with open(checksum_file, "w") as f:
            json.dump(data, f, separators=(',', ':'))
        print("Checksums updated.")
    else:
        print("No changes needed.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 fix_checksums.py <package_dir>")
        sys.exit(1)
    fix_checksums(sys.argv[1])
