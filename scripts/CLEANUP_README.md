# Scripts 目录清理说明

本目录已清理，移除了不必要的测试文件和工具脚本，只保留小规模完整性测试相关文件。

## 保留的文件

### 核心测试文件

-   `small-scale-complete-test.ts` - 小规模完整性测试（支持本地和 devnet 环境）

### 配置文件

-   `spl-tokens-local.json` - 本地环境 SPL 代币配置
-   `spl-tokens-devnet.json` - Devnet 环境 SPL 代币配置

### 密钥文件

-   `buyer-a-keypair.json` - 买家密钥
-   `merchant-a-keypair.json` - 商户密钥

### 测试报告

-   `small-scale-test-report.md` - 测试执行报告

### 工具脚本

-   `cleanup-project.sh` - 项目清理脚本

## 已移除的文件

以下文件已被移除：

-   `compatibility-test.ts` - 兼容性测试
-   `order-management-focused-test.ts` - 订单管理专项测试
-   `product-update-test.ts` - 商品更新测试
-   `test-refactor.ts` - 重构测试
-   `test-refactored-product.ts` - 重构商品测试
-   `verify-refactor.ts` - 重构验证测试
-   `utils/` 目录及其所有内容：
    -   `batch-executor.ts` - 批处理执行器
    -   `batch-transaction-builder.ts` - 批处理交易构建器
    -   `nonce-manager.ts` - Nonce 管理器
    -   `offline-signer.ts` - 离线签名器
    -   `product-listing-batch-builder.ts` - 商品批量上架构建器
-   `devnet/` 目录及其所有内容：
    -   `small-scale-complete-test.ts` - Devnet 环境测试脚本
    -   `buyer-a-keypair.json` - Devnet 买家密钥
    -   `merchant-a-keypair.json` - Devnet 商户密钥

## 使用说明

### 本地环境测试

```bash
# 启动本地验证器
solana-test-validator --reset --compute-unit-limit 1400000 --limit-ledger-size 1400000

# 构建和部署程序
anchor build
anchor deploy

# 运行小规模完整性测试
npx ts-node scripts/small-scale-complete-test.ts --local
```

### Devnet 环境测试

```bash
# 设置网络代理（如需要）
export http_proxy=http://127.0.0.1:7890
export https_proxy=http://127.0.0.1:7890

# 运行devnet测试（使用同一个测试文件）
npx ts-node scripts/small-scale-complete-test.ts
```

## 清理效果

-   **文件数量减少**: 从 20+ 个文件减少到 8 个文件（-60%）
-   **目录结构简化**: 移除了 utils 和 devnet 目录
-   **专注核心功能**: 只保留小规模完整性测试相关文件
-   **维护性提升**: 减少了代码维护负担，避免重复测试文件
-   **统一测试入口**: 使用单一测试文件支持多环境

---

_清理完成时间: 2025 年 7 月 20 日_
