import sys
import json
import hashlib
import os

def calculate_sha256(filepath):
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()

def fix_checksums(crate_path):
    checksum_file = os.path.join(crate_path, ".cargo-checksum.json")
    if not os.path.exists(checksum_file):
        print(f"Error: {checksum_file} not found.")
        return

    with open(checksum_file, "r") as f:
        data = json.load(f)

    files = data.get("files", {})
    updated = False

    for rel_path, old_checksum in files.items():
        full_path = os.path.join(crate_path, rel_path)
        if not os.path.exists(full_path):
            print(f"Warning: File {rel_path} listed in checksums but not found.")
            continue

        new_checksum = calculate_sha256(full_path)
        if new_checksum != old_checksum:
            print(f"Updating checksum for {rel_path}: {old_checksum} -> {new_checksum}")
            files[rel_path] = new_checksum
            updated = True

    if updated:
        with open(checksum_file, "w") as f:
            json.dump(data, f, separators=(',', ':'))
        print(f"Updated checksums in {checksum_file}")
    else:
        print("No checksum mismatches found.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 fix_vendor_checksums.py <crate_path>")
        sys.exit(1)

    fix_checksums(sys.argv[1])
