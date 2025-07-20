import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../target/types/solana_e_commerce";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { BankrunHelper } from "./test-utils/bankrun-helper";

describe("托管支付系统测试", () => {
  let program: Program<SolanaECommerce>;
  let provider: anchor.AnchorProvider;
  let bankrunHelper: BankrunHelper;

  // 测试账户
  let merchant: Keypair;
  let buyer: Keypair;
  let paymentTokenMint: PublicKey;

  // 测试数据
  let productId: number;
  const productName = "测试商品";
  const productDescription = "托管支付测试商品";
  const productPrice = 1000000; // 1 USDC (6 decimals)
  const purchaseAmount = 2;

  beforeAll(async () => {
    // 初始化测试环境
    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    program = bankrunHelper.program;
    provider = bankrunHelper.provider;

    // 创建测试账户
    merchant = Keypair.generate();
    buyer = Keypair.generate();

    // 为账户充值
    await bankrunHelper.fundAccount(merchant.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await bankrunHelper.fundAccount(buyer.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    // 使用模拟支付代币（BankrunProvider不支持createMint）
    paymentTokenMint = Keypair.generate().publicKey; // 模拟USDC代币地址
    console.log("✅ 模拟支付代币地址:", paymentTokenMint.toString());
  });

  it("初始化系统", async () => {
    // 初始化全局ID根
    const [globalRootPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      program.programId
    );

    const systemConfig = {
      maxProductsPerShard: 100,
      maxKeywordsPerProduct: 10,
      chunkSize: 10000,
      bloomFilterSize: 256,
      cacheTtl: 3600,
    };

    await program.methods
      .initializeSystem(systemConfig)
      .accounts({
        globalRoot: globalRootPda,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // 初始化支付系统
    const [paymentConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("payment_config")],
      program.programId
    );

    const supportedTokens = [
      {
        mint: paymentTokenMint,
        symbol: "USDC",
        decimals: 6,
        isActive: true,
        minAmount: new BN(1000), // 0.001 USDC
      },
    ];

    await program.methods
      .initializePaymentSystem(supportedTokens, 250, merchant.publicKey)
      .accounts({
        paymentConfig: paymentConfigPda,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // 原子性注册商户
    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      program.programId
    );

    const [merchantIdAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      program.programId
    );

    const [initialChunkPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("id_chunk"), merchant.publicKey.toBuffer(), Buffer.from([0, 0, 0, 0])],
      program.programId
    );

    await program.methods
      .registerMerchantAtomic("测试商户", "托管支付测试商户")
      .accounts({
        merchant: merchant.publicKey,
        payer: provider.wallet.publicKey,
        globalRoot: globalRootPda,
        merchantInfo: merchantInfoPda,
        merchantIdAccount: merchantIdAccountPda,
        initialChunk: initialChunkPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchant])
      .rpc();

    console.log("✅ 系统初始化完成");
  });

  it("创建测试商品", async () => {
    const keywords = ["测试", "商品", "托管"];

    // 计算相关PDA
    const [globalRootPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      program.programId
    );

    const [merchantIdAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      program.programId
    );

    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      program.programId
    );

    const [activeChunkPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("id_chunk"), merchant.publicKey.toBuffer(), Buffer.from([0, 0, 0, 0])],
      program.programId
    );

    // 先设置临时产品ID（商户的第一个产品ID应该是10000）
    const tempProductId = 10000;
    productId = tempProductId; // 设置全局变量

    const productIdBuffer = Buffer.alloc(8);
    productIdBuffer.writeBigUInt64LE(BigInt(productId), 0);
    const [productAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productIdBuffer],
      program.programId
    );

    const [paymentConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("payment_config")],
      program.programId
    );

    // 计算索引账户PDAs
    const remainingAccounts = [];

    // 为每个关键词添加root和shard账户
    for (const keyword of keywords) {
      const [keywordRootPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("keyword_root"), Buffer.from(keyword)],
        program.programId
      );

      // shard索引使用4字节little-endian格式
      const shardIndexBuffer = Buffer.alloc(4);
      shardIndexBuffer.writeUInt32LE(0, 0); // 第一个分片，索引为0
      const [keywordShardPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("keyword_shard"), Buffer.from(keyword), shardIndexBuffer],
        program.programId
      );

      remainingAccounts.push(
        { pubkey: keywordRootPda, isWritable: true, isSigner: false },
        { pubkey: keywordShardPda, isWritable: true, isSigner: false }
      );
    }

    // 添加价格索引账户（需要计算价格范围）
    const interval = 10000;
    const priceStart = Math.floor(productPrice / interval) * interval;
    const priceEnd = priceStart + interval - 1;

    const priceStartBuffer = Buffer.alloc(8);
    priceStartBuffer.writeBigUInt64LE(BigInt(priceStart), 0);
    const priceEndBuffer = Buffer.alloc(8);
    priceEndBuffer.writeBigUInt64LE(BigInt(priceEnd), 0);

    const [priceIndexPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("price_index"), priceStartBuffer, priceEndBuffer],
      program.programId
    );
    remainingAccounts.push({ pubkey: priceIndexPda, isWritable: true, isSigner: false });

    // 添加销量索引账户（新产品销量为0）
    const salesInterval = 1000;
    const initialSales = 0;
    const salesStart = Math.floor(initialSales / salesInterval) * salesInterval;
    const salesEnd = salesStart + salesInterval - 1;

    const salesStartBuffer = Buffer.alloc(4);
    salesStartBuffer.writeUInt32LE(salesStart, 0);
    const salesEndBuffer = Buffer.alloc(4);
    salesEndBuffer.writeUInt32LE(salesEnd, 0);

    const [salesIndexPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("sales_index"), salesStartBuffer, salesEndBuffer],
      program.programId
    );
    remainingAccounts.push({ pubkey: salesIndexPda, isWritable: true, isSigner: false });

    // 创建产品并原子更新索引
    await program.methods
      .createProductWithAtomicIndex(
        productName,
        productDescription,
        new BN(productPrice),
        keywords,
        paymentTokenMint,
        6, // decimals
        new BN(productPrice) // token price
      )
      .accounts({
        merchant: merchant.publicKey,
        payer: provider.wallet.publicKey,
        merchantIdAccount: merchantIdAccountPda,
        merchantInfo: merchantInfoPda,
        activeChunk: activeChunkPda,
        productAccount: productAccountPda,
        paymentConfig: paymentConfigPda,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts(remainingAccounts)
      .signers([merchant])
      .rpc();

    // 验证商品创建成功
    const [productPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productIdBuffer],
      program.programId
    );

    const productAccount = await program.account.product.fetch(productPda);
    expect(productAccount.name).to.equal(productName);
    expect(productAccount.price.toNumber()).to.equal(productPrice);
    expect(productAccount.paymentToken.toString()).to.equal(paymentTokenMint.toString());
  });

  it("托管购买商品", async () => {
    // 在bankrun环境中创建模拟的token账户
    const buyerTokenAccountKeypair = Keypair.generate();
    const buyerTokenAccount = buyerTokenAccountKeypair.publicKey;

    // 创建模拟的token账户数据
    const tokenAccountData = Buffer.alloc(165); // TokenAccount的标准大小
    // 设置mint地址 (offset 0-32)
    paymentTokenMint.toBuffer().copy(tokenAccountData, 0);
    // 设置owner地址 (offset 32-64)
    buyer.publicKey.toBuffer().copy(tokenAccountData, 32);
    // 设置amount (offset 64-72) - 设置足够的余额
    const amount = BigInt(10000000000); // 10,000 tokens with 6 decimals
    tokenAccountData.writeBigUInt64LE(amount, 64);
    // 设置state (offset 108) - 1表示已初始化
    tokenAccountData.writeUInt8(1, 108);

    // 创建模拟的mint账户数据
    const mintAccountData = Buffer.alloc(82); // Mint账户的标准大小
    // 设置mint authority (offset 4-36) - 可以为空
    // 设置supply (offset 36-44) - 设置大量供应
    const supply = BigInt(1000000000000000); // 1B tokens with 6 decimals
    mintAccountData.writeBigUInt64LE(supply, 36);
    // 设置decimals (offset 44)
    mintAccountData.writeUInt8(6, 44);
    // 设置is_initialized (offset 45)
    mintAccountData.writeUInt8(1, 45);

    // 在bankrun中创建mint账户和token账户
    const context = bankrunHelper.getContext();

    // 创建mint账户
    await context.setAccount(paymentTokenMint, {
      lamports: 1461600, // mint账户的rent
      data: mintAccountData,
      owner: TOKEN_PROGRAM_ID,
      executable: false,
    });

    // 创建token账户
    await context.setAccount(buyerTokenAccount, {
      lamports: 2039280, // token账户的rent
      data: tokenAccountData,
      owner: TOKEN_PROGRAM_ID,
      executable: false,
    });

    // 计算PDA
    const productIdBuffer = Buffer.alloc(8);
    productIdBuffer.writeBigUInt64LE(BigInt(productId), 0);
    const [productPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productIdBuffer],
      program.programId
    );

    const [paymentConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("payment_config")],
      program.programId
    );

    const [escrowAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), buyer.publicKey.toBuffer(), productIdBuffer],
      program.programId
    );

    const [escrowTokenAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow_token"), buyer.publicKey.toBuffer(), productIdBuffer],
      program.programId
    );

    // 执行托管购买
    const tx = await program.methods
      .purchaseProductEscrow(new BN(productId), new BN(purchaseAmount))
      .accounts({
        buyer: buyer.publicKey,
        product: productPda,
        paymentConfig: paymentConfigPda,
        escrowAccount: escrowAccountPda,
        escrowTokenAccount: escrowTokenAccountPda,
        buyerTokenAccount: buyerTokenAccount,
        paymentTokenMint: paymentTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([buyer])
      .rpc();

    console.log("托管购买交易签名:", tx);

    // 验证托管账户创建成功
    const escrowAccount = await program.account.escrowAccount.fetch(escrowAccountPda);
    expect(escrowAccount.buyer.toString()).to.equal(buyer.publicKey.toString());
    expect(escrowAccount.merchant.toString()).to.equal(merchant.publicKey.toString());
    expect(escrowAccount.productId.toNumber()).to.equal(productId);
    expect(escrowAccount.amount.toNumber()).to.equal(purchaseAmount);
    expect(escrowAccount.totalPrice.toNumber()).to.equal(productPrice * purchaseAmount);
    expect(escrowAccount.status).to.deep.equal({ pendingConfirmation: {} });

    // 注意：在bankrun环境中，我们无法验证真实的代币转账
    // 但我们可以验证托管账户的创建和状态

    console.log("✅ 托管购买测试通过");
    console.log(`- 订单ID: ${escrowAccount.orderId.toNumber()}`);
    console.log(`- 总价: ${escrowAccount.totalPrice.toNumber()}`);
    console.log(`- 手续费: ${escrowAccount.feeAmount.toNumber()}`);
    console.log(`- 商户应收: ${escrowAccount.merchantAmount.toNumber()}`);
    console.log(`- 订单状态: 待确认收货`);
  });

  it("验证托管账户状态", async () => {
    const productIdBuffer = Buffer.alloc(8);
    productIdBuffer.writeBigUInt64LE(BigInt(productId), 0);
    const [escrowAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), buyer.publicKey.toBuffer(), productIdBuffer],
      program.programId
    );

    const escrowAccount = await program.account.escrowAccount.fetch(escrowAccountPda);

    // 验证所有字段
    expect(escrowAccount.orderId.toNumber()).to.equal(productId);
    expect(escrowAccount.buyer.toString()).to.equal(buyer.publicKey.toString());
    expect(escrowAccount.merchant.toString()).to.equal(merchant.publicKey.toString());
    expect(escrowAccount.productId.toNumber()).to.equal(productId);
    expect(escrowAccount.paymentToken.toString()).to.equal(paymentTokenMint.toString());
    expect(escrowAccount.amount.toNumber()).to.equal(purchaseAmount);
    expect(escrowAccount.totalPrice.toNumber()).to.equal(productPrice * purchaseAmount);
    expect(escrowAccount.status).to.deep.equal({ pendingConfirmation: {} });
    expect(escrowAccount.createdAt.toNumber()).to.be.greaterThan(0);

    console.log("✅ 托管账户状态验证通过");
  });
});
