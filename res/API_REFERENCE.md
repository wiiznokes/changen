# Command-Line Help for `changen`

This document contains the help content for the `changen` command-line program.

**Command Overview:**

* [`changen`↴](#changen)
* [`changen new`↴](#changen-new)
* [`changen validate`↴](#changen-validate)
* [`changen generate`↴](#changen-generate)
* [`changen release`↴](#changen-release)
* [`changen show`↴](#changen-show)
* [`changen remove`↴](#changen-remove)

## `changen`

Changelog generator

**Usage:** `changen <COMMAND>`

###### **Subcommands:**

* `new` — Create a new changelog file with an accepted syntax
* `validate` — Validate a changelog syntax
* `generate` — Generate release notes. By default, generate from the last release in the changelog to HEAD
* `release` — Generate a new release. By default, use the last tag present in the repo
* `show` — Show a releases on stdout. By default, show the last release
* `remove` — Remove a release



## `changen new`

Create a new changelog file with an accepted syntax

**Usage:** `changen new [OPTIONS]`

###### **Options:**

* `-p`, `--path <PATH>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `-f`, `--force` — Override of existing file



## `changen validate`

Validate a changelog syntax

**Usage:** `changen validate [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `--format` — Format the changelog
* `--map <MAP>` — Path to the commit type to changelog section map
* `--ast` — Show the Abstract Syntax Tree
* `--stdout` — Print the result on the standard output



## `changen generate`

Generate release notes. By default, generate from the last release in the changelog to HEAD

**Usage:** `changen generate [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `--map <MAP>` — Path to the commit type to changelog section map
* `--parsing <PARSING>` — Parsing of the commit message

  Default value: `smart`

  Possible values: `smart`, `strict`

* `--exclude-unidentified` — Don't include unidentified commits
* `--exclude-not-pr` — Don't include commits which are not attached to a pull request
* `--provider <PROVIDER>` — We use the Github api to map commit sha to PRs

  Default value: `github`

  Possible values: `github`, `none`

* `--repo <REPO>` — Needed for fetching PRs. Example: 'wiiznokes/changen'. Already defined for you in Github Actions
* `--omit-pr-link` — Omit the PR link from the output
* `--omit-thanks` — Omit contributors' acknowledgements/mention
* `--stdout` — Print the result on the standard output
* `--specific <SPECIFIC>` — Generate only this commit, or tag
* `--milestone <MILESTONE>` — Include all commits of this milestone
* `--since <SINCE>` — Include all commits in \"since..until\"
* `--until <UNTIL>` — Include all commits in \"since..until\"



## `changen release`

Generate a new release. By default, use the last tag present in the repo

**Usage:** `changen release [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `-v`, `--version <VERSION>` — Version number for the release. If omitted, use the last tag present in the repo
* `--previous-version <PREVIOUS_VERSION>` — Previous version number. Used for the diff
* `--provider <PROVIDER>` — We use the Github link to produce the tags diff

  Default value: `github`

  Possible values: `github`, `none`

* `--repo <REPO>` — Needed for the tags diff PRs. Example: 'wiiznokes/changen'. Already defined for you in Github Actions
* `--omit-diff` — Omit the commit history between releases
* `--force` — Override the release with the same version if it exist, by replacing all the existing release notes
* `--header <HEADER>` — Add this text as a header of the release. If a header already exist, it will be inserted before the existing one
* `--merge-dev-versions <MERGE_DEV_VERSIONS>` — Merge older dev version into this new release

  Default value: `auto`

  Possible values:
  - `auto`:
    Yes if the version is stable, no otherwise
  - `no`
  - `yes`

* `--stdout` — Print the result on the standard output



## `changen show`

Show a releases on stdout. By default, show the last release

**Usage:** `changen show [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `-n <N>` — -1 being unreleased, 0 the last release, ...

  Default value: `0`
* `-v`, `--version <VERSION>` — Show a specific version. Also accept regex. Example: 1.0.0-*



## `changen remove`

Remove a release

**Usage:** `changen remove [OPTIONS] <-n <N>|--version <VERSION>>`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file

  Default value: `CHANGELOG.md`
* `--stdout` — Print the result on the standard output
* `-n <N>` — -1 being unreleased, 0 the last release, ...
* `-v`, `--version <VERSION>` — Remove a specific version. Also accept regex. Example: 1.0.0-*



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
