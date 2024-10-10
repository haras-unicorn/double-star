set windows-shell := ["nu.exe", "-c"]
set shell := ["nu", "-c"]

root := absolute_path('')
gitignore := absolute_path('.gitignore')
prettierignore := absolute_path('.prettierignore')
markdown-link-check-rc := absolute_path('.markdown-link-check.json')
db := absolute_path('scripts/db.nu')

default:
    @just --choose

cargo2nix:
    cd '{{ root }}'; yes yes | cargo2nix

format:
    cd '{{ root }}'; just --fmt --unstable

    nixpkgs-fmt '{{ root }}'

    try { markdownlint --ignore-path '{{ gitignore }}' '{{ root }}' }

    prettier --write \
      --ignore-path '{{ gitignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

    cd '{{ root }}'; cargo fmt --all

lint:
    prettier --check \
      --ignore-path '{{ gitignore }}' \
      --ignore-path '{{ prettierignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

    cspell lint '{{ root }}' \
      --no-progress

    markdownlint --ignore-path '{{ gitignore }}' '{{ root }}'
    markdown-link-check \
      --config '{{ markdown-link-check-rc }}' \
      --quiet ...(fd '.*\.md' | lines)

    cd '{{ root }}'; cargo clippy -- -D warnings

test:
    cd '{{ root }}'; cargo test

db *args:
    {{ db }} {{ args }}

[confirm("This will clean docker containers. Do you want to continue?")]
clean:
    docker compose ps -a -q | lines | each { |x| docker stop $x }
    docker compose down -v
    docker compose up -d
    {{ db }} isready
