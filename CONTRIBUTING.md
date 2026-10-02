# Contributing to Functors

Functors is in early development. This document defines the contribution workflow,
branching model, merge strategy, and coding standards. It will evolve as the
project grows.

---

# 1. Branching Model

Functors uses a disciplined branching strategy inspired by GitFlow, adapted for
solo and multi‑maintainer development.

## Protected Branches

### **main**
- Stable branch  
- Always compiles  
- Contains tagged releases  
- Direct pushes are **not allowed**  
- All changes must come through PRs

### **develop**
- Integration branch  
- Receives completed features  
- Direct pushes are **not allowed**  
- All changes must come through PRs

---

# 2. Feature Branches

Major features and milestones are developed in dedicated **feature/*** branches.

Examples:
- `feature/mvu-core`
- `feature/rendering-engine`
- `feature/text-buffer`
- `feature/notebook-model`
- `feature/agent-protocol`

Rules:
- Feature branches are long‑lived during a milestone  
- They represent the *clean* version of the milestone  
- They merge into `develop` via PR  
- They should not contain experimental or messy commits

Feature branches act as the “official” home of a milestone.

---

# 3. Personal Working Branches

All day‑to‑day development happens in personal branches derived from `develop`.

Examples:
- `pp/mvu-core`
- `pp/rendering-engine`
- `pp/text-buffer`

Rules:
- You may commit freely  
- You may rebase, amend, or force‑push  
- These branches are not protected  
- These branches are used to open PRs into `feature/*` or directly into `develop`

Typical workflow:

develop → pp/mvu-core → PR → feature/mvu-core → PR → develop

For solo development, this may simplify to:

develop → pp/mvu-core → PR → develop

---

# 4. Pull Request Workflow

## PR Targets
- Personal working branches → `develop`  
- Feature branches → `develop`  
- `develop` → `main` (release PR)

## PR Requirements
- PR must compile  
- PR must include documentation updates if relevant  
- PR must follow the merge strategy below  
- PR must be approved (self‑approval allowed for solo maintainers)

---

# 5. Merge Strategy

### **Squash and merge** (for PRs into `develop`)
Use squash for:
- personal working branches → develop  
- feature branches → develop  

This keeps `develop` clean and milestone‑oriented.

### **Merge commit** (for develop → main)
Use a normal merge commit for:
- releases  
- version bumps  
- stable milestones  

This preserves the full milestone history.

### **Rebase**
Allowed **only** on personal working branches.  
Never rebase `main`, `develop`, or `feature/*` after pushing.

---

# 6. Commit Message Conventions

Functors uses **Conventional Commits**:

- `feat:` new feature  
- `fix:` bug fix  
- `docs:` documentation changes  
- `refactor:` internal changes  
- `test:` tests  
- `chore:` maintenance  

Examples:
- `feat(mvu): add Model and Message definitions`
- `docs: update MVU architecture chapter`
- `refactor(render): simplify GPU pipeline init`

---

# 7. Coding Standards

- Rust 2024 edition  
- `rustfmt` must pass  
- `clippy` must pass  
- Prefer small, composable modules  
- MVU code must remain pure and deterministic  
- Rendering code must avoid blocking operations  
- Platform code must isolate OS‑specific logic

---

# 8. Documentation Requirements

Update the documentation owner for the subsystem being changed. Update the
mdBook or API documentation when the change affects that document's audience;
keep internal engineering decisions in contributor references and ADRs.

### **mdBook (`docs/book/`)**
- end-user workflows and concepts
- project capabilities and user-facing architecture
- user-relevant roadmap or platform information

### **API Docs (`docs/api/`)**
- crate‑level documentation  
- module‑level documentation  
- public API explanations

### **Contributor References (`docs/`)**
- development and testing workflow
- module ownership and Rust practices
- internal architecture decisions
- future documentation triggers

Documentation is part of the definition of done.

---

# 9. Testing

Phase 1:
- basic unit tests for MVU core  
- basic rendering initialization tests  

Later phases will introduce:
- integration tests  
- snapshot tests  
- WASM/WASI runtime tests  
- agent protocol tests  

---

# 10. Code of Conduct

All contributors must follow the project's Code of Conduct.

---

This document will expand as Functors grows.
