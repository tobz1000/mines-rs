#!/usr/bin/env python3
import argparse
import sys
import subprocess

native_build_configs = [
    ["--bin=mines-rs-cli", "--features=cli"],
    ["--lib", "--features=native_support"],
    ["--lib", "--features=js_server_connector"],
    ["--lib", "--features=mongodb_connector"],
    ["--lib", "--features=js_server_connector,mongodb_connector"],
]

wasm_build_configs = [
    ["--bin=mines-rs-webapp", "--features=webapp"]
]

parser = argparse.ArgumentParser()

subparsers = parser.add_subparsers(dest="action")

cli_parser = subparsers.add_parser("cli", help="Build mines-rs-cli")
cli_parser.add_argument(
    "--run",
    "-r",
    help="Run executable after building",
    action="store_true"
)

webapp_parser = subparsers.add_parser("webapp", help="Build mines-rs-webapp")
webapp_parser.add_argument(
    "--run",
    "-r",
    help="Run executable after building",
    action="store_true"
)

check_parser = subparsers.add_parser("check", help="Check each feature combination")

# parser.add_argument('--', dest="pass_args", nargs=argparse.REMAINDER)

args = parser.parse_args()

if args.action is None:
    parser.print_help()
    sys.exit(1)

def call(cmd, wasm=False):
    print(" ".join(cmd))
    subprocess.check_call(cmd)

def build_cli():
    return call([
        "cargo",
        "run" if args.run else "build",
        "--features=cli",
        "--bin=mines-rs-cli"
    ])

def build_webapp():
    call([
        "cargo",
        "web",
        "start" if args.run else "build",
        "--features=webapp",
        "--target=wasm32-unknown-unknown",
        "--bin=mines-rs-webapp"
    ])

def check_valid_features():
    native_cmd = ["cargo", "check", "--no-default-features"]

    wasm_cmd = ["cargo", "web", "build", "--no-default-features", "--target=wasm32-unknown-unknown"]

    for check_cmd, target_configs in [
        (native_cmd, native_build_configs),
        (wasm_cmd, wasm_build_configs)
    ]:
        for append_args in target_configs:
            call(check_cmd + append_args)

if __name__ == "__main__":
    try:
        {
            "cli": build_cli,
            "webapp": build_webapp,
            "check": check_valid_features,
        }[args.action]()
    except subprocess.CalledProcessError as err:
        exit(err.returncode)