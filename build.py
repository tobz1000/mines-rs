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

args = parser.parse_args()

if args.action is None:
    parser.print_help()
    sys.exit(1)

def check_install_cargo_web():
    if check_install_cargo_web.checked:
        return

    if subprocess.call(
        ["cargo", "web", "-V"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    ) != 0:
        cargo(["install", "cargo-web"])

    check_install_cargo_web.checked = True

check_install_cargo_web.checked = False

def cargo(args):
    cmd = ["cargo"] + args
    print(" ".join(cmd))
    subprocess.check_call(cmd)

def cargo_web(args):
    check_install_cargo_web()
    cmd = ["cargo", "web"] + args + ["--target=wasm32-unknown-unknown"]
    print(" ".join(cmd))
    subprocess.check_call(cmd)

def build_cli():
    cargo([
        "run" if args.run else "build",
        "--release",
        "--features=cli",
        "--bin=mines-rs-cli"
    ])

def build_webapp():
    cargo_web([
        "start" if args.run else "build",
        "--features=webapp",
        "--bin=mines-rs-webapp"
    ])

def check_valid_features():
    for append_args in native_build_configs:
        cargo(["check", "--no-default-features"] + append_args)

    for append_args in wasm_build_configs:
        cargo_web(["build", "--no-default-features"] + append_args)

if __name__ == "__main__":
    try:
        {
            "cli": build_cli,
            "webapp": build_webapp,
            "check": check_valid_features,
        }[args.action]()
    except subprocess.CalledProcessError as err:
        exit(err.returncode)