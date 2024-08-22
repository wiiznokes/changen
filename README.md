# Changelog generator

## Features

- feature-rich changelog format
- low-config changelog management
- customizable

## See in action

This project use `changelog-gen` to maintain its changelog, using github action

- [The changelog file](./CHANGELOG.md) - see what the syntax have to offer
- [Its commits history](https://github.com/wiiznokes/changelog-generator/commits/master/CHANGELOG.md)
- [The release Github workflow](./.github/workflows/create_release_notes_pr.yml) - It will create a PR

## Getting started

1. **Create the changelog**

   If you don't have a changelog file yet, you can use `changelog-gen new`.

2. **Validate your changelog syntax**

   If you already have a changelog file, you can see if its syntax get accepted by running `changelog-gen validate`.

3. **Generate release notes**

   When you know your changelog is valid, you can use `changelog-gen generate` to generate a release-note about the last commit.

   It can generate release notes

   - between two tags/commits
   - for a specific commit/tag
   - for a milestone

   By default, it will generate release notes from the last release in the changelog to HEAD. It will get the list of commits using a `git log` command, and try to match them against remote PRs if it have the necessary infos.

4. **Make a new release**

   To make a new release, use `changelog-gen release --version 1.0.0`.

> [!WARNING]  
> _All_ tags of the repo and versions in the changelog _must_ follow the [semver](https://semver.org/) format, and should match each other.

**The full API reference can be found [here](./res/API_REFERENCE.md)** (automatically generated).

## Commit syntax

```
fix(project_a): Fix a nasty bug <=> commit-type(scope): commit-message
```

## Advanced use

#### Ignore commit

Currently, you can write theses patterns anywhere in the commit message or description:

- `(skip changelog)`
- `(ignore changelog)`
- `!changelog`
- `!log`

Note that any commit modifying your changelog will be ignored

#### Map commit type to section(ex: `### Fixed`) in the changelog

The default map can be seen [here](./res/map_commit_type_to_section.json). Note than the order will define in witch order the section will appears in the log file.
Use with `changelog-gen generate --map path/to/map.json`

#### Changelog custom path

`changelog-gen generate --file path/to/CHANGELOG.md`

## Acknowledgement

- [pom](https://github.com/J-F-Liu/pom) for being an awesome parser. Without this parser, i would have probably drop this project! The [parser](./changelog_document/src/de.rs) of the changelog is less than 200 lines!

- Iced, for its well maintained [changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md)

- [Gitoxide changelog](https://github.com/Byron/gitoxide/blob/main/CHANGELOG.md) because its use a [similar tool](https://github.com/Byron/cargo-smart-release) (quit complex and more powerful)
- [clap](https://github.com/clap-rs/clap)
