import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  createMint, 
  getOrCreateAssociatedTokenAccount, 
  mintTo, 
  getAccount 
} from "@solana/spl-token";
import fs from "fs";
import path from "path";

async function createRealSPLToken() {
  console.log("🚀 创建真实SPL Token系统");
  console.log("================================================================================");
  
  // 检测环境
  const isLocal = process.argv.includes('--local');
  
  const connection = isLocal 
    ? new Connection("http://localhost:8899", "confirmed")
    : new Connection("https://api.devnet.solana.com", "confirmed");
  
  console.log(`🌐 环境: ${isLocal ? '本地测试环境 (localhost:8899)' : 'Devnet环境'}`);

  // 加载钱包
  const walletPath = path.join(process.env.HOME!, ".config/solana/id.json");
  const walletKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath, "utf8")))
  );

  try {
    // 1. 创建DXDV Token Mint
    console.log("🪙 创建DXDV Token Mint...");
    const dxdvMint = await createMint(
      connection,
      walletKeypair,
      walletKeypair.publicKey, // mint authority
      null, // freeze authority
      9 // decimals
    );
    console.log("✅ DXDV Mint创建成功:", dxdvMint.toString());

    // 2. 创建主钱包的Token账户
    console.log("📦 创建主钱包Token账户...");
    const payerTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      walletKeypair,
      dxdvMint,
      walletKeypair.publicKey
    );
    console.log("✅ 主钱包Token账户:", payerTokenAccount.address.toString());

    // 3. 铸造初始供应量
    const initialSupply = 1_000_000_000; // 10亿 DXDV
    const mintAmount = initialSupply * Math.pow(10, 9); // 考虑9位小数
    
    console.log("⚡ 铸造初始供应量...");
    await mintTo(
      connection,
      walletKeypair,
      dxdvMint,
      payerTokenAccount.address,
      walletKeypair.publicKey,
      mintAmount
    );
    console.log(`✅ 成功铸造 ${initialSupply.toLocaleString()} DXDV`);

    // 4. 验证余额
    const accountInfo = await getAccount(connection, payerTokenAccount.address);
    const balance = Number(accountInfo.amount) / Math.pow(10, 9);
    console.log(`📊 主钱包DXDV余额: ${balance.toLocaleString()} DXDV`);

    // 5. 创建Token配置
    const tokenConfig = {
      symbol: "DXDV",
      name: "DXDV Token",
      mint: dxdvMint.toString(),
      decimals: 9,
      totalSupply: initialSupply.toString(),
      description: "Real SPL Token for testing",
      environment: isLocal ? "local" : "devnet"
    };

    // 6. 保存Token配置
    const configPath = isLocal 
      ? "scripts/spl-tokens-local.json"
      : "scripts/spl-tokens-devnet.json";
    
    const tokenData = {
      environment: tokenConfig.environment,
      description: "Real SPL Token configuration",
      created_at: new Date().toISOString(),
      tokens: [tokenConfig]
    };
    
    fs.writeFileSync(configPath, JSON.stringify(tokenData, null, 2));
    console.log(`📄 Token配置已保存到: ${configPath}`);

    console.log("================================================================================");
    console.log("🎉 真实SPL Token创建完成！");
    console.log(`📋 Token信息:`);
    console.log(`   符号: ${tokenConfig.symbol}`);
    console.log(`   Mint: ${tokenConfig.mint}`);
    console.log(`   精度: ${tokenConfig.decimals}`);
    console.log(`   总供应量: ${tokenConfig.totalSupply}`);
    console.log(`   环境: ${tokenConfig.environment}`);
    console.log(`📊 主钱包余额: ${balance.toLocaleString()} DXDV`);
    
    console.log("\n🔧 下一步操作:");
    console.log("1. 运行测试脚本验证SPL Token功能");
    console.log("2. 将Token添加到支付系统");
    console.log("3. 测试完整的电商流程");

  } catch (error) {
    console.error("❌ SPL Token创建失败:", error);
    process.exit(1);
  }
}

if (require.main === module) {
  createRealSPLToken();
}
