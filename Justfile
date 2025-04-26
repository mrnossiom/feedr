_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

run-server *ARGS:
	cargo run {{ARGS}}
	
build:
	cargo build

watch:
	# reload both templates and api
	watchexec \
		--exts rs,html \
		--debounce 5s \
		--restart \
		'sh -c "cargo rustc -- -Awarnings && target/debug/feedr-server"'

fmt:
	cargo fmt -- --config "group_imports=StdExternalCrate"
