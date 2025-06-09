# Section 21: Governance & Community

## 21.1 Project Governance Model
- **BDFL‑Lite / Steering Council**
    - A small group of elected maintainers (“Core Team”) with final merge authority
    - Advisory Council of experienced contributors for design reviews
- **Term & Rotation**
    - Core Team members serve 1‑year terms, renewable by community vote
    - Advisory seats are perpetual but can be rotated on request

---

## 21.2 Code of Conduct
- **Adopted Standard**: Contributor Covenant v2.1
- **Enforcement**
    - Dedicated team of “CoC Guardians” to handle reports
    - Confidential reporting channel and timely resolution
- **Zero tolerance** for harassment, hate speech, or discriminatory behavior

---

## 21.3 Contribution Workflow
1. **Issue Triage**
    - Labeling (“bug”, “feature request”, “discussion”)
    - Assign to milestone or close as “out of scope”
2. **Pull Request Guidelines**
    - Follow the **Conventional Commits** style
    - Include tests and documentation updates
    - Link to related issue(s)
3. **Review Process**
    - At least two approving reviews from distinct Core Team members
    - Automated CI must pass (build, lint, tests)
    - Merge via GitHub “squash and merge” to keep history clean

---

## 21.4 Release Team & Roles
- **Release Manager**: coordinates cut of `release/*`, tagging, and publication
- **Docs Lead**: prepares release notes, updates website and API docs
- **Security Lead**: oversees CVE triage and advisories

---

## 21.5 Decision‑Making Guidelines
- **RFC Process**
    - All major proposals must be submitted as an RFC in `tlang/rfcs/`
    - 2‑week discussion period followed by Core Team vote
- **Consensus vs. Voting**
    - Aim for consensus; if not reached, Core Team majority decides
- **Fast‑track Exceptions**
    - Critical security fixes or urgent hotfixes may bypass RFC

---

## 21.6 Community Communication Channels
- **Mailing List**: announcement and design discussion
- **Chat**: real‑time support and casual questions (e.g. Discord, Matrix)
- **Office Hours**: weekly video calls for newcomers & deep dives
- **Forum**: long‑form design threads, tutorials, and Q&A

---

## 21.7 Issue & PR Etiquette
- **Search before opening** to avoid duplicates
- **Fill out templates** accurately (bug report, feature request)
- **Be patient** and responsive to review comments

---

## 21.8 Onboarding & Mentorship
- **Good First Issues** and **Help Wanted** labels
- Pair‑programming sessions with mentors
- “Hello, World!” tutorials and guided walkthroughs

---

## 21.9 Recognition & Rewards
- **Contributor Awards**: public “Hall of Fame” on website
- **GitHub Sponsor** program for sustained contributors
- **Swag** (stickers, T‑shirts) for major milestones and top contributors

---

## 21.10 Handling Forks & Derivatives
- Encourage ecosystem growth via official **Plugin Registry**
- Guidelines for maintaining alignment with core project
- Trademark policy for use of "T" and “T‑Lang” names
