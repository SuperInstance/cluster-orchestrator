#!/bin/bash
# Setup script for test cluster using kind

set -e

CLUSTER_NAME="cluster-orchestrator-test"
KIND_VERSION="v0.20.0"

echo "Setting up test cluster..."

# Check if kind is installed
if ! command -v kind &> /dev/null; then
    echo "Installing kind..."
    curl -Lo ./kind "https://kind.sigs.k8s.io/dl/${KIND_VERSION}/kind-linux-amd64"
    chmod +x ./kind
    sudo mv ./kind /usr/local/bin/kind
fi

# Check if cluster exists
if kind get clusters | grep -q "^${CLUSTER_NAME}$"; then
    echo "Cluster ${CLUSTER_NAME} already exists"
    echo "Deleting existing cluster..."
    kind delete cluster --name ${CLUSTER_NAME}
fi

# Create cluster
echo "Creating kind cluster: ${CLUSTER_NAME}"
kind create cluster --name ${CLUSTER_NAME} --config scripts/kind-config.yaml

# Wait for cluster to be ready
echo "Waiting for cluster to be ready..."
kubectl wait --for=condition=ready node --all --timeout=300s

# Install metrics server (required for HPA)
echo "Installing metrics server..."
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# Wait for metrics server
kubectl wait --for=condition=available deployment/metrics-server -n kube-system --timeout=300s

echo "Test cluster setup complete!"
echo "Cluster: ${CLUSTER_NAME}"
echo "Kubeconfig: $(kind get kubeconfig-path --name ${CLUSTER_NAME})"
