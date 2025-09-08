default:
    @just --list


compose-up:
    docker compose -f compose.dev.yml up -d

compose-down:
    docker compose -f compose.dev.yml down

compose-build:
    docker compose -f compose.dev.yml build

compose-build-and-up:
    docker compose -f compose.dev.yml build && docker compose -f compose.dev.yml up -d

run name:
    cargo run -p {{name}}-service

test:
    cargo nextest run
