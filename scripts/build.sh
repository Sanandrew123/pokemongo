#!/bin/bash
# 高性能宝可梦游戏构建脚本
# 开发心理：自动化构建流程，支持Rust和C++混合编译

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🚀 Pokemon GO 高性能游戏构建器${NC}"
echo "======================================"

# 解析命令行参数
BUILD_MODE="debug"
BUILD_NATIVE=false
RUN_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --native)
            BUILD_NATIVE=true
            shift
            ;;
        --test)
            RUN_TESTS=true
            shift
            ;;
        *)
            echo "未知参数: $1"
            exit 1
            ;;
    esac
done

echo -e "${YELLOW}构建模式: $BUILD_MODE${NC}"
echo -e "${YELLOW}构建C++模块: $BUILD_NATIVE${NC}"
echo -e "${YELLOW}运行测试: $RUN_TESTS${NC}"

# 检查依赖
check_dependencies() {
    echo -e "\n${YELLOW}检查构建依赖...${NC}"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}错误: 需要安装 Rust/Cargo${NC}"
        exit 1
    fi
    
    if [ "$BUILD_NATIVE" = true ]; then
        if ! command -v cmake &> /dev/null; then
            echo -e "${RED}错误: 需要安装 CMake${NC}"
            exit 1
        fi
        
        if ! command -v g++ &> /dev/null; then
            echo -e "${RED}错误: 需要安装 G++${NC}" 
            exit 1
        fi
    fi
    
    echo -e "${GREEN}✓ 依赖检查完成${NC}"
}

# 构建C++模块
build_native() {
    if [ "$BUILD_NATIVE" = true ]; then
        echo -e "\n${YELLOW}构建C++高性能模块...${NC}"
        
        mkdir -p build
        cd build
        
        cmake .. -DCMAKE_BUILD_TYPE=$([ "$BUILD_MODE" = "release" ] && echo "Release" || echo "Debug")
        make -j$(nproc)
        
        cd ..
        echo -e "${GREEN}✓ C++模块构建完成${NC}"
    fi
}

# 构建Rust项目
build_rust() {
    echo -e "\n${YELLOW}构建Rust游戏引擎...${NC}"
    
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --release --no-default-features
    else
        cargo build --no-default-features
    fi
    
    echo -e "${GREEN}✓ Rust项目构建完成${NC}"
}

# 运行测试
run_tests() {
    if [ "$RUN_TESTS" = true ]; then
        echo -e "\n${YELLOW}运行测试套件...${NC}"
        
        cargo test --no-default-features -- --nocapture
        
        echo -e "${GREEN}✓ 测试完成${NC}"
    fi
}

# 生成构建报告
generate_report() {
    echo -e "\n${YELLOW}生成构建报告...${NC}"
    
    BINARY_SIZE=""
    if [ "$BUILD_MODE" = "release" ]; then
        BINARY_PATH="target/release/pokemongo"
    else
        BINARY_PATH="target/debug/pokemongo"
    fi
    
    if [ -f "$BINARY_PATH" ]; then
        BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    fi
    
    echo -e "\n${GREEN}📊 构建报告${NC}"
    echo "==================="
    echo "构建模式: $BUILD_MODE"
    echo "C++模块: $([ "$BUILD_NATIVE" = true ] && echo "已构建" || echo "跳过")"
    echo "测试状态: $([ "$RUN_TESTS" = true ] && echo "已运行" || echo "跳过")"
    [ -n "$BINARY_SIZE" ] && echo "二进制大小: $BINARY_SIZE"
    echo "构建时间: $(date)"
}

# 主构建流程
main() {
    local start_time=$(date +%s)
    
    check_dependencies
    build_native
    build_rust
    run_tests
    generate_report
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo -e "\n${GREEN}🎉 构建完成！耗时: ${duration}秒${NC}"
    echo -e "${GREEN}运行演示: cargo run --bin demo --no-default-features${NC}"
}

# 清理构建文件
clean() {
    echo -e "${YELLOW}清理构建文件...${NC}"
    cargo clean
    [ -d "build" ] && rm -rf build
    echo -e "${GREEN}✓ 清理完成${NC}"
}

# 如果第一个参数是 clean，执行清理
if [ "$1" = "clean" ]; then
    clean
    exit 0
fi

# 执行主构建流程
main "$@"