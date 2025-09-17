default:
    @just --list

compose:
    docker compose -f compose.dev.yml build
    docker compose -f compose.dev.yml down
    docker compose -f compose.dev.yml up -d

compose-up:
    docker compose -f compose.dev.yml up -d

compose-down:
    docker compose -f compose.dev.yml down

compose-build:
    docker compose -f compose.dev.yml build


run name:
    cargo run -p {{name}}-service

log name:
    docker logs live-bootcamp-project-{{name}}-1

test args="":
    cargo nextest run {{args}}
