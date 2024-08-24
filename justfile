set windows-powershell := true
set dotenv-path := ".env"

pull: fmt prettier fix test

###################  Test

test:
	cargo test --workspace --all-features

###################  Format

fix:
	cargo clippy --workspace --all-features --fix --allow-dirty --allow-staged

fmt:
	cargo fmt --all

prettier:
	# install on Debian: sudo snap install node --classic
	# npx is the command to run npm package, node is the runtime
	npx prettier -w .


###################  Handy


test_spe:
    cargo test -p changelog_document test_file  -- --show-output

test_perf:
    cargo test -p changelog_document changelog2 --release  -- --show-output

expand:

gen_notes f='res/CHANGELOG_DEFAULT.md':
	cargo run -- generate -f {{f}} --stdout --exclude-unidentified --since 0.1.7 > CHANGELOG2.md

gen_release f='CHANGELOG2.md' v='1.0.0':
	cargo run -- release -f {{f}} --force -v {{v}}

gen_fmt f='CHANGELOG2.md' v='':
	cargo run -- validate -f {{f}} --fmt --ast


gen_show f='CHANGELOG2.md' v='':
	cargo run -- show -f {{f}} --version 0.1.*

gen_remove f='CHANGELOG2.md' v='':
	cargo run -- remove -f {{f}} -v 1.0.0

gen_doc:
	cargo run --locked --bin gen-doc