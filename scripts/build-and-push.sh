
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

# Collect proxy settings from environment if available
# Convert 127.0.0.1 to host.docker.internal for Docker containers (macOS/Windows)
BUILD_ARGS=()
convert_proxy_for_docker() {
    local proxy="$1"
    if [[ "$proxy" =~ ^http://127\.0\.0\.1: ]] || [[ "$proxy" =~ ^https://127\.0\.0\.1: ]]; then
        # Replace 127.0.0.1 with host.docker.internal
        echo "$proxy" | sed 's|127\.0\.0\.1|host.docker.internal|g'
    else
        echo "$proxy"
    fi
}

if [ -n "${http_proxy:-}" ]; then
    DOCKER_PROXY=$(convert_proxy_for_docker "${http_proxy}")
    BUILD_ARGS+=(--build-arg "http_proxy=${DOCKER_PROXY}")
    BUILD_ARGS+=(--build-arg "HTTP_PROXY=${DOCKER_PROXY}")
    echo "Using HTTP proxy: ${DOCKER_PROXY}"
fi
if [ -n "${https_proxy:-}" ]; then
    DOCKER_PROXY=$(convert_proxy_for_docker "${https_proxy}")
    BUILD_ARGS+=(--build-arg "https_proxy=${DOCKER_PROXY}")
    BUILD_ARGS+=(--build-arg "HTTPS_PROXY=${DOCKER_PROXY}")
    echo "Using HTTPS proxy: ${DOCKER_PROXY}"
fi
if [ -n "${HTTP_PROXY:-}" ]; then
    DOCKER_PROXY=$(convert_proxy_for_docker "${HTTP_PROXY}")
    BUILD_ARGS+=(--build-arg "HTTP_PROXY=${DOCKER_PROXY}")
    BUILD_ARGS+=(--build-arg "http_proxy=${DOCKER_PROXY}")
    echo "Using HTTP proxy: ${DOCKER_PROXY}"
fi
if [ -n "${HTTPS_PROXY:-}" ]; then
    DOCKER_PROXY=$(convert_proxy_for_docker "${HTTPS_PROXY}")
    BUILD_ARGS+=(--build-arg "HTTPS_PROXY=${DOCKER_PROXY}")
    BUILD_ARGS+=(--build-arg "https_proxy=${DOCKER_PROXY}")
    echo "Using HTTPS proxy: ${DOCKER_PROXY}"
fi

# Build with both tags (from project root)
docker build -f docker/Dockerfile \
    -t "${IMAGE_BRANCH}" \
    -t "${IMAGE_SHA}" \
    "${BUILD_ARGS[@]}" \
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
    # Login to registry before pushing
    login_to_registry() {
        local registry="$1"
        
        # Check if already logged in by checking docker config
        if [ -f ~/.docker/config.json ] && grep -q "\"${registry}\"" ~/.docker/config.json 2>/dev/null; then
            echo "Already authenticated to ${registry}"
            return 0
        fi
        
        echo "Authenticating to ${registry}..."
        
        # Try GITHUB_TOKEN first (for ghcr.io)
        if [ -n "${GITHUB_TOKEN:-}" ]; then
            echo "Using GITHUB_TOKEN for authentication..."
            # Get GitHub username - try multiple sources
            local username
            if [ -n "${GITHUB_ACTOR:-}" ]; then
                username="${GITHUB_ACTOR}"
            elif [ -n "${GITHUB_USERNAME:-}" ]; then
                username="${GITHUB_USERNAME}"
            else
                # Try to get from git config, fallback to current user
                username=$(git config user.name 2>/dev/null || echo "${USER}")
            fi
            
            if echo "${GITHUB_TOKEN}" | docker login "${registry}" -u "${username}" --password-stdin 2>/dev/null; then
                echo "✓ Successfully authenticated using GITHUB_TOKEN"
                return 0
            else
                echo "✗ GITHUB_TOKEN authentication failed, trying interactive login..."
            fi
        fi
        
        # Fallback to interactive login
        echo ""
        if [ "${registry}" = "ghcr.io" ]; then
            echo "For GitHub Container Registry:"
            echo "  Username: your GitHub username"
            echo "  Password: GitHub Personal Access Token (PAT)"
            echo "  PAT needs 'write:packages' permission"
            echo ""
        fi
        docker login "${registry}"
        return $?
    }
    
    # Login before pushing
    if ! login_to_registry "${REGISTRY}"; then
        echo "Error: Failed to authenticate to ${REGISTRY}"
        echo "Please check your credentials and try again."
        exit 1
    fi
    
    echo ""
    echo "Pushing ${IMAGE_BRANCH}..."
    if ! docker push "${IMAGE_BRANCH}"; then
        echo "Error: Failed to push ${IMAGE_BRANCH}"
        exit 1
    fi

    echo "Pushing ${IMAGE_SHA}..."
    if ! docker push "${IMAGE_SHA}"; then
        echo "Error: Failed to push ${IMAGE_SHA}"
        exit 1
    fi

    echo ""
    echo "✓ Images pushed successfully!"
    echo ""
    echo "Deploy with:"
    echo "kubectl set image -n starcoin-proxima statefulset/starcoin starcoin=${IMAGE_SHA}"
fi