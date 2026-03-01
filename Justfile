mod? local

fmt:
    find . \( -name BUCK -o -name '*.bzl' \) | xargs -n1 buildifier --path=BUILD
    cargo fmt --all

lint:
    cargo clippy --fix --allow-dirty --allow-staged --all-features --all-targets
    buck2 starlark lint //...