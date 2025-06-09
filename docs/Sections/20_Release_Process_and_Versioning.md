# Section 20: Release Process & Versioning

## 20.1 Semantic Versioning
- **MAJOR** version when you make incompatible API changes
- **MINOR** version when you add functionality in a backwards‐compatible manner
- **PATCH** version when you make backwards‐compatible bug fixes
- Pre‑release identifiers (`-alpha`, `-beta`, `-rc`) for in‐flight work
- Metadata tags (`+build`) for CI build identifiers

---

## 20.2 Release Workflow
1. **Branching Model**
    - `main` (always “green”, production ready)
    - `develop` (integration of new features, stabilized before release)
    - `feature/*`, `bugfix/*`, `hotfix/*` branches off `develop` or `main`
2. **Automated CI/CD**
    - PR builds & tests on `develop` and feature branches
    - `release/*` branches cut from `develop` once ready
    - Merge `release/*` → `main`, tag with version, merge back into `develop`
3. **Artifact Generation**
    - Build and publish `.tar.gz`, `.zip`, platform installers
    - Publish official Docker images (`tlang:MAJOR.MINOR.PATCH`, `tlang:latest`)
    - Generate checksums and digital signatures

---

## 20.3 Changelogs & Release Notes
- **Auto‑generated CHANGELOG.md** via commit messages following Conventional Commits
- **Human‑curated release notes** to highlight:
    - Breaking changes
    - New features
    - Bug fixes
    - Deprecations

---

## 20.4 Nightly & LTS Channels
- **Nightly** builds published on every successful merge to `main` with `-nightly` suffix
- **Long‑Term Support (LTS)** releases maintained for critical bug‐fix backports (e.g., annually)

---

## 20.5 Backwards Compatibility Policy
- Guarantee patch‐level compatibility for one year after release
- Major upgrade guide published alongside breaking changes
- Deprecation warnings surfaced by compiler for one minor release

---

## 20.6 Package Registry & Distribution
- Official **T Registry** for shared libraries and tools
- Support proxying to crates.io, npm registry (for JS frontend), PyPI (bindings), etc.
- Artifact signing and verification (GPG, SHA‑256)

---

## 20.7 Rollback & Hotfixes
- **Hotfix branch** off `main` for critical regressions
- Immediate patch release (`MAJOR.MINOR.(PATCH+1)`)
- Merge hotfix back into both `main` and `develop`

---

## 20.8 Security Releases
- Rapid response pathway for security vulnerabilities (CVE process)
- Coordinated disclosure, advisory publication, and patch tagging
- Automated notifications to users who depend on affected versions

---

## 20.9 Documentation Versioning
- Host docs per version (`docs/vMAJOR.MINOR/`)
- Cross‐link “latest” vs. “stable” vs. “legacy” doc sets

---

## 20.10 Metric & Telemetry Releases
- Optionally include build flags for collecting anonymized usage metrics
- Ensure opt‐in privacy settings and clear documentation of collected data  
