

test:
    cargo test -p changelog_document test_file  -- --show-output

test_perf:
    cargo test -p changelog_document changelog2 --release  -- --show-output