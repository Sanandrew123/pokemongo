#!/bin/bash
# 测试脚本
# 开发心理：全面的测试策略，确保游戏质量和性能

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🧪 Pokemon GO 测试套件${NC}"
echo "=========================="

# 单元测试
echo -e "\n${YELLOW}运行单元测试...${NC}"
cargo test --no-default-features --lib

# 集成测试  
echo -e "\n${YELLOW}运行集成测试...${NC}"
cargo test --no-default-features --tests

# 文档测试
echo -e "\n${YELLOW}运行文档测试...${NC}"
cargo test --no-default-features --doc

# 运行演示程序
echo -e "\n${YELLOW}运行功能演示...${NC}"
cargo run --bin demo --no-default-features

echo -e "\n${GREEN}✅ 所有测试通过！${NC}"