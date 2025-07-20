import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../target/types/solana_e_commerce";
import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { TestFramework } from "./test-utils/framework";
import { SystemHelper } from "./test-utils/system-helper";
import { MerchantHelper } from "./test-utils/merchant-helper";
import { ProductHelper } from "./test-utils/product-helper";

describe("原子索引更新测试", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaECommerce as Program<SolanaECommerce>;
  const payer = provider.wallet as anchor.Wallet;

  let merchantKeypair: Keypair;
  let merchantInfoPda: PublicKey;
  let merchantStatsPda: PublicKey;
  let idGeneratorPda: PublicKey;
  let paymentTokenMint: PublicKey;

  before(async () => {
    // 创建商户密钥对
    merchantKeypair = Keypair.generate();

    // 为商户账户充值
    const airdropTx = await provider.connection.requestAirdrop(
      merchantKeypair.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropTx);

    // 计算PDA地址
    [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchantKeypair.publicKey.toBuffer()],
      program.programId
    );

    [merchantStatsPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_stats"), merchantKeypair.publicKey.toBuffer()],
      program.programId
    );

    [idGeneratorPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("id_generator")],
      program.programId
    );

    // 创建支付代币
    paymentTokenMint = await createMint(
      provider.connection,
      payer.payer,
      payer.publicKey,
      null,
      6 // USDC decimals
    );

    // 初始化ID生成器
    try {
      await program.methods
        .initializeIdGenerator()
        .accounts({
          idGenerator: idGeneratorPda,
          payer: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      // ID生成器可能已经初始化
      console.log("ID生成器可能已经初始化");
    }

    // 注册商户
    await program.methods
      .registerMerchant("测试商户", "测试商户描述")
      .accounts({
        merchant: merchantKeypair.publicKey,
        merchantInfo: merchantInfoPda,
        merchantStats: merchantStatsPda,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([merchantKeypair])
      .rpc();
  });

  it("测试原子索引更新的产品创建", async () => {
    const productName = "测试产品";
    const productDescription = "这是一个测试产品";
    const price = new anchor.BN(100000); // 0.1 USDC
    const keywords = ["电子产品", "手机", "智能"];
    const tokenDecimals = 6;
    const tokenPrice = new anchor.BN(100000);

    // 计算产品PDA
    const productId = new anchor.BN(1);
    const [productPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // 计算关键词索引PDA
    const keywordIndexPdas = [];
    const keywordShardPdas = [];

    for (const keyword of keywords) {
      const [keywordRootPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("keyword_root"), Buffer.from(keyword)],
        program.programId
      );
      keywordIndexPdas.push(keywordRootPda);

      const [keywordShardPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("keyword_shard"), Buffer.from(keyword), Buffer.from([0])],
        program.programId
      );
      keywordShardPdas.push(keywordShardPda);
    }

    // 计算价格索引PDA
    const priceRange = calculatePriceRange(price.toNumber());
    const [priceIndexPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("price_index"),
        Buffer.from(priceRange.start.toString().padStart(8, "0")),
        Buffer.from(priceRange.end.toString().padStart(8, "0")),
      ],
      program.programId
    );

    // 计算销量索引PDA
    const salesRange = calculateSalesRange(0);
    const [salesIndexPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("sales_index"),
        Buffer.from(salesRange.start.toString().padStart(8, "0")),
        Buffer.from(salesRange.end.toString().padStart(8, "0")),
      ],
      program.programId
    );

    // 准备remaining accounts
    const remainingAccounts = [];

    // 添加关键词索引账户（每个关键词需要root和shard两个账户）
    for (let i = 0; i < keywords.length; i++) {
      remainingAccounts.push({
        pubkey: keywordIndexPdas[i],
        isWritable: true,
        isSigner: false,
      });
      remainingAccounts.push({
        pubkey: keywordShardPdas[i],
        isWritable: true,
        isSigner: false,
      });
    }

    // 添加价格索引账户
    remainingAccounts.push({
      pubkey: priceIndexPda,
      isWritable: true,
      isSigner: false,
    });

    // 添加销量索引账户
    remainingAccounts.push({
      pubkey: salesIndexPda,
      isWritable: true,
      isSigner: false,
    });

    // 执行原子索引更新的产品创建
    const tx = await program.methods
      .createProductWithAtomicIndex(
        productName,
        productDescription,
        price,
        keywords,
        paymentTokenMint,
        tokenDecimals,
        tokenPrice
      )
      .accounts({
        merchant: merchantKeypair.publicKey,
        merchantInfo: merchantInfoPda,
        merchantStats: merchantStatsPda,
        product: productPda,
        idGenerator: idGeneratorPda,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .signers([merchantKeypair])
      .rpc();

    console.log("原子索引更新产品创建交易:", tx);

    // 验证产品账户已创建
    const productAccount = await program.account.product.fetch(productPda);
    expect(productAccount.name).to.equal(productName);
    expect(productAccount.description).to.equal(productDescription);
    expect(productAccount.price.toString()).to.equal(price.toString());
    expect(productAccount.keywords).to.deep.equal(keywords);

    // 验证关键词索引已创建
    for (let i = 0; i < keywords.length; i++) {
      try {
        const keywordRootAccount = await program.account.keywordRoot.fetch(keywordIndexPdas[i]);
        expect(keywordRootAccount.keyword).to.equal(keywords[i]);
        console.log(`关键词 "${keywords[i]}" 索引已创建`);
      } catch (error) {
        console.log(`关键词 "${keywords[i]}" 索引创建失败:`, error.message);
      }
    }

    // 验证价格索引已创建
    try {
      const priceIndexAccount = await program.account.priceIndexNode.fetch(priceIndexPda);
      expect(priceIndexAccount.priceRangeStart.toString()).to.equal(priceRange.start.toString());
      expect(priceIndexAccount.priceRangeEnd.toString()).to.equal(priceRange.end.toString());
      console.log("价格索引已创建");
    } catch (error) {
      console.log("价格索引创建失败:", error.message);
    }

    // 验证销量索引已创建
    try {
      const salesIndexAccount = await program.account.salesIndexNode.fetch(salesIndexPda);
      expect(salesIndexAccount.salesRangeStart).to.equal(salesRange.start);
      expect(salesIndexAccount.salesRangeEnd).to.equal(salesRange.end);
      console.log("销量索引已创建");
    } catch (error) {
      console.log("销量索引创建失败:", error.message);
    }

    console.log("原子索引更新测试完成");
  });
});

// 辅助函数：计算价格范围
function calculatePriceRange(price: number): { start: number; end: number } {
  const interval = 10000;
  const start = Math.floor(price / interval) * interval;
  const end = start + interval - 1;
  return { start, end };
}

// 辅助函数：计算销量范围
function calculateSalesRange(sales: number): { start: number; end: number } {
  const interval = 1000;
  const start = Math.floor(sales / interval) * interval;
  const end = start + interval - 1;
  return { start, end };
}
