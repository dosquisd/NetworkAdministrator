#!/bin/bash

# Simple CLI to run ansible playbooks with fixed inventory and key
# Usage: ./run-playbooks.sh <playbook-path>

if [ $# -eq 0 ]; then
    echo "Error: No playbook specified"
    echo "Usage: $0 <playbook-path>"
    echo ""
    echo "Examples:"
    echo "  $0 ./ansible/homebrew-router.yaml"
    echo "  $0 ./ansible/configure-systemd.yaml"
    echo "  $0 ./ansible/update-repo.yaml"
    exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

PLAYBOOK="$1"
INVENTORY="$PROJECT_ROOT/ansible/inventory.ini"

if [ ! -f "$PLAYBOOK" ]; then
    echo "Error: Playbook not found: $PLAYBOOK"
    exit 1
fi

if [ ! -f "$INVENTORY" ]; then
    echo "Error: Inventory not found: $INVENTORY"
    exit 1
fi

echo "Running playbook: $PLAYBOOK"
echo "Inventory: $INVENTORY"
echo ""

ansible-playbook -i "$INVENTORY" \
    "$PLAYBOOK"
