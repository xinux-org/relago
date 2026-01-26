NIX_SHELL_DIR := source_dir() + "/.nix-shell"


set dotenv-filename := ".env"
set dotenv-load
set export

[doc('Start default server cli with default config')]
start:
  cargo run server run ./config.toml

[doc('Start specific package in the project')]
run crate:
  cargo run --bin {{crate}}

[doc('Lint your rust codebase with clippy')]
lint:
  cargo clippy

[doc('Format your codebase with rust formatter')]
format:
  cargo fmt

[working-directory('./crates/database')]
[confirm("Are you sure you want to do migration?")]
[doc('Perform migration with working database')]
migrate:
  diesel migration run

[confirm("Are you sure you want to delete all data & records?!")]
[doc('Clean up all mess created by development environment')]
clean: db-stop
  rm -rf .nix-shell
  rm -rf .env
  rm -rf target
  rm -rf result

[doc('Kill working postgres instance')]
db-stop:
  pkill postgres

