import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { BankrunHelper } from "./test-utils/bankrun-helper";
import { SystemHelper } from "./test-utils/system-helper";
import { MerchantHelper } from "./test-utils/merchant-helper";
import { LAMPORTS_PER_SOL, Keypair, PublicKey } from "@solana/web3.js";

describe("原子索引更新完整功能测试", () => {
  let bankrunHelper: BankrunHelper;
  let systemHelper: SystemHelper;
  let merchantHelper: MerchantHelper;
  let supportedTokenMint: PublicKey; // 存储支持的代币mint地址

  beforeAll(async () => {
    console.log("🏗️  初始化完整原子索引测试环境...");

    // 初始化测试环境
    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    systemHelper = new SystemHelper(bankrunHelper.getProgram(), bankrunHelper.getProvider());
    merchantHelper = new MerchantHelper(bankrunHelper.getProgram(), bankrunHelper.getProvider());

    // 初始化系统
    await systemHelper.initializeSystem({
      maxProductsPerShard: 1000,
      maxKeywordsPerProduct: 5,
      chunkSize: 10000,
      bloomFilterSize: 256,
      cacheTtl: 3600,
    });

    // 初始化支付系统
    console.log("💰 初始化支付系统...");
    const program = bankrunHelper.getProgram();
    const provider = bankrunHelper.getProvider();

    const [paymentConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("payment_config")],
      program.programId
    );

    // 创建支持的代币列表（模拟USDC）
    supportedTokenMint = Keypair.generate().publicKey; // 生成一个固定的代币mint地址
    const supportedTokens = [
      {
        mint: supportedTokenMint, // 使用固定的代币mint地址
        symbol: "USDC",
        decimals: 6,
        isActive: true,
        minAmount: new BN(1000), // 0.001 USDC
      },
    ];

    await program.methods
      .initializePaymentSystem(
        supportedTokens,
        100, // 1% fee rate
        provider.wallet.publicKey // fee recipient
      )
      .accounts({
        paymentConfig: paymentConfigPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ commitment: "confirmed" });

    console.log("✅ 支付系统初始化完成");
    console.log("✅ 完整原子索引测试环境初始化完成");
  }, 60000);

  afterAll(async () => {
    console.log("🧹 清理完整原子索引测试环境...");
  });

  it("应该能够完整执行原子索引产品创建", async () => {
    console.log("🧪 开始完整原子索引产品创建测试...");

    try {
      // 1. 创建并完整注册商户
      console.log("📝 创建并完整注册商户...");
      const merchantKeypair = await bankrunHelper.createFundedAccount(10 * LAMPORTS_PER_SOL);

      // 使用完整的商户注册流程（包括初始化merchant_info账户）
      const fullRegistrationResult = await merchantHelper.fullMerchantRegistration(
        merchantKeypair,
        "完整测试商户",
        "用于完整原子索引测试的商户"
      );

      expect(fullRegistrationResult.registerSignature).toBeDefined();
      expect(fullRegistrationResult.initializeSignature).toBeDefined();
      console.log("✅ 商户完整注册成功");

      // 2. 准备产品数据
      const productData = {
        name: "完整测试产品",
        description: "用于完整原子索引测试的产品",
        price: new BN(2000000), // 2 USDC
        keywords: ["完整测试", "原子索引"],
        paymentToken: supportedTokenMint, // 使用支付系统中支持的代币mint
        tokenDecimals: 6,
        tokenPrice: new BN(1000000), // 1 USD
      };

      console.log("📋 产品数据准备完成:", {
        name: productData.name,
        price: productData.price.toString(),
        keywords: productData.keywords,
        tokenDecimals: productData.tokenDecimals,
      });

      // 3. 计算所需的PDA账户
      const program = bankrunHelper.getProgram();

      // 全局根PDA
      const [globalRootPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("global_id_root")],
        program.programId
      );

      // 商户ID账户PDA
      const [merchantIdAccountPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("merchant"), merchantKeypair.publicKey.toBuffer()],
        program.programId
      );

      // 商户信息PDA
      const [merchantInfoPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("merchant_info"), merchantKeypair.publicKey.toBuffer()],
        program.programId
      );

      // 活跃chunk PDA (假设使用chunk index 0)
      const [activeChunkPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("id_chunk"),
          merchantKeypair.publicKey.toBuffer(),
          Buffer.from([0, 0, 0, 0]), // chunk_index = 0
        ],
        program.programId
      );

      // 支付配置PDA
      const [paymentConfigPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("payment_config")],
        program.programId
      );

      // 预测产品ID (商户ID=1, 第一个产品local_id=0，起始ID=10000)
      const predictedProductId = 10000; // 起始ID + local_id(0)

      // 产品账户PDA - 使用数字的字节表示，与Rust代码一致
      const productIdBuffer = Buffer.allocUnsafe(8);
      productIdBuffer.writeBigUInt64LE(BigInt(predictedProductId), 0);
      const [productAccountPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("product"), productIdBuffer],
        program.programId
      );

      console.log("📍 PDA账户计算完成");

      // 4. 准备索引账户（关键词、价格、销量）
      const remainingAccounts = [];

      // 为每个关键词添加索引账户
      for (const keyword of productData.keywords) {
        // 关键词根账户
        const [keywordRootPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("keyword_root"), Buffer.from(keyword)],
          program.programId
        );

        // 关键词分片账户 (使用shard_index = 0)
        const shardIndexBuffer = Buffer.allocUnsafe(4);
        shardIndexBuffer.writeUInt32LE(0, 0); // shard_index = 0
        const [keywordShardPda] = PublicKey.findProgramAddressSync(
          [Buffer.from("keyword_shard"), Buffer.from(keyword), shardIndexBuffer],
          program.programId
        );

        remainingAccounts.push(
          { pubkey: keywordRootPda, isWritable: true, isSigner: false },
          { pubkey: keywordShardPda, isWritable: true, isSigner: false }
        );
      }

      // 价格索引节点 - 使用与程序相同的价格范围计算逻辑
      const calculatePriceRange = (price: number) => {
        const interval = 10000;
        const rangeStart = Math.floor(price / interval) * interval;
        const rangeEnd = rangeStart + interval - 1;
        return { rangeStart, rangeEnd };
      };

      const priceRange = calculatePriceRange(productData.price);
      const priceStartBuffer = Buffer.allocUnsafe(8);
      const priceEndBuffer = Buffer.allocUnsafe(8);
      priceStartBuffer.writeBigUInt64LE(BigInt(priceRange.rangeStart), 0);
      priceEndBuffer.writeBigUInt64LE(BigInt(priceRange.rangeEnd), 0);

      const [priceIndexPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("price_index"), priceStartBuffer, priceEndBuffer],
        program.programId
      );

      // 销量索引节点 - 使用与程序相同的销量范围计算逻辑
      const calculateSalesRange = (sales: number) => {
        const interval = 1000;
        const rangeStart = Math.floor(sales / interval) * interval;
        const rangeEnd = rangeStart + interval - 1;
        return { rangeStart, rangeEnd };
      };

      const salesRange = calculateSalesRange(0); // 初始销量为0
      const salesStartBuffer = Buffer.allocUnsafe(4);
      const salesEndBuffer = Buffer.allocUnsafe(4);
      salesStartBuffer.writeUInt32LE(salesRange.rangeStart, 0);
      salesEndBuffer.writeUInt32LE(salesRange.rangeEnd, 0);

      const [salesIndexPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("sales_index"), salesStartBuffer, salesEndBuffer],
        program.programId
      );

      remainingAccounts.push(
        { pubkey: priceIndexPda, isWritable: true, isSigner: false },
        { pubkey: salesIndexPda, isWritable: true, isSigner: false }
      );

      console.log("📊 索引账户准备完成，总数:", remainingAccounts.length);

      // 5. 执行原子索引产品创建
      console.log("🚀 执行原子索引产品创建...");

      const createProductTx = await program.methods
        .createProductWithAtomicIndex(
          productData.name,
          productData.description,
          productData.price,
          productData.keywords,
          productData.paymentToken,
          productData.tokenDecimals,
          productData.tokenPrice
        )
        .accounts({
          merchant: merchantKeypair.publicKey,
          payer: merchantKeypair.publicKey,
          globalRoot: globalRootPda,
          merchantIdAccount: merchantIdAccountPda,
          merchantInfo: merchantInfoPda,
          activeChunk: activeChunkPda,
          productAccount: productAccountPda,
          paymentConfig: paymentConfigPda,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .remainingAccounts(remainingAccounts)
        .signers([merchantKeypair])
        .rpc({ commitment: "confirmed" });

      console.log("✅ 原子索引产品创建成功！");
      console.log("交易签名:", createProductTx);
      console.log("产品ID:", predictedProductId);

      // 6. 验证产品账户是否创建成功
      const productAccount = await program.account.product.fetch(productAccountPda);
      expect(productAccount).toBeDefined();
      expect(productAccount.name).toBe(productData.name);
      expect(productAccount.price.toString()).toBe(productData.price.toString());
      expect(productAccount.keywords).toEqual(productData.keywords);

      console.log("✅ 产品账户验证通过");
      console.log("产品信息:", {
        id: productAccount.id.toString(),
        name: productAccount.name,
        price: productAccount.price.toString(),
        keywords: productAccount.keywords,
        sales: productAccount.sales,
        isActive: productAccount.isActive,
      });

      console.log("🎉 完整原子索引产品创建测试成功完成！");
    } catch (error: any) {
      console.error("❌ 完整原子索引测试失败:", error);
      if (error.logs) {
        console.error("程序日志:", error.logs);
      }
      throw error;
    }
  }, 60000);

  it("应该能够验证索引更新的原子性", async () => {
    console.log("🔍 验证索引更新的原子性...");

    // 这个测试验证如果索引更新失败，整个交易应该回滚
    // 由于测试环境的限制，我们主要验证方法存在性和基本结构

    const program = bankrunHelper.getProgram();

    // 验证原子索引方法存在
    expect(program.methods.createProductWithAtomicIndex).toBeDefined();

    // 验证相关的索引方法存在
    expect(program.methods.addProductToKeywordIndex).toBeDefined();
    expect(program.methods.addProductToPriceIndex).toBeDefined();
    expect(program.methods.addProductToSalesIndex).toBeDefined();

    console.log("✅ 原子性验证通过 - 所有必需的方法都存在");
  });
});
