#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Matrix Bridge Messager - Build Script${NC}"
echo "=================================="

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    exit 1
fi

# Build the Docker image
echo -e "${YELLOW}Building Docker image...${NC}"
docker build -t matrix-bridge-messager:latest .

echo -e "${GREEN}Build complete!${NC}"
echo ""
echo "To run the bridge:"
echo "  docker run -d -p 9006:9006 -v \$(pwd)/config.yaml:/data/config.yaml matrix-bridge-messager:latest"
echo ""
echo "Or with docker-compose:"
echo "  docker-compose up -d"


