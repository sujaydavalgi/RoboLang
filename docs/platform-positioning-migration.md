# Platform Positioning — Migration Notes

Spanda's public positioning evolved from **language-first** to **platform-first** while keeping the language identity intact. This guide helps contributors, docs authors, and integrators update copy consistently.

---

## Summary of change

| Aspect | Before | After |
|--------|--------|-------|
| **Primary tagline** | The Autonomous Systems Language | The Autonomous Systems Platform |
| **Full positioning** | Spanda is an AI-native programming language for robotics… | Spanda is an Autonomous Systems Platform with a safety-first programming language at its core |
| **Short alternative** | (same as primary) | The Autonomous Systems Platform |
| **Language role** | Spanda *is* the language | Spanda Language (`.sd`) is **one component** of Spanda Platform |
| **Pulse tagline** | *The pulse of autonomous intelligence.* | **Unchanged** — keep as secondary brand line |

**Do not rename:** the project (`Spanda`), file extension (`.sd`), CLI (`spanda`), or core language docs.

---

## What stays the same

- Project name: **Spanda**
- File extension: **`.sd`**
- Language tutorials: Spanda 101, For Dummies, language reference
- CLI commands and examples
- Philosophy block (body / senses / mind / muscles / pulse)
- Technical terms: `ActionProposal`, `SafeAction`, `spanda verify`, etc.

---

## Messaging hierarchy

Use this order in hero copy and elevator pitches:

1. **Platform** — Autonomous Systems Platform (what Spanda is overall)
2. **Language** — safety-first `.sd` language at the core (how you express systems)
3. **Workflow** — Build · Verify · Simulate · Deploy · Operate
4. **Pulse** — *The pulse of autonomous intelligence* (brand poetry, optional)

### Good

> Spanda is an Autonomous Systems Platform with a safety-first programming language at its core. Write `.sd` programs, verify hardware fit, simulate missions, and operate fleets with built-in health monitoring.

### Avoid

> Spanda is just a programming language. (Understates verify, sim, fleet, packages.)

> Spanda replaced its language with a platform. (False — language is central.)

---

## Document updates

| Location | Action |
|----------|--------|
| `README.md` | Platform intro, Spanda Platform section, Why Spanda?, What makes Spanda different? |
| `docs/platform-overview.md` | Canonical platform vs language reference |
| `docs/roadmap.md` | Organized by platform areas |
| `docs/vision.md`, `docs/product-strategy.md` | Executive positioning aligned to platform |
| `docs/README.md` | Link platform-overview near top |
| `website/index.html`, `docs/website-content.md` | Hero and meta description |
| `CONTRIBUTING.md` | One-line intro |

Language-specific pages (`spanda-language.md`, tutorials) need **no** repositioning unless they incorrectly claim Spanda is *only* a language. Prefer: "Spanda Language is the core of the Spanda Platform."

---

## GitHub repository metadata

Update when you have repo admin access:

**Description (long):**

```
Safety-first autonomous systems platform with a dedicated programming language, verification engine, simulation, replay, and hardware-aware deployment.
```

**Description (short):**

```
Autonomous Systems Platform with a safety-first language at its core.
```

**Topics to add** (merge with existing; do not remove language-related topics):

`autonomous-systems`, `robotics`, `safety`, `simulation`, `verification`, `digital-twin`, `agentic-ai`, `runtime`, `programming-language`, `platform-engineering`, `iot`, `embedded`, `fleet-management`

```bash
gh repo edit Davalgi/Spanda \
  --description "Safety-first autonomous systems platform with a dedicated programming language, verification engine, simulation, replay, and hardware-aware deployment." \
  --add-topic autonomous-systems,robotics,safety,simulation,verification,digital-twin,agentic-ai,runtime,programming-language,platform-engineering,iot,embedded,fleet-management
```

---

## Branding recommendations

### Keep

| Element | Rationale |
|---------|-----------|
| **Spanda** name | Established; no user confusion |
| **`.sd` extension** | Tied to tooling, CI, and examples |
| **Pulse metaphor** | Distinctive; works for platform + language |
| **Sanskrit meaning** in README | Brand depth; optional in technical docs |
| **Safety-Typed AI** | Lead differentiator in all tiers |

### Evolve

| Element | Guidance |
|---------|----------|
| **Hero headline** | Lead with "Autonomous Systems Platform"; mention language in subhead |
| **Diagrams** | Show platform tree (Language → Runtime → Verify → …) before crate-only maps |
| **Release notes** | Frame features by platform area (Verify, Sim, Fleet) not only language syntax |
| **VS Code / LSP** | "Spanda Language support" inside "Spanda Platform" extension branding when published |

### Avoid

- Renaming crates, commands, or file types for "platform" alignment
- Dropping "programming language" from SEO entirely — many users discover Spanda via language search
- Implying ROS2/Python replacement — stay coordination-layer honest per [product-strategy.md](./product-strategy.md)

### Optional future assets

- Platform architecture diagram in `docs/diagrams/` (Mermaid source)
- One-page PDF: Platform vs Language for enterprise evaluators
- `spanda demo` output: print platform tagline in banner

---

## FAQ

**Is Spanda still a programming language?**  
Yes. The Spanda Language (`.sd`) is a first-class product. The platform name reflects verify, sim, replay, health, fleet, and packages that ship alongside the compiler.

**Should I stop saying "Autonomous Systems Language"?**  
Use it when referring specifically to the language (e.g. "learn the Autonomous Systems Language in Spanda 101"). Use "Autonomous Systems Platform" for the overall project.

**Do examples or APIs change?**  
No. This is a positioning and documentation update only.

---

## Changelog

- **2026-06-22** — Initial platform positioning migration (README, platform-overview, roadmap, vision, product-strategy, website drafts).
