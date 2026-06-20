#!/usr/bin/env bash
# Build std_msgs Rust bindings in a throwaway colcon workspace (ROS 2 Humble).
set -euo pipefail

WS="${1:-/tmp/spanda-ros2ws}"

if [[ -z "${ROS_DISTRO:-}" ]]; then
  if [[ -f /opt/ros/humble/setup.sh ]]; then
    # shellcheck disable=SC1091
    . /opt/ros/humble/setup.sh
  else
    echo "ROS 2 Humble is not installed or sourced" >&2
    exit 1
  fi
fi

mkdir -p "$WS"
cd "$WS"

if ! command -v vcs >/dev/null 2>&1; then
  echo "python3-vcstool is required (vcs command)" >&2
  exit 1
fi

if ! command -v colcon >/dev/null 2>&1; then
  echo "colcon is required" >&2
  exit 1
fi

REPOS="$WS/ros2_rust_humble.repos"
if [[ ! -f "$REPOS" ]]; then
  curl -fsSL \
    https://raw.githubusercontent.com/ros2-rust/ros2_rust/main/ros2_rust_humble.repos \
    -o "$REPOS"
fi

mkdir -p src
if [[ ! -d src/common_interfaces ]]; then
  vcs import src <"$REPOS"
fi

export RUSTFLAGS="${RUSTFLAGS:-}"
colcon build --packages-select std_msgs

echo "std_msgs Rust bindings installed under $WS/install"
