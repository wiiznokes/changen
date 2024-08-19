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

gen_all:
	cargo run -- generate -f res/CHANGELOG_DEFAULT.md --stdout --exclude-not-pr --exclude-unidentified --tags 0.1.7 > CHANGELOG2.md