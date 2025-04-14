_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

run-server *ARGS:
	cargo run {{ARGS}}
	
build:
	cargo build

fmt:
	cargo fmt -- --config "group_imports=StdExternalCrate"
