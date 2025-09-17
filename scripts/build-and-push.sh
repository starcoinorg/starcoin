#!/usr/bin/env bash

set -e

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Check if we're in scripts directory or project root
if [ -f "${SCRIPT_DIR}/docker/Dockerfile" ]; then
    PROJECT_ROOT="${SCRIPT_DIR}"
else
    PROJECT_ROOT="$( cd "${SCRIPT_DIR}/.." && pwd )"
fi

# Change to project root for git commands and docker build
cd "${PROJECT_ROOT}"

# Get current branch name and commit SHA
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "detached")
SHA=$(git rev-parse --short=7 HEAD)

# Clean branch name for docker tag
BRANCH_TAG=$(echo "$BRANCH" | sed 's/[^a-zA-Z0-9._-]/-/g')
if [ "$BRANCH" = "HEAD" ]; then
    BRANCH_TAG="sha-${SHA}"
fi

# Image names
REGISTRY="ghcr.io"
IMAGE_BASE="${REGISTRY}/starcoinorg/starcoin"
IMAGE_BRANCH="${IMAGE_BASE}:${BRANCH_TAG}"
IMAGE_SHA="${IMAGE_BASE}:sha-${SHA}"

echo "Building Docker image..."
echo "Project root: ${PROJECT_ROOT}"
echo "Branch: $BRANCH_TAG"
echo "SHA: $SHA"

# Build with both tags (from project root)
docker build -f docker/Dockerfile \
    -t "${IMAGE_BRANCH}" \
    -t "${IMAGE_SHA}" \
    "$@" \
    .

echo "Images built successfully:"
echo "  - ${IMAGE_BRANCH}"
echo "  - ${IMAGE_SHA}"

# Push to registry
echo ""
read -p "Push to ${REGISTRY}? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Pushing ${IMAGE_BRANCH}..."
    docker push "${IMAGE_BRANCH}"

    echo "Pushing ${IMAGE_SHA}..."
    docker push "${IMAGE_SHA}"

    echo ""
    echo "Deploy with:"
    echo "kubectl set image -n starcoin-proxima statefulset/starcoin starcoin=${IMAGE_SHA}"
fi