# GitHub Issues Guide

This document provides guidance on creating GitHub issues from the planning documents in the `plans/` directory. Issues should focus on **what** needs to be achieved (goals, acceptance criteria) while plans provide **how** (implementation details, architecture).

## Relationship Between Plans and Issues

- **Plans** (`plans/` directory): Technical specifications, architecture designs, implementation details
- **Issues** (GitHub): Work items, acceptance criteria, progress tracking, discussion

**Key Principle**: Issues reference plans for details, avoiding content duplication.

## Active Plans Requiring Issues

### v0.2.0 Release

#### 1. Property Constants Extraction

**Plan**: [`plans/indigo-constants-extraction.md`](../plans/indigo-constants-extraction.md)

**Suggested Issues**:

**Issue: Automate props.rs Generation from INDIGO Headers**

- Type: 🛠️ Chore
- Labels: `area:core`, `size:medium`, `priority:high`
- Milestone: v0.2.0
- Description:

  ```
  Automate generation of props.rs from INDIGO C headers as part of build process.

  See plans/indigo-constants-extraction.md for implementation details.

  Acceptance Criteria:
  - [ ] Build script extracts property names from INDIGO headers
  - [ ] Generated props.rs matches current manual version
  - [ ] props.rs kept under version control
  - [ ] Documentation explains regeneration process
  - [ ] Tests verify generated constants match INDIGO
  ```

**Issue: Replace Hardcoded Property Strings with Constants**

- Type: 🛠️ Chore
- Labels: `area:core`, `area:protocol`, `size:large`, `priority:high`
- Milestone: v0.2.0
- Depends on: Property constants generation
- Description:

  ```
  Replace all hardcoded property name strings with references to generated constants.

  See plans/indigo-constants-extraction.md Phase 2 for details.

  Acceptance Criteria:
  - [ ] No hardcoded strings in src/strategies/rs/
  - [ ] No hardcoded strings in src/strategies/ffi.rs
  - [ ] All tests pass with new constants
  - [ ] Type safety improved through constant usage
  ```

#### 2. Documentation Organization

**Plan**: [`plans/documentation-organization.md`](../plans/documentation-organization.md)

**Issue: Extract Completed Documentation from plans/ to doc/**

- Type: 🛠️ Chore
- Labels: `area:docs`, `size:medium`, `priority:medium`
- Milestone: v0.2.0
- Description:

  ```
  Organize documentation into clear categories and extract completed docs from plans/.

  See plans/documentation-organization.md for structure.

  Acceptance Criteria:
  - [ ] Architecture docs in doc/architecture/
  - [ ] Protocol docs in doc/protocols/
  - [ ] Development docs in doc/development/
  - [ ] Comprehensive doc/README.md index
  - [ ] Plans/ contains only active planning documents
  ```

### v0.3.0 Release

#### 3. Trait-Based Device API

**Plan**: [`plans/trait-based-device-api-v3.md`](../plans/trait-based-device-api-v3.md)

**Tracking Issue: Implement Trait-Based Device API**

- Type: 📍 Tracking Issue
- Labels: `tracking`, `area:core`, `priority:high`
- Milestone: v0.3.0
- Description:

  ```
  Implement high-level, trait-based API for common INDIGO device types.

  See plans/trait-based-device-api-v3.md for complete architecture.

  Prerequisites:
  - Property extraction work (v0.2.0) must be complete

  Sub-issues:
  - [ ] #TBD Foundation: Core Device trait and property wrappers
  - [ ] #TBD Camera trait implementation
  - [ ] #TBD Mount trait implementation
  - [ ] #TBD Focuser & FilterWheel traits
  - [ ] #TBD Additional devices (Dome, GPS, Guider, AO, Rotator, Aux)
  - [ ] #TBD Integration & polish
  ```

### Infrastructure & CI/CD

#### 4. CI/CD Strategy

**Plan**: [`plans/ci-cd-strategy.md`](../plans/ci-cd-strategy.md)

**Issue: Implement CI/CD Pipeline per Strategy Document**

- Type: 🛠️ Chore
- Labels: `area:testing`, `size:medium`, `priority:medium`
- Description:

  ```
  Implement comprehensive CI/CD pipeline as documented in plans/ci-cd-strategy.md.

  Acceptance Criteria:
  - [ ] Fast feedback job (Pure Rust, 3-5 min)
  - [ ] Comprehensive job (FFI, 15-30 min)
  - [ ] Caching strategy implemented
  - [ ] Graceful test degradation when server unavailable
  ```

#### 5. Integration Test Harness

**Plan**: [`plans/integration_test_harness_architecture.md`](../plans/integration_test_harness_architecture.md)

**Tracking Issue: Implement Integration Test Harness**

- Type: 📍 Tracking Issue
- Labels: `tracking`, `area:testing`, `priority:medium`
- Description:

  ```
  Implement test harness for managing live INDIGO server during integration tests.

  See plans/integration_test_harness_architecture.md for complete design.

  Sub-issues:
  - [ ] #TBD Core infrastructure (ServerManager, HealthMonitor)
  - [ ] #TBD State management between tests
  - [ ] #TBD Test integration and migration
  - [ ] #TBD Documentation and examples
  ```

### Completed Work (Reference Only)

These plans document completed work and should be moved to `plans/archive/`:

- ✅ [`plans/discovery-implementation.md`](../plans/discovery-implementation.md) - ZeroConf discovery (Phases 1 & 2 complete)
- ✅ [`plans/json-protocol-implementation.md`](../plans/json-protocol-implementation.md) - JSON protocol complete
- ✅ [`plans/integration-test-server-config.md`](../plans/integration-test-server-config.md) - Server config complete

**Action**: Move these to `plans/archive/` after creating reference documentation in `doc/`.

### Architecture Plans (Keep in plans/)

These are reference architecture documents that should remain in `plans/`:

- [`plans/code-review-and-architecture.md`](../plans/code-review-and-architecture.md) - Overall architecture review
- [`plans/crate-restructuring-architecture-v3.md`](../plans/crate-restructuring-architecture-v3.md) - Crate structure (if needed)
- [`plans/zeroconf_discovery_architecture.md`](../plans/zeroconf_discovery_architecture.md) - Discovery architecture

### Issue Tracking Plans (Keep in plans/)

- [`plans/issues.md`](../plans/issues.md) - Known issues and structural concerns
- [`plans/immediate-ci-fix.md`](../plans/immediate-ci-fix.md) - CI/CD fixes (may be obsolete)

## Issue Creation Workflow

### 1. For Tracking Issues (Epics)

```markdown
**Title**: [Feature Name]

**Type**: 📍 Tracking Issue

**Labels**: `tracking`, `area:*`, `priority:*`

**Milestone**: v0.X.0

**Description**:
Brief overview of the feature/epic.

See plans/[plan-name].md for complete architecture and implementation details.

**Prerequisites**:
- List any dependencies

**Sub-issues**:
- [ ] #TBD Sub-issue 1
- [ ] #TBD Sub-issue 2
...

**Success Criteria**:
- [ ] High-level acceptance criteria
```

### 2. For Enhancement Issues

```markdown
**Title**: [Specific Enhancement]

**Type**: ✨ Enhancement

**Labels**: `area:*`, `size:*`, `priority:*`

**Milestone**: v0.X.0

**Parent**: #[tracking-issue] (if applicable)

**Description**:
What needs to be implemented.

See plans/[plan-name].md section [X] for implementation details.

**Acceptance Criteria**:
- [ ] Specific, testable criteria
- [ ] ...

**Implementation Notes**:
Reference specific sections of the plan document.
```

### 3. For Chore Issues

```markdown
**Title**: [Maintenance Task]

**Type**: 🛠️ Chore

**Labels**: `area:*`, `size:*`, `priority:*`

**Description**:
What needs to be done and why.

See plans/[plan-name].md for details.

**Acceptance Criteria**:
- [ ] Specific outcomes
```

## Best Practices

### DO

- ✅ Reference plan documents for implementation details
- ✅ Focus on acceptance criteria in issues
- ✅ Use tracking issues for multi-phase work
- ✅ Link related issues and plans
- ✅ Keep issue descriptions concise
- ✅ Update issue status as work progresses

### DON'T

- ❌ Duplicate implementation details from plans in issues
- ❌ Create issues for already-completed work
- ❌ Mix multiple unrelated tasks in one issue
- ❌ Forget to assign milestones
- ❌ Leave issues without acceptance criteria

## Milestone Planning

### v0.2.0 - Property Extraction

- Property constants generation
- Hardcoded string replacement
- Documentation organization

### v0.3.0 - High-Level Device API

- Trait-based device abstractions
- Property wrappers
- Device-specific methods

### v1.0.0 - Production Ready

- Complete test coverage
- Full documentation
- Performance optimization
- Stability and polish

## Labels Reference

**Issue Types**:

- `tracking` - Epic/Feature tracking issue
- `enhancement` - New functionality
- `chore` - Internal maintenance
- `bug` - Defect
- `discussion` - Architectural proposal

**Priority**:

- `priority:high` - Urgent, blocking
- `priority:medium` - Important
- `priority:low` - Nice to have

**Size**:

- `size:small` - < 1 day
- `size:medium` - 1-3 days
- `size:large` - > 3 days

**Area**:

- `area:core` - Core library
- `area:ffi` - FFI bindings
- `area:protocol` - Protocol implementation
- `area:discovery` - Server discovery
- `area:testing` - Testing infrastructure
- `area:docs` - Documentation

## Next Steps

1. **Review this guide** and the referenced plan documents
2. **Run GitHub Actions workflow** to create labels (see doc/github-setup-instructions.md)
3. **Create tracking issues** for v0.2.0 and v0.3.0 work
4. **Break down tracking issues** into specific enhancement/chore issues
5. **Assign milestones** and priorities
6. **Begin implementation** following the Roo workflow scheme (doc/roo-workflow-scheme.md)

## References

- [Ways of Working](ways-of-working.md) - GitHub issue types and processes
- [Roo Workflow Scheme](roo-workflow-scheme.md) - AI persona workflow
- [GitHub Setup Instructions](github-setup-instructions.md) - Initial setup
- [CHANGES.md](../CHANGES.md) - Feature backlog and changelog
