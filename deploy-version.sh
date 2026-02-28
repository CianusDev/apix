#!/usr/bin/env bash
set -e

# Usage: ./deploy-version.sh <version>
# Example: ./deploy-version.sh 1.0.2

VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 1.0.2"
  exit 1
fi

# Validate format x.y.z
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Error: version must be in format x.y.z (e.g. 1.0.2)"
  exit 1
fi

TAG="v$VERSION"

# Check no uncommitted changes
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Error: uncommitted changes detected. Commit or stash them first."
  exit 1
fi

# Check we are on main
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "main" ]; then
  echo "Error: must be on branch 'main' (current: $BRANCH)"
  exit 1
fi

# Check tag doesn't already exist
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: tag $TAG already exists"
  exit 1
fi

echo "Deploying apix $TAG..."

# Bump version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update Cargo.lock
cargo update --workspace --quiet

git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION"
git push origin main

git tag "$TAG"
git push origin "$TAG"

echo "Done — GitHub Actions will build and publish release $TAG"
