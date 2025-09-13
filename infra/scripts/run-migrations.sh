#!/usr/bin/env bash

# shellcheck disable=SC2046
export $(grep -v '^#' infra/environment/node.env | xargs)

echo "Starting refinery migration"
cargo run --package migration
sleep 2
echo "Finished refinery migration"