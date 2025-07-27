import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import fs from "fs";
import path from "path";

interface TokenConfig {
  symbol: string;
  name: string;
  mint: string;
  decimals: number;
  totalSupply: string;
  description: string;
  environment: string;
}

class RealSPLTokenCreator {
  private connection: Connection;
  private payer: Keypair;
  private programId: PublicKey;
  private program: anchor.Program<any>;

  constructor() {
    // 检测环境
    const isLocal = process.argv.includes("--local");

    if (isLocal) {
      this.connection = new Connection("http://localhost:8899", "confirmed");
      console.log("🌐 环境: 本地测试环境 (localhost:8899)");
    } else {
      this.connection = new Connection("https://api.devnet.solana.com", "confirmed");
      console.log("🌐 环境: Devnet环境");
    }

    // 加载钱包
    const walletPath = path.join(process.env.HOME!, ".config/solana/id.json");
    const walletKeypair = Keypair.fromSecretKey(
      new Uint8Array(JSON.parse(fs.readFileSync(walletPath, "utf8")))
    );
    this.payer = walletKeypair;

    // 设置程序
    this.programId = new PublicKey("mo5xPstZDm27CAkcyoTJnEovMYcW45tViAU6PZikv5q");

    const provider = new anchor.AnchorProvider(this.connection, new anchor.Wallet(this.payer), {
      commitment: "confirmed",
    });
    anchor.setProvider(provider);

    // 加载程序IDL
    const idl = JSON.parse(fs.readFileSync("target/idl/solana_e_commerce.json", "utf8"));
    this.program = new anchor.Program(idl, provider);
  }

  async createRealSPLToken(): Promise<TokenConfig> {
    console.log("🚀 开始创建真实SPL Token...");

    try {
      // 1. 创建DXDV Token Mint
      console.log("🪙 创建DXDV Token Mint...");
      const dxdvMint = await createMint(
        this.connection,
        this.payer,
        this.payer.publicKey, // mint authority
        null, // freeze authority
        9 // decimals
      );
      console.log("✅ DXDV Mint创建成功:", dxdvMint.toString());

      // 2. 创建主钱包的Token账户
      console.log("📦 创建主钱包Token账户...");
      const payerTokenAccount = await getOrCreateAssociatedTokenAccount(
        this.connection,
        this.payer,
        dxdvMint,
        this.payer.publicKey
      );
      console.log("✅ 主钱包Token账户:", payerTokenAccount.address.toString());

      // 3. 铸造初始供应量
      const initialSupply = 1_000_000_000; // 10亿 DXDV
      const mintAmount = initialSupply * Math.pow(10, 9); // 考虑9位小数

      console.log("⚡ 铸造初始供应量...");
      await mintTo(
        this.connection,
        this.payer,
        dxdvMint,
        payerTokenAccount.address,
        this.payer.publicKey,
        mintAmount
      );
      console.log(`✅ 成功铸造 ${initialSupply.toLocaleString()} DXDV`);

      // 4. 验证余额
      const accountInfo = await getAccount(this.connection, payerTokenAccount.address);
      const balance = Number(accountInfo.amount) / Math.pow(10, 9);
      console.log(`📊 主钱包DXDV余额: ${balance.toLocaleString()} DXDV`);

      // 5. 创建Token配置
      const tokenConfig: TokenConfig = {
        symbol: "DXDV",
        name: "DXDV Token",
        mint: dxdvMint.toString(),
        decimals: 9,
        totalSupply: initialSupply.toString(),
        description: "Real SPL Token for local testing",
        environment: process.argv.includes("--local") ? "local" : "devnet",
      };

      // 6. 保存Token配置
      const configPath = process.argv.includes("--local")
        ? "scripts/spl-tokens-local.json"
        : "scripts/spl-tokens-devnet.json";

      fs.writeFileSync(configPath, JSON.stringify([tokenConfig], null, 2));
      console.log(`📄 Token配置已保存到: ${configPath}`);

      return tokenConfig;
    } catch (error) {
      console.error("❌ 创建SPL Token失败:", error);
      throw error;
    }
  }

  async addTokenToPaymentSystem(tokenConfig: TokenConfig): Promise<string> {
    console.log("💳 将SPL Token添加到支付系统...");

    try {
      const mint = new PublicKey(tokenConfig.mint);

      // 计算PaymentConfig PDA
      const [paymentConfigPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("payment_config")],
        this.programId
      );

      // 检查PaymentConfig是否存在
      const paymentConfigInfo = await this.connection.getAccountInfo(paymentConfigPda);

      if (!paymentConfigInfo) {
        console.log("🔧 PaymentConfig不存在，先初始化...");

        // 创建支持的Token列表
        const supportedTokens = [
          {
            mint: mint,
            symbol: tokenConfig.symbol,
            isActive: true,
          },
        ];

        const tx = await this.program.methods
          .initializePaymentSystem(
            supportedTokens,
            new anchor.BN(1000 * Math.pow(10, tokenConfig.decimals)) // merchant_deposit_required
          )
          .accounts({
            paymentConfig: paymentConfigPda,
            authority: this.payer.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();

        console.log("✅ PaymentConfig初始化完成:", tx);
        return tx;
      } else {
        // PaymentConfig已存在，更新支持的Token
        console.log("🔄 更新支付系统中的Token...");

        // 获取当前配置
        const currentConfig = await this.program.account.paymentConfig.fetch(paymentConfigPda);
        const currentTokens = currentConfig.supportedTokens || [];

        // 添加新Token
        const newToken = {
          mint: mint,
          symbol: tokenConfig.symbol,
          isActive: true,
        };

        const updatedTokens = [...currentTokens, newToken];

        const updateTx = await this.program.methods
          .updateSupportedTokens(updatedTokens)
          .accounts({
            paymentConfig: paymentConfigPda,
            authority: this.payer.publicKey,
          })
          .rpc();

        console.log("✅ Token更新完成:", updateTx);
        return updateTx;
      }
    } catch (error) {
      console.error("❌ 添加Token到支付系统失败:", error);
      throw error;
    }
  }

  async verifyTokenInPaymentSystem(tokenConfig: TokenConfig): Promise<void> {
    console.log("🔍 验证Token在支付系统中的状态...");

    try {
      const [paymentConfigPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("payment_config")],
        this.programId
      );

      const paymentConfig = await this.program.account.paymentConfig.fetch(paymentConfigPda);
      console.log("📊 支付系统状态:");
      console.log(`   支持的代币数量: ${paymentConfig.supportedTokens.length}`);

      paymentConfig.supportedTokens.forEach((token: any, index: number) => {
        const isTarget = token.mint.toString() === tokenConfig.mint;
        const status = token.isActive ? "✅活跃" : "❌停用";
        console.log(
          `   [${index + 1}] ${token.symbol}: ${token.mint.toString()} (${status})${
            isTarget ? " ⭐目标Token" : ""
          }`
        );
      });
    } catch (error) {
      console.error("❌ 验证Token状态失败:", error);
      throw error;
    }
  }
}

async function main() {
  console.log("🚀 真实SPL Token创建和配置系统");
  console.log("================================================================================");

  const creator = new RealSPLTokenCreator();

  try {
    // 1. 创建真实SPL Token
    const tokenConfig = await creator.createRealSPLToken();
    console.log("✅ SPL Token创建完成");

    // 2. 添加到支付系统
    const addTokenTx = await creator.addTokenToPaymentSystem(tokenConfig);
    console.log("✅ Token添加到支付系统完成");

    // 3. 验证配置
    await creator.verifyTokenInPaymentSystem(tokenConfig);
    console.log("✅ Token配置验证完成");

    console.log("================================================================================");
    console.log("🎉 真实SPL Token系统配置完成！");
    console.log(`📋 Token信息:`);
    console.log(`   符号: ${tokenConfig.symbol}`);
    console.log(`   Mint: ${tokenConfig.mint}`);
    console.log(`   精度: ${tokenConfig.decimals}`);
    console.log(`   总供应量: ${tokenConfig.totalSupply}`);
    console.log(`   环境: ${tokenConfig.environment}`);
    console.log(`📝 添加交易: ${addTokenTx}`);
  } catch (error) {
    console.error("❌ 系统配置失败:", error);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
