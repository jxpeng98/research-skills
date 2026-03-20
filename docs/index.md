---
layout: home

hero:
  name: Research Skills
  text: Contract-Driven Academic Workflow Docs
  tagline: A consolidated documentation hub for installation, CLI usage, workflow entrypoints, architecture, MCP integrations, and maintainer guidance.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/
    - theme: alt
      text: CLI Reference
      link: /reference/cli
    - theme: alt
      text: Architecture
      link: /architecture

features:
  - title: Operator Path
    details: Start with quickstart, installation, upgrade, troubleshooting, and the shell/Python CLI flows.
  - title: Contract First
    details: Understand how standards, capability routing, roles, skills, pipelines, and bridges fit together.
  - title: Advanced Integrations
    details: Configure MCP providers, Zotero workflows, and multi-agent collaboration patterns without hunting across repo files.
  - title: Maintainer Ready
    details: Find conventions, CLAUDE-oriented maintainer guidance, and release publishing docs in one place.
---

## Documentation Map

- [Guide](/guide/): start here for install, upgrade, troubleshooting, and quickstart.
- [Task Recipes](/guide/task-recipes): choose stages and skills from real user goals such as review, writing, code, or rebuttal.
- [Examples](/examples/): follow concrete paper-type playbooks for systematic review, empirical, methods, and theory papers.
- [Reference](/reference/): command reference, skills guide, conventions, and stable operator-facing entrypoints.
- [Architecture](/architecture): layer model, dependency direction, discipline domains, and runtime collaboration.
- [Advanced](/advanced/): MCP providers, Zotero, skill-agent collaboration, and extension guides.
- [Maintainer](/maintainer/): CLAUDE-aligned implementation guidance and release operations.

## Source Coverage

This site consolidates material previously spread across:

- `README.md`
- `README_CN.md`
- `docs/`
- `guides/basic/`
- `guides/advanced/`
- `CLAUDE.md`

The original markdown files remain in the repository. The VitePress site reorganizes them around user tasks instead of repository folders.
