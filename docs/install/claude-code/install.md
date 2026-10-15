# Installation

tightship depends on three companion plugins, each listed in its own catalog. Add those three catalogs first,
then tightship's, then install once:

```sh
claude plugin marketplace add DietrichGebert/ponytail
claude plugin marketplace add ayghri/i-have-adhd
claude plugin marketplace add blader/humanizer
claude plugin marketplace add https://github.com/ATNexusLab/tightship/releases/latest/download/marketplace.json
claude plugin install tightship@tightship
```

The install adds the companions along with the plugin:

```text
✔ Successfully installed plugin: tightship@tightship (scope: user) (+ 3 dependencies: ponytail, i-have-adhd, humanizer)
```

If a companion's catalog is missing, Claude Code warns that the dependency was not installed, and tightship
does not load. Add the missing catalog and run the install again.

The install asks for no option from the shell. Set them as you install with `--config`, or in the next
session's dialog: see [the options](options.md).

## Check it

Run `claude plugin list`. `tightship@tightship` shows with no error, and so do `ponytail`, `i-have-adhd`, and
`humanizer`.
