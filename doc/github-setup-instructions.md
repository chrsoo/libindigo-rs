# GitHub Setup Instructions

This document provides step-by-step instructions for setting up the GitHub repository with issue templates, labels, and workflows.

## Prerequisites

- Repository must be pushed to GitHub
- You must have admin access to the repository
- GitHub Actions must be enabled for the repository

## Step 1: Verify Files Are Pushed

Ensure all the following files are committed and pushed to GitHub:

```bash
# Check that these files exist in your repository
git ls-files .github/

# Should show:
# .github/ISSUE_TEMPLATE/bug.yml
# .github/ISSUE_TEMPLATE/chore.yml
# .github/ISSUE_TEMPLATE/config.yml
# .github/ISSUE_TEMPLATE/discussion.yml
# .github/ISSUE_TEMPLATE/enhancement.yml
# .github/ISSUE_TEMPLATE/tracking.yml
# .github/workflows/setup-labels.yml
```

If any files are missing, commit and push them:

```bash
git add .github/
git commit -m "chore(github): add issue templates and workflows"
git push origin main
```

## Step 2: Run the Label Setup Workflow

The label setup workflow creates all the predefined labels in your repository.

### Manual Trigger via GitHub UI

1. Navigate to your repository on GitHub
2. Click on the **Actions** tab
3. In the left sidebar, find and click **"Project Infrastructure: Setup Labels"**
4. Click the **"Run workflow"** button (top right)
5. Select the branch (usually `main`)
6. Click the green **"Run workflow"** button

### Expected Output

The workflow will create the following labels:

**Issue Type Labels:**

- `tracking` (purple) - High-level Epic/Feature tracking issue
- `enhancement` (light blue) - New functionality or requirement
- `chore` (yellow) - Internal technical tasks and maintenance
- `bug` (red) - Functional defect or error
- `discussion` (light purple) - Architectural proposals or RFCs

**Priority Labels:**

- `priority:high` (dark red) - Urgent items requiring immediate attention
- `priority:medium` (yellow) - Important but not urgent
- `priority:low` (green) - Nice to have, low urgency

**Size Labels:**

- `size:small` (light green) - Small effort (< 1 day)
- `size:medium` (light blue) - Medium effort (1-3 days)
- `size:large` (light orange) - Large effort (> 3 days)

**Area Labels:**

- `area:core` (light blue) - Core library functionality
- `area:ffi` (light blue) - FFI bindings and C interop
- `area:protocol` (light blue) - Protocol implementation (XML/JSON)
- `area:discovery` (light blue) - Server discovery (ZeroConf/Bonjour)
- `area:testing` (light blue) - Testing infrastructure and tests
- `area:docs` (light blue) - Documentation

### Verify Labels Were Created

1. Go to your repository on GitHub
2. Click on **Issues** tab
3. Click on **Labels** (next to Milestones)
4. Verify all labels are present with correct colors and descriptions

### Troubleshooting

If the workflow fails:

**Permission Error:**

```
Error: Resource not accessible by integration
```

Solution: Ensure GitHub Actions has write permissions:

1. Go to Settings → Actions → General
2. Under "Workflow permissions", select "Read and write permissions"
3. Click Save
4. Re-run the workflow

**Label Already Exists:**

```
Label "enhancement" already exists, skipping.
```

This is normal - the workflow skips existing labels.

## Step 3: Verify Issue Templates

Test that issue templates are working:

1. Go to your repository on GitHub
2. Click **Issues** tab
3. Click **New issue** button
4. You should see a template chooser with:
   - ✨ Enhancement
   - 🛠️ Chore
   - 🐛 Bug
   - 📍 Tracking Issue
   - 🔍 Discussion

5. Click on each template to verify the form fields are correct

### Troubleshooting

**Templates Not Showing:**

- Verify files are in `.github/ISSUE_TEMPLATE/` directory
- Check that files have `.yml` extension (not `.yaml`)
- Ensure files are valid YAML (no syntax errors)
- Wait a few minutes for GitHub to process the changes

**Config.yml Issues:**

- If you see "Open a blank issue" option and don't want it, check that `blank_issues_enabled: false` is set in `config.yml`
- Update the contact links in `config.yml` to match your repository

## Step 4: Update Config.yml (Optional)

Edit `.github/ISSUE_TEMPLATE/config.yml` to customize contact links:

```yaml
blank_issues_enabled: false
contact_links:
  - name: 💬 GitHub Discussions
    url: https://github.com/YOUR_ORG/libindigo/discussions  # Update this
    about: For general questions and community discussions
  - name: 📚 Documentation
    url: https://github.com/YOUR_ORG/libindigo/tree/main/doc  # Update this
    about: Check the documentation for guides and references
```

Replace `YOUR_ORG` with your GitHub organization or username.

Commit and push the changes:

```bash
git add .github/ISSUE_TEMPLATE/config.yml
git commit -m "chore(github): update issue template config links"
git push origin main
```

## Step 5: Create Milestones (Optional)

Create milestones for release planning:

1. Go to **Issues** → **Milestones**
2. Click **New milestone**
3. Create milestones for planned releases:
   - **v0.2.0** - Property extraction and constants generation
   - **v0.3.0** - High-level trait-based device API
   - **v1.0.0** - First stable release

For each milestone:

- Set a due date (optional)
- Add a description
- Click **Create milestone**

## Step 6: Test the Complete Workflow

Create a test issue to verify everything works:

1. Click **New issue**
2. Select **✨ Enhancement** template
3. Fill in the form:
   - Goal: "Test issue template"
   - Acceptance Criteria: "- [ ] Template works correctly"
4. Click **Submit new issue**
5. Verify:
   - Issue has `enhancement` label
   - Form fields are properly formatted
   - Issue number is assigned

6. Add additional labels:
   - Click **Labels** on the right sidebar
   - Add `priority:low` and `size:small`
   - Add `area:docs` (since this is a test)

7. Close the test issue

## Step 7: Set Up Branch Protection (Recommended)

Protect your main branch:

1. Go to **Settings** → **Branches**
2. Click **Add rule** under "Branch protection rules"
3. Branch name pattern: `main`
4. Enable:
   - ✅ Require a pull request before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require conversation resolution before merging
5. Click **Create** or **Save changes**

## Step 8: Configure GitHub Actions Permissions

Ensure workflows have necessary permissions:

1. Go to **Settings** → **Actions** → **General**
2. Under "Workflow permissions":
   - Select **Read and write permissions**
   - ✅ Allow GitHub Actions to create and approve pull requests
3. Click **Save**

## Verification Checklist

Before considering setup complete, verify:

- [ ] All issue templates appear in the New Issue chooser
- [ ] All labels are created with correct colors
- [ ] Test issue can be created successfully
- [ ] Labels can be applied to issues
- [ ] Milestones are created (if using)
- [ ] Branch protection is configured (if desired)
- [ ] GitHub Actions has write permissions
- [ ] Config.yml links are updated to your repository

## Next Steps

After completing the GitHub setup:

1. **Create Initial Issues**: Convert plans from `plans/` directory to GitHub issues
2. **Create Tracking Issue**: Create a tracking issue for v0.2.0 release
3. **Assign Milestones**: Assign issues to appropriate milestones
4. **Start Development**: Begin working on issues using the Roo AI workflow

## Reference Documentation

- [GitHub Issue Templates](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository)
- [GitHub Actions](https://docs.github.com/en/actions)
- [GitHub Labels](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels)
- [Ways of Working](./ways-of-working.md) - Project issue types and workflow
- [Roo Workflow Scheme](./roo-workflow-scheme.md) - AI persona workflow guide

## Troubleshooting Common Issues

### Issue Templates Not Showing

**Problem:** Templates don't appear in the New Issue chooser.

**Solutions:**

1. Verify files are in `.github/ISSUE_TEMPLATE/` (not `.github/ISSUE_TEMPLATES/`)
2. Check file extensions are `.yml` (not `.yaml`)
3. Validate YAML syntax: <https://www.yamllint.com/>
4. Wait 5-10 minutes for GitHub to process changes
5. Try hard refresh (Ctrl+Shift+R or Cmd+Shift+R)

### Labels Not Created

**Problem:** Workflow runs but labels aren't created.

**Solutions:**

1. Check workflow logs for errors
2. Verify GitHub Actions has write permissions
3. Check if labels already exist (workflow skips existing)
4. Manually create labels if workflow continues to fail

### Permission Errors

**Problem:** Workflow fails with "Resource not accessible by integration"

**Solutions:**

1. Go to Settings → Actions → General
2. Set "Workflow permissions" to "Read and write permissions"
3. Enable "Allow GitHub Actions to create and approve pull requests"
4. Re-run the workflow

### Config.yml Not Working

**Problem:** Blank issues still enabled or contact links not showing.

**Solutions:**

1. Verify `config.yml` is in `.github/ISSUE_TEMPLATE/` directory
2. Check YAML syntax is valid
3. Ensure `blank_issues_enabled: false` is set
4. Update URLs to match your repository
5. Wait a few minutes and refresh

## Support

If you encounter issues not covered here:

1. Check GitHub's status page: <https://www.githubstatus.com/>
2. Review GitHub documentation: <https://docs.github.com/>
3. Check repository's GitHub Discussions (if enabled)
4. Create an issue using the 🐛 Bug template
