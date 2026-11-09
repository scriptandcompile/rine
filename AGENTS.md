# AGENTS.md — rine

## Agent skills

### Coding standards

Documented coding standards live in `docs/standards.md`. The code-review skill's
Standards axis checks diffs against that file. The rule everything else hangs
off: the Win32 interface is the contract a guest app sees, and it must be
honored exactly; the implementation under that interface is safe, modern,
bitness-neutral Rust.

### Issue tracker

Issues and specs live in GitHub Issues at `scriptandcompile/rine`, via the `gh`
CLI. `roadmap.md` at the repo root is the index of open tracks toward the
oracle objective; a picked-up track gets a GitHub issue with its design, and
done tracks are removed from the matrix. Check the roadmap's status matrix
before proposing new coverage work — coverage picks are steered by it.

### Triage labels

The triage label vocabulary is `needs-triage`, `needs-info`,
`ready-for-agent`, `ready-for-human`, `wontfix`. If the five labels do not
exist on GitHub yet, create them:
`gh label create <name> --repo scriptandcompile/rine --description "..."`.

### Domain docs

`GLOSSARY.md` at the repo root holds the domain vocabulary (device context,
paint session, window class, backend adapter, control, export maturity, …).
Create it lazily when a concept gets named; keep doc comments, log messages,
and error text on those exact terms. ADRs live in `docs/adr/` (none yet).

### Commits

Match the existing history: plain English, lowercase subject, a past-tense
verb doing a plain verb to a plain object — `added <thing>`, `fixed <thing>`,
`made <thing> async so that <why>`. Name the crate or area when plain words
need the precision. Add a body only when it says why the change exists; the
body must be readable by someone who has never read the glossary. Reference
the issue number where one exists.

- Before: `refactor: deepen DC module`
- After: `gave every device context a real surface, so GDI drawing reaches the screen (issue #NN)`

### Reply style

When reporting work to a human in conversation:

- Lead with what the change gives a person, in complete sentences.
- Cite file:line and issue numbers instead of coined abstractions.
- One sentence, one claim. No aphoristic closers. End on the fact.
- Name an actor and a verb in every sentence; the first sentence of any
  paragraph must be readable without the glossary.

### Build & test entry points

- `./scripts/build-rine.sh` — build the loader with dev features.
- `./scripts/build-integration-prereqs.sh` — build the MinGW C fixtures.
- `cargo test` — unit + integration; fixtures run for both arches via
  `RINE_FIXTURE_ARCH=x64|x86`.
- `./scripts/full_build.sh` — CI-style build, includes the attribute-metadata
  validator that enforces the export doc/tag format.
