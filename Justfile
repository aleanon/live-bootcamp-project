default:
    @just --list


compose-up:
    docker compose -f compose.dev.yml up -d

compose-down:
    docker compose -f compose.dev.yml down
