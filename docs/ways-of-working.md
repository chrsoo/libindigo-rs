# libidingo Ways of Working

## Github Issue types for software development

GitHub leverages a flexible issue system where "Types" are defined by labels and structural conventions like Task Lists.

### GitHub Issue Classifications


| Icon | Name | Summary |
| :--- | :--- | :--- |
| 📍 | **Tracking Issue** | A parent issue using task lists to manage the progress of an Epic or Feature. |
| ✨ | **Enhancement** | A request for new functionality or improvements (User Stories & Requirements). |
| 🛠️ | **Chore** | Technical tasks, refactoring, or maintenance that doesn't add direct user value. |
| 🐛 | **Bug** | A report of a functional defect, regression, or unintended behavior. |
| 🔍 | **Discussion** | A proposal for architectural changes or RFCs to gather feedback before coding. |

### Hierarchical Relationships

```text
📍 Tracking Issue (Epic/Feature)
└── ✨ Enhancement
    ├── 📄 Task List Item
    └── 📄 Task List Item
└── 🛠️ Chore
└── 🐛 Bug
```

---

### Field Definitions

These properties map to GitHub's native sidebar and metadata fields:

```yaml
# Field Definitions
id: "The unique issue number (e.g., #123)."
created_at: "Date and time the issue was opened."
author: "The GitHub user who created the issue."
labels: "Tags used for classification (e.g., enhancement, chore, priority:high)."
milestone: "The specific release or sprint target (e.g., v1.2.0)."
assignees: "The developers responsible for the implementation."
state: "The current status: open or closed."
```

---

### Issue Type Details

#### 1. Tracking Issue

* **Usage**: High-level coordination. Replaces the "Epic."
* **Format**: Contextual overview followed by a GitHub-flavored markdown Task List.

```markdown
---
id: #1
created_at: 2024-03-05T09:00:00
author: "project-lead"
labels: ["tracking", "priority:medium"]
milestone: "Q1 Launch"
assignees: ["manager-user"]
state: "open"
---

# Tracking: [Feature] Advanced Search Filters

## Description
Centralized tracking for implementing multi-parameter search capabilities.

## Task List
- [ ] #10 ✨ Enhancement: Date Range Picker
- [ ] #11 ✨ Enhancement: Filter by Category
- [ ] #15 🛠️ Chore: Index database search columns
```

#### 2. Enhancement

* **Usage**: The primary unit of value. Covers both user-facing stories and technical requirements.
* **Format**: Focuses on the "Proposed Solution" and "Acceptance Criteria."

```markdown
---
id: #10
created_at: 2024-03-05T10:30:00
author: "dev-alpha"
labels: ["enhancement", "size:medium"]
milestone: "Q1 Launch"
assignees: ["frontend-dev"]
state: "open"
---

# Enhancement: Date Range Picker for Search

## User Story / Requirement
The system should allow users to filter results between two specific dates to narrow down search results.

## Acceptance Criteria
- [ ] User can select a 'start' and 'end' date.
- [ ] UI handles invalid date ranges gracefully.
```

#### 3. Chore

* **Usage**: Internal tasks like dependency updates, CI/CD tweaks, or refactoring.
* **Format**: Explicit technical steps without the need for user-value justification.

```markdown
---
id: #15
created_at: 2024-03-05T11:45:00
author: "backend-lead"
labels: ["chore", "area:database"]
milestone: "Infrastructure"
assignees: ["db-admin"]
state: "open"
---

# Chore: Index Database Search Columns

## Technical Details
Add B-Tree indexes to the `created_at` and `category_id` columns in the `search_results` table.

## Verification
- [ ] Explain plan shows index usage on production-sized datasets.
```

#### 4. Bug

* **Usage**: Defect tracking.
* **Format**: Steps to Reproduce, Expected vs. Actual, and Environment.

```markdown
---
id: #22
created_at: 2024-03-05T12:00:00
author: "tester-pro"
labels: ["bug", "priority:critical"]
milestone: "Hotfix"
assignees: ["dev-alpha"]
state: "open"
---

# Bug: Search crash on empty query

## Steps to Reproduce
1. Navigate to /search.
2. Hit 'Enter' without typing.

## Observed Behavior
The application throws a 500 Internal Server Error.
```

## Github YAML templates

To use these in a repository, save these as individual `.yml` files in your `.github/ISSUE_TEMPLATE/` directory.

```yaml
# enhancement.yml
name: ✨ Enhancement
description: Propose a new feature or technical requirement.
labels: ["enhancement"]
body:
  - type: textarea
    id: goal
    attributes:
      label: Goal
      description: What is the desired outcome?
    validations:
      required: true
  - type: textarea
    id: ac
    attributes:
      label: Acceptance Criteria
      placeholder: "- [ ] Must do X..."
```

```yaml
# chore.yml
name: 🛠️ Chore
description: Internal maintenance or technical tasks.
labels: ["chore"]
body:
  - type: textarea
    id: details
    attributes:
      label: Technical Details
      description: Describe the technical work to be performed.
    validations:
      required: true
```

```yaml
# bug.yml
name: 🐛 Bug
description: Report a defect or error.
labels: ["bug"]
body:
  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      placeholder: "1. Open app..."
    validations:
      required: true
```

## GitHub Actions: Automated Project Label Setup

To use this, create a file in your repository at `.github/workflows/setup-labels.yml`. Once committed, you can run this manually from the **Actions** tab to instantly synchronize your repository labels with the "Tracking", "Enhancement", and "Chore" system defined in your documentation.

```yaml
name: "Project Infrastructure: Setup Labels"

on:
  workflow_dispatch: # Allows manual execution from the GitHub Actions UI

jobs:
  labeler:
    runs-on: ubuntu-latest
    steps:
      - name: Create Project Labels
        uses: actions/github-script@v7
        with:
          script: |
            const labels = [
              {
                name: "tracking",
                color: "5319e7",
                description: "High-level Epic/Feature tracking issue"
              },
              {
                name: "enhancement",
                color: "a2eeef",
                description: "New functionality or requirement (User Story)"
              },
              {
                name: "chore",
                color: "fbca04",
                description: "Internal technical tasks and maintenance"
              },
              {
                name: "bug",
                color: "d73a4a",
                description: "Functional defect or error"
              },
              {
                name: "priority:high",
                color: "b60205",
                description: "Urgent items requiring immediate attention"
              }
            ];

            for (const label of labels) {
              try {
                // Check if label exists, if not, create it
                await github.rest.issues.createLabel({
                  owner: context.repo.owner,
                  repo: context.repo.repo,
                  name: label.name,
                  color: label.color,
                  description: label.description
                });
                console.log(`Created label: ${label.name}`);
              } catch (error) {
                if (error.status === 422) {
                  console.log(`Label "${label.name}" already exists, skipping.`);
                } else {
                  console.error(`Error creating "${label.name}": ${error.message}`);
                }
              }
            }
```

### Usage Note

- **Color Codes**: These utilize the standard GitHub hex palette for visual consistency.
* **Manual Trigger**: Go to your repo -> **Actions** -> **Setup Project Labels** -> **Run workflow**.
* **Permissions**: This action requires `issues: write` permissions, which is standard for the default `GITHUB_TOKEN`.
