# 🔒 Sensitive Files Cleanup Guide

## Current Status ✅
**Good News**: Based on the scan, your main sensitive files (.env files) are **NOT currently tracked by git**.

However, here's the complete process to clean up any sensitive files that might be tracked:

## 🧹 **Method 1: Remove from Latest Commit Only**
If you just committed sensitive files in your most recent commit:

```bash
# Remove specific files from the last commit
git rm --cached .env src/DeFlow_admin/.env
git commit --amend --no-edit

# Or reset the entire last commit (if it only contains sensitive files)
git reset --soft HEAD~1
```

## 🔥 **Method 2: Remove from Entire Git History (NUCLEAR OPTION)**
**⚠️ WARNING: This rewrites history and requires force push**

### For Specific Files:
```bash
# Remove specific sensitive files from entire history
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch .env src/DeFlow_admin/.env *.key *.pem' \
  --prune-empty --tag-name-filter cat -- --all

# Alternative using git-filter-repo (recommended if available)
pip install git-filter-repo
git filter-repo --path .env --invert-paths
git filter-repo --path src/DeFlow_admin/.env --invert-paths
```

### For Files by Pattern:
```bash
# Remove all .env files from history
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch *.env */.env **/*.env' \
  --prune-empty --tag-name-filter cat -- --all

# Remove all key/certificate files
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch *.key *.pem *.crt *.p12 *.pfx' \
  --prune-empty --tag-name-filter cat -- --all
```

## 🛡️ **Method 3: Using BFG Repo-Cleaner (Recommended)**
BFG is faster and safer than git filter-branch:

```bash
# Install BFG
brew install bfg  # On macOS
# or download from: https://rtyley.github.io/bfg-repo-cleaner/

# Remove files by name
bfg --delete-files ".env"
bfg --delete-files "*.key"
bfg --delete-files "*.pem"

# Remove folders
bfg --delete-folders "secrets"
bfg --delete-folders "*-keys"

# Clean up
git reflog expire --expire=now --all && git gc --prune=now --aggressive
```

## 📁 **Method 4: Remove Specific File Types**

Create a cleanup script:

```bash
#!/bin/bash
# cleanup-sensitive.sh

echo "🔍 Scanning for sensitive files..."

# Files to remove from git history
SENSITIVE_PATTERNS=(
    "*.env"
    "*.key" 
    "*.pem"
    "*.crt"
    "*.p12"
    "*.secret"
    "*password*"
    "*token*"
    "canister_ids.json"
    "identity.pem"
    ".ii-identity"
    ".nfid-keys"
)

for pattern in "${SENSITIVE_PATTERNS[@]}"; do
    echo "Removing $pattern from git history..."
    git filter-branch --force --index-filter \
        "git rm --cached --ignore-unmatch '$pattern'" \
        --prune-empty --tag-name-filter cat -- --all
done

echo "🧹 Cleaning up..."
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo "✅ Cleanup complete!"
```

## 🚀 **After Cleanup Steps**

1. **Force Push Changes** (⚠️ Coordinate with team):
```bash
git push origin --force --all
git push origin --force --tags
```

2. **Verify Cleanup**:
```bash
# Check that files are gone from history
git log --name-only --pretty=format: --all | sort -u | grep -E "\.(env|key|pem)$"

# Should return no results
```

3. **Team Coordination**:
```bash
# All team members need to re-clone:
git clone <repository-url>

# Or if they have local copies:
git fetch origin
git reset --hard origin/main  # Replace 'main' with your default branch
```

## 🛡️ **Prevention Going Forward**

1. **Always check before committing**:
```bash
git status
git diff --cached  # Review staged changes
```

2. **Use git hooks** to prevent sensitive commits:
```bash
# Create .git/hooks/pre-commit
#!/bin/sh
if git diff --cached --name-only | grep -E "\.(env|key|pem|secret)$"; then
    echo "❌ Sensitive files detected! Commit aborted."
    echo "Files:"
    git diff --cached --name-only | grep -E "\.(env|key|pem|secret)$"
    exit 1
fi
```

3. **Regular audits**:
```bash
# Check for accidentally tracked files
git ls-files | grep -E "\.(env|key|pem|secret)$"
```

## 📋 **Quick Commands Summary**

```bash
# Remove from staging (not committed yet)
git reset HEAD .env src/DeFlow_admin/.env

# Remove from last commit only
git rm --cached .env src/DeFlow_admin/.env
git commit --amend --no-edit

# Remove from entire history (nuclear option)
git filter-branch --force --index-filter 'git rm --cached --ignore-unmatch .env src/DeFlow_admin/.env' --prune-empty --tag-name-filter cat -- --all

# Clean up after filter-branch
git reflog expire --expire=now --all && git gc --prune=now --aggressive

# Force push (coordinate with team!)
git push origin --force --all
```

## 🎯 **Recommendation**

Since your sensitive files (.env files) are **not currently tracked**, you just need to:

1. ✅ **Verify they stay untracked**: `git status` (should not show them)
2. ✅ **Your .gitignore is updated** (already done)
3. ✅ **Create .env.example templates** for team members

**You're already in good shape! No cleanup needed.** 🎉