import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

export interface MerchantInfo {
  authority: PublicKey;
  name: string;
  description: string;
  totalProducts: number;
  totalSales: number;
  createdAt: number;
  updatedAt: number;
}

export class MerchantHelper {
  constructor(private program: Program<SolanaECommerce>, private provider: AnchorProvider) {}

  async registerMerchant(merchant: Keypair): Promise<string> {
    const [globalRootPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      this.program.programId
    );

    const [merchantAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    const [initialChunkPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("id_chunk"),
        merchant.publicKey.toBuffer(),
        Buffer.from([0, 0, 0, 0]), // chunk index 0
      ],
      this.program.programId
    );

    return await this.program.methods
      .registerMerchant()
      .accounts({
        globalRoot: globalRootPda,
        merchantAccount: merchantAccountPda,
        initialChunk: initialChunkPda,
        payer: merchant.publicKey,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async initializeMerchantAccount(
    merchant: Keypair,
    name: string,
    description: string
  ): Promise<string> {
    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    return await this.program.methods
      .initializeMerchantAccount(name, description)
      .accounts({
        merchantInfo: merchantInfoPda,
        owner: merchant.publicKey,
        payer: merchant.publicKey,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async updateMerchantInfo(
    merchant: Keypair,
    name?: string,
    description?: string
  ): Promise<string> {
    const [merchantAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    return await this.program.methods
      .updateMerchantInfo(name || null, description || null)
      .accounts({
        merchant: merchant.publicKey,
        merchantAccount: merchantAccountPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async getMerchantStats(merchant: Keypair): Promise<any> {
    const [merchantAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    return await this.program.methods
      .getMerchantStats()
      .accounts({
        merchantAccount: merchantAccountPda,
      } as any)
      .view();
  }

  /**
   * 获取商户账户PDA
   */
  getMerchantAccountPda(merchantKey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchantKey.toBuffer()],
      this.program.programId
    );
  }

  /**
   * 获取商户ID范围PDA
   */
  getMerchantIdRangePda(merchantKey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_id_range"), merchantKey.toBuffer()],
      this.program.programId
    );
  }

  /**
   * 检查商户是否已注册
   */
  async isMerchantRegistered(merchant: Keypair): Promise<boolean> {
    try {
      const [merchantAccountPda] = this.getMerchantAccountPda(merchant.publicKey);
      const accountInfo = await this.program.provider.connection.getAccountInfo(merchantAccountPda);
      return accountInfo !== null;
    } catch (error) {
      return false;
    }
  }

  /**
   * 获取商户账户信息
   */
  async getMerchantAccount(merchant: Keypair): Promise<any> {
    const [merchantAccountPda] = this.getMerchantAccountPda(merchant.publicKey);

    try {
      const accountInfo = await this.program.provider.connection.getAccountInfo(merchantAccountPda);
      if (!accountInfo) {
        throw new Error("Merchant account not found");
      }

      // 尝试解码账户数据
      try {
        return this.program.coder.accounts.decode("MerchantAccount", accountInfo.data);
      } catch (decodeError) {
        // 如果解码失败，返回原始账户信息
        return {
          authority: merchantAccountPda,
          name: "Unknown",
          description: "Account data decode failed",
          totalProducts: 0,
          totalSales: 0,
          accountInfo,
        };
      }
    } catch (error) {
      throw new Error(`Failed to fetch merchant account: ${error}`);
    }
  }

  /**
   * 完整商户注册流程
   */
  async fullMerchantRegistration(
    merchant: Keypair,
    name: string,
    description: string
  ): Promise<{
    registerSignature: string;
    initializeSignature: string;
    merchantAccountPda: PublicKey;
  }> {
    // 1. 注册商户
    const registerSignature = await this.registerMerchant(merchant);

    // 2. 初始化商户账户
    const initializeSignature = await this.initializeMerchantAccount(merchant, name, description);

    // 3. 获取PDA
    const [merchantAccountPda] = this.getMerchantAccountPda(merchant.publicKey);

    return {
      registerSignature,
      initializeSignature,
      merchantAccountPda,
    };
  }

  /**
   * 验证商户数据格式
   */
  validateMerchantData(data: any): boolean {
    return !!(
      data &&
      data.authority &&
      data.name &&
      data.description !== undefined &&
      data.totalProducts !== undefined &&
      data.totalSales !== undefined
    );
  }

  /**
   * 批量注册商户
   */
  async batchRegisterMerchants(
    merchants: Array<{
      keypair: Keypair;
      name: string;
      description: string;
    }>
  ): Promise<
    Array<{
      merchant: Keypair;
      success: boolean;
      error?: string;
      signatures?: {
        register: string;
        initialize: string;
      };
    }>
  > {
    const results = [];

    for (const merchantInfo of merchants) {
      try {
        const { registerSignature, initializeSignature } = await this.fullMerchantRegistration(
          merchantInfo.keypair,
          merchantInfo.name,
          merchantInfo.description
        );

        results.push({
          merchant: merchantInfo.keypair,
          success: true,
          signatures: {
            register: registerSignature,
            initialize: initializeSignature,
          },
        });
      } catch (error) {
        results.push({
          merchant: merchantInfo.keypair,
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    return results;
  }

  /**
   * 等待商户注册完成
   */
  async waitForMerchantRegistration(merchant: Keypair, timeoutMs: number = 30000): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      if (await this.isMerchantRegistered(merchant)) {
        return;
      }

      // 等待100ms后重试
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    throw new Error(`Merchant registration timed out after ${timeoutMs}ms`);
  }

  /**
   * 创建测试商户（兼容性方法）
   */
  async createTestMerchant(
    name: string,
    description: string
  ): Promise<{
    merchant: Keypair;
    registerSignature: string;
    initializeSignature: string;
    merchantAccountPda: PublicKey;
  }> {
    const merchant = Keypair.generate();

    // 为商户账户提供资金
    const airdropSignature = await this.provider.connection.requestAirdrop(
      merchant.publicKey,
      1000000000 // 1 SOL
    );
    await this.provider.connection.confirmTransaction(airdropSignature);

    const result = await this.fullMerchantRegistration(merchant, name, description);

    return {
      merchant,
      ...result,
    };
  }

  /**
   * 获取商户信息（兼容性方法）
   */
  async getMerchantInfo(merchant: Keypair): Promise<MerchantInfo | null> {
    try {
      const account = await this.getMerchantAccount(merchant);
      return {
        authority: merchant.publicKey,
        name: account.name || "Unknown",
        description: account.description || "",
        totalProducts: account.totalProducts || 0,
        totalSales: account.totalSales || 0,
        createdAt: account.createdAt || Date.now(),
        updatedAt: account.updatedAt || Date.now(),
      };
    } catch (error) {
      return null;
    }
  }
}
