#!/bin/bash

# shellcheck disable=SC2046
export $(grep -v '^#' infra/environment/node.env | xargs)

echo "Starting node in dev mode"
cargo run --package app
echo "Node dev stopped"