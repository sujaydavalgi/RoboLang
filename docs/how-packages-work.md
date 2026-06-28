# How Packages Work

Spanda packages are self-contained projects declared in `spanda.toml`. The runtime loads them in a fixed pipeline before executing your robot program.

## Loading pipeline

```
spanda run robot.sd
    ↓
find project root (walk up to spanda.toml)
    ↓
read spanda.toml manifest
    ↓
resolve dependencies → spanda.lock
    ↓
vendor packages → .spanda/packages/
    ↓
load .sd modules into ModuleRegistry
    ↓
discover official packages from lockfile (registry provenance)
    ↓
bootstrap ProviderRegistry
    ↓
execute program
```

## Package kinds

| Kind | Resolution | On disk |
|------|------------|---------|
| **Local path** | `[dependencies.foo]` with `path = "../foo"` | Referenced in place |
| **Official registry** | `spanda-gps = "0.1"` | `packages/registry/` or remote tarball |
| **Git** | `git = "https://…"` | `.spanda/packages/<name>/` |

## CLI workflow

```bash
spanda init my_robot
spanda add spanda-gps --version 0.1
spanda install          # resolve + lock + vendor
spanda update           # refresh lockfile to latest compatible versions
spanda build            # install (quiet) + compile all sources
spanda run src/main.sd  # loads packages automatically
```

`spanda add` and `spanda remove` edit the manifest only. Run `spanda install` or `spanda update` to refresh `spanda.lock`.

## Official packages

Official packages under `packages/registry/` export dotted module paths (e.g. `positioning.gps`, `communication.mqtt`). The `.sd` exports are thin scaffolds; live behavior is wired through [provider registration](./how-providers-work.md) when the package is **provenanced** in `spanda.lock`:

| Provenance | Built-in providers wire? |
|------------|--------------------------|
| Registry version (`spanda-mqtt = "0.1"`) | Yes |
| Lockfile `registry` source | Yes |
| Path to canonical `packages/registry/<name>` | Yes (monorepo dev) |
| Path/git override of an official name elsewhere | **No** — `.sd` stubs only |

Reusing an official package name with a path or git override emits an `official_provenance` validation warning and excludes the name from provider bootstrap.

See [official-packages.md](./official-packages.md) and [packages.md](./packages.md) for the full catalog.

## Validation

`spanda install` and `spanda build` validate:

- Version compatibility (semver constraints)
- Capability requirements (`[capabilities]`)
- Hardware requirements (`[hardware]`)
- Safety levels (`[safety]`)
- Official package provenance (`official_provenance` warning on name squatting)

Unauthorized or incompatible packages produce actionable diagnostics before runtime. Production rollout should also run `spanda deploy gate --policy production` (see [deployment-gates.md](./deployment-gates.md)).

## Project layout

```
my_robot/
├── spanda.toml
├── spanda.lock
├── src/
│   └── main.sd
├── tests/
└── .spanda/packages/    # vendored dependencies (after install)
```

## See also

- [How Providers Work](./how-providers-work.md)
- [How Runtime Resolution Works](./how-runtime-resolution-works.md)
- [spanda-toml.md](./spanda-toml.md)
