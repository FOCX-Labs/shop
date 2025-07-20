#!/bin/bash

# Solana电商平台项目清理脚本
# 用途：安全清理项目文件，减少内存占用，不影响项目功能
# 作者：Augment Agent
# 日期：2025-07-19

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}🧹 Solana电商平台项目清理工具${NC}"
echo -e "${BLUE}======================================${NC}"
echo -e "📁 项目路径: $PROJECT_ROOT"
echo ""

# 检查是否在正确的项目目录
if [ ! -f "Anchor.toml" ] || [ ! -d "programs/solana-e-commerce" ]; then
    echo -e "${RED}❌ 错误：当前目录不是Solana电商项目根目录${NC}"
    exit 1
fi

# 计算清理前的大小
echo -e "${YELLOW}📊 计算清理前项目大小...${NC}"
BEFORE_SIZE=$(du -sh . 2>/dev/null | cut -f1 || echo "未知")
echo -e "清理前大小: ${BEFORE_SIZE}"
echo ""

# 清理函数
cleanup_section() {
    local section_name="$1"
    local description="$2"
    echo -e "${BLUE}🔧 ${section_name}${NC}"
    echo -e "   ${description}"
}

# 1. 清理Rust编译产物
cleanup_section "清理Rust编译产物" "删除target目录中的编译缓存"
if [ -d "target" ]; then
    echo -e "   🗑️  删除 target/ 目录..."
    rm -rf target/
    echo -e "   ✅ target/ 目录已删除"
else
    echo -e "   ℹ️  target/ 目录不存在，跳过"
fi
echo ""

# 2. 清理Node.js依赖缓存
cleanup_section "清理Node.js缓存" "删除node_modules和包管理器缓存"
if [ -d "node_modules" ]; then
    echo -e "   🗑️  删除 node_modules/ 目录..."
    rm -rf node_modules/
    echo -e "   ✅ node_modules/ 目录已删除"
else
    echo -e "   ℹ️  node_modules/ 目录不存在，跳过"
fi

# 清理npm缓存
if command -v npm >/dev/null 2>&1; then
    echo -e "   🧹 清理npm缓存..."
    npm cache clean --force >/dev/null 2>&1 || true
    echo -e "   ✅ npm缓存已清理"
fi

# 清理yarn缓存
if command -v yarn >/dev/null 2>&1; then
    echo -e "   🧹 清理yarn缓存..."
    yarn cache clean >/dev/null 2>&1 || true
    echo -e "   ✅ yarn缓存已清理"
fi
echo ""

# 3. 清理测试和日志文件
cleanup_section "清理测试和日志文件" "删除临时测试文件和日志"

# 清理测试报告
find . -name "*test-report*.md" -type f -delete 2>/dev/null || true
find . -name "*test-result*.json" -type f -delete 2>/dev/null || true
echo -e "   ✅ 测试报告文件已清理"

# 清理日志文件
find . -name "*.log" -type f -delete 2>/dev/null || true
find . -name "*.log.*" -type f -delete 2>/dev/null || true
echo -e "   ✅ 日志文件已清理"

# 清理临时文件
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name "*.temp" -type f -delete 2>/dev/null || true
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
echo -e "   ✅ 临时文件已清理"
echo ""

# 4. 清理Solana本地数据
cleanup_section "清理Solana本地数据" "删除本地验证器数据和缓存"

# 清理test-ledger
if [ -d "test-ledger" ]; then
    echo -e "   🗑️  删除 test-ledger/ 目录..."
    rm -rf test-ledger/
    echo -e "   ✅ test-ledger/ 目录已删除"
else
    echo -e "   ℹ️  test-ledger/ 目录不存在，跳过"
fi

# 清理.anchor目录
if [ -d ".anchor" ]; then
    echo -e "   🗑️  删除 .anchor/ 目录..."
    rm -rf .anchor/
    echo -e "   ✅ .anchor/ 目录已删除"
else
    echo -e "   ℹ️  .anchor/ 目录不存在，跳过"
fi
echo ""

# 5. 清理IDE和编辑器缓存
cleanup_section "清理IDE和编辑器缓存" "删除编辑器临时文件"

# VSCode缓存
find . -name ".vscode" -type d -exec rm -rf {} + 2>/dev/null || true
echo -e "   ✅ VSCode缓存已清理"

# Vim缓存
find . -name "*.swp" -type f -delete 2>/dev/null || true
find . -name "*.swo" -type f -delete 2>/dev/null || true
find . -name "*~" -type f -delete 2>/dev/null || true
echo -e "   ✅ Vim缓存已清理"

# Emacs缓存
find . -name "*#" -type f -delete 2>/dev/null || true
find . -name ".#*" -type f -delete 2>/dev/null || true
echo -e "   ✅ Emacs缓存已清理"
echo ""

# 6. 清理Git缓存（可选）
cleanup_section "清理Git缓存" "清理Git对象缓存"
if [ -d ".git" ]; then
    echo -e "   🧹 运行git垃圾回收..."
    git gc --prune=now >/dev/null 2>&1 || true
    echo -e "   ✅ Git缓存已清理"
else
    echo -e "   ℹ️  不是Git仓库，跳过Git清理"
fi
echo ""

# 7. 保留重要文件检查
cleanup_section "保留重要文件检查" "确认重要文件未被删除"
important_files=(
    "Anchor.toml"
    "Cargo.toml"
    "package.json"
    "programs/solana-e-commerce/src/lib.rs"
    "scripts/small-scale-complete-test.ts"
)

all_files_exist=true
for file in "${important_files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "   ✅ $file 存在"
    else
        echo -e "   ❌ $file 缺失"
        all_files_exist=false
    fi
done

if [ "$all_files_exist" = true ]; then
    echo -e "   ${GREEN}✅ 所有重要文件都完整保留${NC}"
else
    echo -e "   ${RED}⚠️  部分重要文件缺失，请检查${NC}"
fi
echo ""

# 计算清理后的大小
echo -e "${YELLOW}📊 计算清理后项目大小...${NC}"
AFTER_SIZE=$(du -sh . 2>/dev/null | cut -f1 || echo "未知")
echo -e "清理后大小: ${AFTER_SIZE}"
echo ""

# 清理完成
echo -e "${GREEN}🎉 项目清理完成！${NC}"
echo -e "${GREEN}=================${NC}"
echo -e "📊 清理前大小: ${BEFORE_SIZE}"
echo -e "📊 清理后大小: ${AFTER_SIZE}"
echo ""
echo -e "${BLUE}💡 下次使用项目时，请运行以下命令恢复依赖：${NC}"
echo -e "   ${YELLOW}npm install${NC}  # 或 yarn install"
echo -e "   ${YELLOW}anchor build${NC}  # 重新编译Rust程序"
echo ""
echo -e "${GREEN}✅ 项目内存占用已显著减少，核心代码完整保留！${NC}"
