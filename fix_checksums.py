import hashlib
import json
import os

VENDOR_DIR = "vendor/criterion"
CHECKSUM_FILE = os.path.join(VENDOR_DIR, ".cargo-checksum.json")

def sha256_file(filepath):
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        h.update(f.read())
    return h.hexdigest()

if not os.path.exists(CHECKSUM_FILE):
    print(f"Error: {CHECKSUM_FILE} not found")
    exit(1)

with open(CHECKSUM_FILE, "r") as f:
    data = json.load(f)

for filename in data["files"]:
    filepath = os.path.join(VENDOR_DIR, filename)
    if os.path.exists(filepath):
        actual = sha256_file(filepath)
        if actual != data["files"][filename]:
            print(f"Updating {filename}: {data['files'][filename]} -> {actual}")
            data["files"][filename] = actual

with open(CHECKSUM_FILE, "w") as f:
    json.dump(data, f, separators=(',', ':'))
print("Checksums updated.")
