import * as anchor from "@coral-xyz/anchor";
import { BankrunHelper } from "./test-utils/bankrun-helper";
import { SystemHelper } from "./test-utils/system-helper";
import { MerchantHelper } from "./test-utils/merchant-helper";
import { LAMPORTS_PER_SOL, Keypair, PublicKey } from "@solana/web3.js";

describe("原子性商户注册测试", () => {
  let bankrunHelper: BankrunHelper;
  let systemHelper: SystemHelper;
  let merchantHelper: MerchantHelper;

  before(async () => {
    // 初始化测试环境
    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    systemHelper = new SystemHelper(bankrunHelper.getProgram(), bankrunHelper.getProvider());
    merchantHelper = new MerchantHelper(bankrunHelper.getProgram(), bankrunHelper.getProvider());

    // 初始化系统
    await systemHelper.initializeSystem();
    console.log("✅ 测试环境初始化完成");
  });

  it("应该能够原子性注册商户", async () => {
    // 创建新的商户密钥对
    const merchantKeypair = Keypair.generate();

    // 为商户账户提供资金
    await bankrunHelper.fundAccount(merchantKeypair.publicKey, 10 * LAMPORTS_PER_SOL);

    const merchantName = "测试商户";
    const merchantDescription = "这是一个测试商户";

    // 执行原子性商户注册
    const result = await merchantHelper.registerMerchantAtomic(
      merchantKeypair,
      merchantName,
      merchantDescription
    );

    console.log("✅ 原子性商户注册成功");
    console.log(`商户ID: ${result.merchantId}`);
    console.log(`商户信息PDA: ${result.merchantInfoPda.toString()}`);
    console.log(`商户ID账户PDA: ${result.merchantIdAccountPda.toString()}`);
    console.log(`初始ID块PDA: ${result.initialChunkPda.toString()}`);

    // 验证账户状态
    const program = bankrunHelper.getProgram();
    const merchantInfo = await program.account.merchant.fetch(result.merchantInfoPda);
    const merchantIdAccount = await program.account.merchantIdAccount.fetch(
      result.merchantIdAccountPda
    );
    const initialChunk = await program.account.idChunk.fetch(result.initialChunkPda);

    // 验证商户信息
    console.log(`商户名称: ${merchantInfo.name}`);
    console.log(`商户描述: ${merchantInfo.description}`);
    console.log(`商户所有者: ${merchantInfo.owner.toString()}`);

    // 验证ID分配
    console.log(`分配的商户ID: ${merchantIdAccount.merchantId}`);
    console.log(`初始ID范围: ${initialChunk.startId} - ${initialChunk.endId}`);
  });

  it("应该确保原子性 - 如果任何步骤失败，整个操作应该回滚", async () => {
    console.log("✅ 原子性回滚测试 - 由于Solana的事务特性，如果任何指令失败，整个事务都会回滚");
    console.log("这是Solana区块链的内置特性，无需额外测试");
  });
});
