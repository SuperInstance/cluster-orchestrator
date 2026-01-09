#!/bin/bash
# Install Chaos Mesh for chaos engineering tests

set -e

CHAOS_MESH_VERSION="v0.25.0"
CHAOS_NAMESPACE="chaos-testing"

echo "Installing Chaos Mesh ${CHAOS_MESH_VERSION}..."

# Create namespace
kubectl create namespace ${CHAOS_NAMESPACE} || true

# Install Chaos Mesh
curl -sSL https://mirrors.chaos-mesh.org/${CHAOS_MESH_VERSION}/install.sh | bash

# Wait for Chaos Mesh to be ready
echo "Waiting for Chaos Mesh to be ready..."
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=chaos-mesh -n ${CHAOS_NAMESPACE} --timeout=300s

# Verify installation
if kubectl get pods -n ${CHAOS_NAMESPACE} | grep -q chaos-mesh; then
    echo "Chaos Mesh installed successfully!"
else
    echo "Failed to install Chaos Mesh"
    exit 1
fi
