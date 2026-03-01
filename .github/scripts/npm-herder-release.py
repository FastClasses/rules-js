import argparse
import hashlib
import json
import os
import urllib.request
from urllib.error import HTTPError

def set_output(name, value):
    out_file = os.environ.get("GITHUB_OUTPUT")
    if out_file:
        with open(out_file, "a") as f:
            f.write(f"{name}={value}\n")
    else:
        print(f"::set-output name={name}::{value}")

def get_version():
    with open("npm-herder/Cargo.toml", "r") as f:
        for line in f:
            if line.startswith("version ="):
                return line.split("=")[1].strip().strip('"')
    return "0.0.0"

def cmd_check():
    version = get_version()
    tag = f"npm-herder@v{version}"
    try:
        url = f"https://api.github.com/repos/FastClasses/rules-js/releases/tags/{tag}"
        req = urllib.request.Request(url)
        req.add_header("User-Agent", "npm-herder-release-script")
        
        if "GITHUB_TOKEN" in os.environ:
            req.add_header("Authorization", f"Bearer {os.environ['GITHUB_TOKEN']}")
        
        urllib.request.urlopen(req)
        print(f"Release {tag} already exists.")
        set_output("needs_release", "false")
    except HTTPError as e:
        if e.code == 404:
            print(f"Release {tag} does not exist. A new release will be created.")
            set_output("needs_release", "true")
            set_output("version", version)
            set_output("tag", tag)
        else:
            raise e

def compute_sha256(filepath):
    sha256_hash = hashlib.sha256()
    with open(filepath,"rb") as f:
        for byte_block in iter(lambda: f.read(4096),b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()

def cmd_publish():
    version = get_version()
    tag = f"npm-herder@v{version}"
    repo = os.environ.get("GITHUB_REPOSITORY", "FastClasses/rules-js")
    
    platforms = {}
    
    for ds_plat, artifact_name, bin_name in [
        ("linux-x86_64", "npm-herder-linux-x86_64.tar.gz", "npm-herder"),
        ("macos-aarch64", "npm-herder-macos-aarch64.tar.gz", "npm-herder"),
        ("macos-x86_64", "npm-herder-macos-x86_64.tar.gz", "npm-herder"),
        ("windows-x86_64", "npm-herder-windows-x86_64.tar.gz", "npm-herder.exe"),
    ]:
        artifact_path = os.path.join("dist", artifact_name)
        if not os.path.exists(artifact_path):
            print(f"Warning: {artifact_path} not found. Skipping platform {ds_plat}.")
            continue
            
        size = os.path.getsize(artifact_path)
        digest = compute_sha256(artifact_path)
        
        platforms[ds_plat] = {
            "size": size,
            "hash": "sha256",
            "digest": digest,
            "format": "tar.gz",
            "path": bin_name,
            "providers": [
                {
                    "type": "github-release",
                    "repo": repo,
                    "tag": tag,
                    "name": artifact_name
                }
            ]
        }
    
    if not platforms:
        print("Error: No artifacts found in dist/. Aborting dotslash generation.")
        exit(1)
        
    dotslash_payload = {
        "name": "npm-herder",
        "platforms": platforms
    }
    
    dotslash_content = "#!/usr/bin/env dotslash\n\n" + json.dumps(dotslash_payload, indent=2) + "\n"
    
    output_path = os.path.join("dist", "npm-herder")
    with open(output_path, "w") as f:
        f.write(dotslash_content)
    
    os.chmod(output_path, 0o755)
    print(f"Successfully generated {output_path} dotslash file.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["check", "publish"])
    args = parser.parse_args()
    
    if args.command == "check":
        cmd_check()
    elif args.command == "publish":
        cmd_publish()
