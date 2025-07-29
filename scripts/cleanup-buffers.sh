#!/bin/bash

# 设置网络代理
export https_proxy=http://127.0.0.1:7890
export http_proxy=http://127.0.0.1:7890

# Buffer账户列表（仅包含实际存在的buffer账户）
BUFFERS=(
    "9BL6F5sUDj5XqbE8Xnx8ert8U9rxHUNa4ctGfQyDGzGA"  # 7.214214 SOL
    "CjLQRyEtdtWobwxxczvAgML9eNL6g9jJArbgcNSk1VJD"  # 7.18069464 SOL
    "3eHtqPsyLuucrktHq94aQy4p8MESApRzD2VUJa7xc5vX"  # 7.22457048 SOL
    "AvEENZZV1pkfxCfQSFz7DRumP5nACwLCWjyyKTAqcWqo"  # 7.22457048 SOL
    "BaVpk1JFJpvkRQKUyueDTQmKpHSwKhXsjSS5wRxeSt7e"  # 7.18069464 SOL
    "GmUSXgr2s2LRN35kGe6NiZ9U8XuTK1AN9hhyqZuaeMXg"  # 7.41844824 SOL
    "A2WcE5hwnmMXnihjEox9wfJJrGDNVwyqu2sFiGEXxf4"   # 7.18058328 SOL
    "5F5BcQCKZ6Tw8LmHWAcnKUDq8A6KArhgMVW7Yh2AL1Ax"  # 7.22457048 SOL
)

echo "🧹 开始清理Buffer账户..."
echo "总计需要清理: ${#BUFFERS[@]} 个账户"

# 计数器
success_count=0
fail_count=0

# 批量关闭buffer账户
for buffer in "${BUFFERS[@]}"; do
    echo "🗑️ 关闭Buffer账户: $buffer"
    
    if solana program close "$buffer" --bypass-warning; then
        echo "✅ 成功关闭: $buffer"
        ((success_count++))
    else
        echo "❌ 关闭失败: $buffer"
        ((fail_count++))
    fi
    
    # 添加小延迟避免网络拥堵
    sleep 0.5
done

echo ""
echo "🎉 Buffer账户清理完成!"
echo "✅ 成功关闭: $success_count 个"
echo "❌ 失败: $fail_count 个"
echo ""

# 显示清理后的状态
echo "📊 清理后的Buffer状态:"
solana program show --buffers
