import hashlib
import json
import os

VENDOR_ROOT = "vendor"

def sha256_file(filepath):
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        h.update(f.read())
    return h.hexdigest()

if not os.path.exists(VENDOR_ROOT):
    print("No vendor directory found.")
    exit(0)

for package in os.listdir(VENDOR_ROOT):
    package_dir = os.path.join(VENDOR_ROOT, package)
    if not os.path.isdir(package_dir):
        continue

    checksum_file = os.path.join(package_dir, ".cargo-checksum.json")
    if os.path.exists(checksum_file):
        with open(checksum_file, "r") as f:
            try:
                data = json.load(f)
            except:
                continue

        modified = False
        for filename in data.get("files", {}):
            filepath = os.path.join(package_dir, filename)
            if os.path.exists(filepath):
                try:
                    actual = sha256_file(filepath)
                    if actual != data["files"][filename]:
                        print(f"Updating {package}/{filename}")
                        data["files"][filename] = actual
                        modified = True
                except:
                    pass

        if modified:
            with open(checksum_file, "w") as f:
                json.dump(data, f, separators=(',', ':'))
print("All checksums verified/updated.")
