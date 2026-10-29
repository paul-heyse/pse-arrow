# Declared GitHub configuration

These files record local desired repository settings. Changes here do not apply settings
on GitHub. The optional setup and label-sync wrappers have retired; an operator can inspect
and apply selected declarations directly with the native GitHub CLI.

| Declaration | Purpose |
| --- | --- |
| `repo.json` | Repository metadata, merge preferences and security settings |
| `topics.json` | Repository topics, including independent scientific references |
| `ruleset-main-full.json` | The declared main-branch ruleset |
| `ruleset-tags.json` | The declared version-tag ruleset |
| `solver-images.json` | Immutable solver-image pins and their local projections |
| [`../labels.yml`](../labels.yml) | Desired label names, colours and descriptions |

Check workflows use manual dispatch. The branch and tag declarations do not introduce
required CI checks. The retained hosted checks cover their named compile, source,
documentation and pure-test scopes; local Linux/user-systemd, selected native solver and
assembled scientific qualification remain with their native owners and Plan58h. No native
release publishing replacement is selected here.

For an explicitly authorized future operation, use `gh auth status` to confirm the account
and repository before choosing a declaration. For example, these commands inspect current
repository and ruleset settings:

```bash
repository='paul-heyse/pse-arrow'
gh api "repos/$repository"
gh api "repos/$repository/rulesets" --jq '.[] | {id, name}'
```

The CLI accepts a selected JSON declaration directly. These examples change remote settings
when deliberately executed:

```bash
gh api --method PATCH "repos/$repository" --input .github/setup/repo.json
gh api --method PUT "repos/$repository/topics" --input .github/setup/topics.json
```

For rulesets, inspect the existing ruleset by its observed ID before choosing `PUT
/repos/{owner}/{repo}/rulesets/{id}`. Use `POST /repos/{owner}/{repo}/rulesets` only for an
explicitly intended new ruleset. Supply the selected ruleset JSON with `--input`. Compare
the returned settings with that declaration; creation, update and read operations remain
distinct choices. See the [GitHub CLI API manual](https://cli.github.com/manual/gh_api).

Labels are individually selectable through `gh label create --force` or `gh label edit`;
the values remain in `../labels.yml`. Removing an obsolete label from that declaration
does not authorize deleting or changing a live label. See the
[label command manual](https://cli.github.com/manual/gh_label).

Solver pins keep their native local owner: `xtask solver-images check`, `ref`, `sync` and
`update`. The corresponding `just solver-pin-check` and `just solver-pin-update` routes are
local checks or explicit pin changes; image publication is a separate manual workflow input.

The former PyPI and TestPyPI environment declarations are retired with the Python product
release workflow. That local retirement neither deletes live GitHub environments nor claims
that remote settings have migrated. Existing remote settings require a separately selected
operation if an operator chooses to change them.
