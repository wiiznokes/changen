# Changelog generator

## Features

- feature-rich changelog format
- low-config changelog management
- customizable
- continuous logging, with an unreleased section

## See in action

This project use `changelog-gen` to maintain its changelog, using github action

- [The changelog file](./CHANGELOG.md) - see what the syntax have to offer
- [Its commits history](https://github.com/wiiznokes/changelog-generator/commits/master/CHANGELOG.md)
- [The Github workflow](./.github/workflows/changelog.yml)

## Getting started

_If you don't have a changelog file yet, you can use `changelog-gen new`_

If you already have a changelog file, you can see if its syntax get accepted by running `changelog-gen validate`

> Note: You can always use the --help command to see avaiable options! For example, `changelog-gen validate --fmt` will format your changelog.

When you know your changelog is valid, you can use `changelog-gen generate` to generate a release-note about the last commit.

## Commit syntax

```
fix(project_a): Fix a nasty bug <=> commit-type(scope): commit-message
```

## Advanced use

#### Ignore commit

Currently, you can write theses patterns anyhere in the commit message or desciption:

- `(skip changelog)`
- `(ignore changelog)`
- `!changelog`
- `!log`

#### Map commit type to section(ex: `### Fixed`) in the changelog

The default map can be seen [here](./config_example/config.json). Note than the order will define in witch order the section will appears in the log file.
Use with `changelog-gen generate --map path/to/map.json`

#### Changelog custom path

`changelog-gen generate --file path/to/CHANGELOG.md`

#### Other

A lot of options are avaiable. Use `changelog generate --help` to see them.

## Acknowledgement

- [pom](https://github.com/J-F-Liu/pom) for being an awesome parser. Without this parser, i would have probably drop this project! The [parser](./changelog_document/src/de.rs) of the changelog is less than 200 lines!

- Iced, for its well maintained [changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md)

- [Gitoxide changelog](https://github.com/Byron/gitoxide/blob/main/CHANGELOG.md) because its use a [similar tool](https://github.com/Byron/cargo-smart-release) (quit complex and more powerful)
- [clap](https://github.com/clap-rs/clap)
