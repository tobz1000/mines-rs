#!/usr/bin/env python3
import argparse
import sys
import subprocess

valid_features_native = [
    ["cli"],
    ["native_support"],
    ["js_server_connector"],
    ["mongodb_connector"],
    ["js_server_connector", "mongodb_connector"],
]

valid_features_wasm = [["webapp"]]

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
    def check(features, wasm):
        check_args = [
            "cargo",
            "build" if wasm else "check",
            "--features={}".format(",".join(features)),
            "--no-default-features"
        ]

        if wasm:
            check_args.append("--target=wasm32-unknown-unknown")

        call(check_args)

    for features in valid_features_native:
        check(features, wasm=False)

    for features in valid_features_wasm:
        check(features, wasm=True)

if __name__ == "__main__":
    try:
        {
            "cli": build_cli,
            "webapp": build_webapp,
            "check": check_valid_features,
        }[args.action]()
    except subprocess.CalledProcessError as err:
        exit(err.returncode)