#!/bin/bash

# Set network proxy
export https_proxy=http://127.0.0.1:7890
export http_proxy=http://127.0.0.1:7890

# Buffer account list (only includes actually existing buffer accounts)
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

echo "🧹 Starting Buffer account cleanup..."
echo "Total accounts to clean: ${#BUFFERS[@]}"

# Counters
success_count=0
fail_count=0

# Batch close buffer accounts
for buffer in "${BUFFERS[@]}"; do
    echo "🗑️ Closing Buffer account: $buffer"

    if solana program close "$buffer" --bypass-warning; then
        echo "✅ Successfully closed: $buffer"
        ((success_count++))
    else
        echo "❌ Failed to close: $buffer"
        ((fail_count++))
    fi

    # Add small delay to avoid network congestion
    sleep 0.5
done

echo ""
echo "🎉 Buffer account cleanup completed!"
echo "✅ Successfully closed: $success_count accounts"
echo "❌ Failed: $fail_count accounts"
echo ""

# Show status after cleanup
echo "📊 Buffer status after cleanup:"
solana program show --buffers
