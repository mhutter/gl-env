# `gl-env`

Bulk-edit Group- and Project-level CI/CD variables in GitLab.

This tools allows you to define your desired variables in a YAML file, and then diff or apply them automatically.

Features

- List variables
- Dump existing variables to YAML
- Diff between desired and current values
- Apply variables from local file
- Support for different environment scopes

## Installation

```sh
cargo install gl-env
```

(OS packages & GitHub releases soon?)

## Usage

Start off by dumping existing variables into a YAML-File

```sh
gl-env dump -p mygroup/myproject > myproject.yml
```

Alternatively, create a file from scratch:

```yaml
# Default configuration to apply to all variables.
#
# The same attributes can be applied to individual variables.
defaults:
  # Whether to mask the value in job logs.
  # NOTE: masked variable values must
  # - be a single line with no spaces
  # - be at least 8 characters long
  # - not match the name of an existing or predefined CI/CD variable
  # - not include special characters other than `@`, `_`, `-`, `:`, `+`
  # Default: `false`
  masked: false
  # Export variable to pipelines running on protected branches and tags only.
  # Default: `false`
  protected: false
  # Don't expand references to other variables.
  # `$` will be treated as the start of a reference to another variable.
  # Default: `false`
  raw: false

# Variables valid for ALL environments (`*`)
variables:
  KEY:
    value: The value!
    raw: true
  ANOTHER:
    value: some-val

# Variables only valid for specific environments
environment:
  test:
    MYSERVICE_KEY:
      value: test-environment-api-key
  prod:
    MYSERVICE_KEY:
      value: production-environment-api-key
```

You can then use the `diff` and `apply` subcommands:

```sh
gl-env diff -p mygroup/myproject < myproject.yml

# edit myproject.yml

gl-env apply -p mygroup/myproject < myproject.yml
```

See `gl-env help` to get complete usage information.

## Backlog

- [x] Support group-level variables or instance-level variables
- [ ] Non-ugly error messages
- [ ] String-or-struct for variables (?)
- [x] Remove obsolete variables
- [x] Default settings for all variables
- [x] diff
- [x] apply
- [x] Support identically-named variables in different environments
- [x] Ensure output order of variables is stable
- [x] apply: List variables that are defined in GitLab but not locally

## License

Licensed under the [MIT license](LICENSE).
