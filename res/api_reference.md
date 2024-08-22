# Command-Line Help for `changelog`

This document contains the help content for the `changelog` command-line program.

**Command Overview:**

* [`changelog`↴](#changelog)
* [`changelog generate`↴](#changelog-generate)
* [`changelog release`↴](#changelog-release)
* [`changelog validate`↴](#changelog-validate)
* [`changelog show`↴](#changelog-show)
* [`changelog new`↴](#changelog-new)

## `changelog`

Changelog generator

**Usage:** `changelog <COMMAND>`

###### **Subcommands:**

* `generate` — Generate release notes. Default to `last_release_in_changelog..HEAD`
* `release` — Generate a new release
* `validate` — Validate a changelog syntax
* `show` — Show a specific release on stdout
* `new` — Create a new changelog file with an accepted syntax



## `changelog generate`

Generate release notes. Default to `last_release_in_changelog..HEAD`

**Usage:** `changelog generate [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file.

  Default value: `CHANGELOG.md`
* `--map <MAP>` — Path to the commit type to changelog section map.
* `--parsing <PARSING>` — Parsing of the commit message.

  Default value: `smart`

  Possible values: `smart`, `strict`

* `--exclude-unidentified` — Don't include unidentified commits.
* `--exclude-not-pr` — Don't include commits which are not attached to a pull request.
* `--provider <PROVIDER>` — We use the Github api to map commit sha to PRs.

  Default value: `github`

  Possible values: `github`, `other`

* `--repo <REPO>` — Needed for fetching PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions.
* `--omit-pr-link` — Omit the PR link from the output.
* `--omit-thanks` — Omit contributors' acknowledgements/mention.
* `--stdout` — Print the result on the standard output.
* `--specific <SPECIFIC>` — Generate only this commit, or tag.
* `--milestone <MILESTONE>` — Include all commits of this milestone
* `--since <SINCE>` — Include all commits in "since..until".
* `--until <UNTIL>` — Include all commits in "since..until".



## `changelog release`

Generate a new release

**Usage:** `changelog release [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file.

  Default value: `CHANGELOG.md`
* `-v`, `--version <VERSION>` — Version number for the release. If omitted, use the last tag using "git".
* `--previous-version <PREVIOUS_VERSION>` — Previous version number. Used for the diff.
* `--provider <PROVIDER>` — We use the Github link to produce the tags diff

  Default value: `github`

  Possible values: `github`, `other`

* `--repo <REPO>` — Needed for the tags diff PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions.
* `--omit-diff` — Omit the commit history between releases.
* `--stdout` — Print the result on the standard output.



## `changelog validate`

Validate a changelog syntax

**Usage:** `changelog validate [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file.

  Default value: `CHANGELOG.md`
* `--format` — Format the changelog.
* `--map <MAP>` — Path to the commit type to changelog section map.
* `--ast` — Show the Abstract Syntax Tree.
* `--stdout` — Print the result on the standard output.



## `changelog show`

Show a specific release on stdout

**Usage:** `changelog show [OPTIONS]`

###### **Options:**

* `-f`, `--file <FILE>` — Path to the changelog file.

  Default value: `CHANGELOG.md`
* `-n <N>` — 0 being unreleased, 1 is the last release

  Default value: `1`
* `-v`, `--version <VERSION>` — Specific version.



## `changelog new`

Create a new changelog file with an accepted syntax

**Usage:** `changelog new [OPTIONS]`

###### **Options:**

* `-p`, `--path <PATH>` — Path to the changelog file.

  Default value: `CHANGELOG.md`
* `-f`, `--force` — Override of existing file.



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
