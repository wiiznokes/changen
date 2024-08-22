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

gen_all f='res/CHANGELOG_DEFAULT.md':
	cargo run -- generate -f {{f}} --stdout --exclude-unidentified --tag 0.1.7 > CHANGELOG2.md

gen_release f='CHANGELOG3.md' v='':
	cargo run -- release -f {{f}} --stdout

gen_doc:
	cargo run --locked --bin gen-doc