set windows-shell := ["nu.exe", "-c"]
set shell := ["nu", "-c"]

root := absolute_path('')
gitignore := absolute_path('.gitignore')
prettierignore := absolute_path('.prettierignore')

default:
    @just --choose

format:
    cd '{{ root }}'; just --fmt --unstable

    nixpkgs-fmt '{{ root }}'

    prettier --write \
      --ignore-path '{{ gitignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

lint:
    prettier --check \
      --ignore-path '{{ gitignore }}' \
      --ignore-path '{{ prettierignore }}' \
      --cache --cache-strategy metadata \
      '{{ root }}'

    cspell lint '{{ root }}' \
      --no-progress

    markdownlint '{{ root }}'
    markdown-link-check \
      --config .markdown-link-check.json \
      --quiet ...(glob '{{ root }}/**/*.md')
