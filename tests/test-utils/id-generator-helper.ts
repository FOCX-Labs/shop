import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

export interface IdChunkInfo {
  merchantId: number;
  chunkIndex: number;
  startId: number;
  endId: number;
  nextAvailable: number;
  utilization: number;
}

export interface MerchantIdInfo {
  merchantId: number;
  lastChunkIndex: number;
  lastLocalId: number;
  activeChunk: PublicKey;
  totalChunks: number;
}

export class IdGeneratorHelper {
  private program: Program<SolanaECommerce>;

  constructor(program: Program<SolanaECommerce>, private provider: AnchorProvider) {
    // 重新创建program实例以确保使用正确的provider
    this.program = new Program(program.idl, provider) as Program<SolanaECommerce>;
  }

  /**
   * 注册商户并分配初始ID范围
   */
  async registerMerchant(merchant: Keypair): Promise<{
    signature: string;
    merchantId: number;
    merchantAccountPda: PublicKey;
    initialChunkPda: PublicKey;
    idRange: { start: number; end: number };
  }> {
    const [globalIdRootPda] = this.getGlobalIdRootPda();
    const [merchantAccountPda] = this.getMerchantIdAccountPda(merchant.publicKey);
    const [initialChunkPda] = this.getIdChunkPda(merchant.publicKey, 0);

    const signature = await this.program.methods
      .registerMerchant()
      .accounts({
        globalRoot: globalIdRootPda,
        merchantAccount: merchantAccountPda,
        initialChunk: initialChunkPda,
        payer: merchant.publicKey,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });

    // 获取分配的商户信息
    const merchantAccount = await this.getMerchantIdAccount(merchant);
    const initialChunk = await this.getIdChunk(merchant.publicKey, 0);

    return {
      signature,
      merchantId: merchantAccount.merchantId,
      merchantAccountPda,
      initialChunkPda,
      idRange: {
        start: initialChunk.startId,
        end: initialChunk.endId,
      },
    };
  }

  /**
   * 生成单个产品ID
   */
  async generateProductId(merchant: Keypair): Promise<{
    signature: string;
    productId: number;
    localId: number;
    chunkIndex: number;
  }> {
    const merchantAccountBefore = await this.getMerchantIdAccount(merchant);
    const [merchantAccountPda] = this.getMerchantIdAccountPda(merchant.publicKey);
    const [activeChunkPda] = this.getIdChunkPda(
      merchant.publicKey,
      merchantAccountBefore.lastChunkIndex
    );

    // 预计生成的产品ID
    const predictedProductId =
      merchantAccountBefore.merchantId * 10000 + (merchantAccountBefore.lastLocalId + 1);

    // 直接执行交易（不再依赖日志提取）
    const signature = await this.program.methods
      .generateProductId()
      .accounts({
        merchantAccount: merchantAccountPda,
        merchant: merchant.publicKey,
        activeChunk: activeChunkPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });

    // 获取更新后的账户状态来验证生成的ID
    const updatedMerchantAccount = await this.getMerchantIdAccount(merchant);
    const actualProductId =
      merchantAccountBefore.merchantId * 10000 + updatedMerchantAccount.lastLocalId;

    return {
      signature,
      productId: actualProductId,
      localId: updatedMerchantAccount.lastLocalId,
      chunkIndex: updatedMerchantAccount.lastChunkIndex,
    };
  }

  /**
   * 分配新的ID块
   */
  async allocateNewChunk(
    merchant: Keypair,
    payer?: Keypair
  ): Promise<{
    signature: string;
    chunkIndex: number;
    chunkPda: PublicKey;
    idRange: { start: number; end: number };
  }> {
    const payerAccount = payer || merchant;
    const merchantAccount = await this.getMerchantIdAccount(merchant);
    const newChunkIndex = merchantAccount.lastChunkIndex + 1;

    const [globalIdRootPda] = this.getGlobalIdRootPda();
    const [merchantAccountPda] = this.getMerchantIdAccountPda(merchant.publicKey);
    const [newChunkPda] = this.getIdChunkPda(merchant.publicKey, newChunkIndex);

    const signature = await this.program.methods
      .allocateNewChunk()
      .accounts({
        globalRoot: globalIdRootPda,
        merchantAccount: merchantAccountPda,
        merchant: merchant.publicKey,
        newChunk: newChunkPda,
        payer: payerAccount.publicKey,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([merchant, payerAccount])
      .rpc({ commitment: "confirmed" });

    const newChunk = await this.getIdChunk(merchant.publicKey, newChunkIndex);

    return {
      signature,
      chunkIndex: newChunkIndex,
      chunkPda: newChunkPda,
      idRange: {
        start: newChunk.startId,
        end: newChunk.endId,
      },
    };
  }

  /**
   * 验证ID是否存在
   */
  async isIdExists(
    merchant: Keypair,
    id: number
  ): Promise<{
    exists: boolean;
    chunkIndex?: number;
    localId?: number;
  }> {
    try {
      // 计算ID应该在哪个块中
      const merchantAccount = await this.getMerchantIdAccount(merchant);
      const chunkIndex = this.calculateChunkIndexForId(id, merchantAccount.merchantId);
      const localId = this.calculateLocalIdFromGlobalId(id, merchantAccount.merchantId);

      // 获取ID块信息
      const [idChunkPda] = this.getIdChunkPda(merchant.publicKey, chunkIndex);

      try {
        const chunk = await this.program.account.idChunk.fetch(idChunkPda);

        // 检查bitmap中的位
        const byteIndex = Math.floor(localId / 8);
        const bitIndex = localId % 8;

        if (byteIndex < chunk.bitmap.length) {
          const byte = chunk.bitmap[byteIndex];
          const exists = (byte & (1 << bitIndex)) !== 0;

          return {
            exists,
            chunkIndex: exists ? chunkIndex : undefined,
            localId: exists ? localId : undefined,
          };
        }

        return { exists: false };
      } catch (chunkError) {
        // 如果ID块不存在，ID也不存在
        return { exists: false };
      }
    } catch (error) {
      return { exists: false };
    }
  }

  /**
   * 批量生成ID
   */
  async batchGenerateIds(
    merchant: Keypair,
    count: number
  ): Promise<{
    signature: string;
    productIds: number[];
    chunksUsed: number[];
  }> {
    const merchantAccountBefore = await this.getMerchantIdAccount(merchant);
    const [merchantAccountPda] = this.getMerchantIdAccountPda(merchant.publicKey);
    const [activeChunkPda] = this.getIdChunkPda(
      merchant.publicKey,
      merchantAccountBefore.lastChunkIndex
    );

    // 预计生成的产品ID范围
    const startLocalId = merchantAccountBefore.lastLocalId + 1;
    const predictedProductIds = [];
    for (let i = 0; i < count; i++) {
      const localId = startLocalId + i;
      const globalId = merchantAccountBefore.merchantId * 10000 + localId;
      predictedProductIds.push(globalId);
    }

    // 直接执行交易（不再依赖日志提取）
    const signature = await this.program.methods
      .batchGenerateIds(count)
      .accounts({
        merchantAccount: merchantAccountPda,
        merchant: merchant.publicKey,
        activeChunk: activeChunkPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });

    // 获取更新后的账户状态来验证生成的ID
    const updatedMerchantAccount = await this.getMerchantIdAccount(merchant);
    const chunksUsed = [updatedMerchantAccount.lastChunkIndex];

    // 计算实际生成的产品ID
    const actualProductIds = [];
    for (let i = 0; i < count; i++) {
      const localId = startLocalId + i;
      const globalId = merchantAccountBefore.merchantId * 10000 + localId;
      actualProductIds.push(globalId);
    }

    return {
      signature,
      productIds: actualProductIds,
      chunksUsed,
    };
  }

  /**
   * 获取商户ID账户信息
   */
  async getMerchantIdAccount(merchant: Keypair): Promise<MerchantIdInfo> {
    const [merchantAccountPda] = this.getMerchantIdAccountPda(merchant.publicKey);

    try {
      const account = await this.program.account.merchantIdAccount.fetch(merchantAccountPda);

      // 正确处理BN类型转换
      const merchantId =
        typeof account.merchantId === "object" &&
        account.merchantId &&
        "toNumber" in account.merchantId
          ? (account.merchantId as any).toNumber()
          : account.merchantId;
      const lastLocalId =
        typeof account.lastLocalId === "object" &&
        account.lastLocalId &&
        "toNumber" in account.lastLocalId
          ? (account.lastLocalId as any).toNumber()
          : account.lastLocalId;

      return {
        merchantId: merchantId,
        lastChunkIndex: account.lastChunkIndex,
        lastLocalId: lastLocalId,
        activeChunk: account.activeChunk,
        totalChunks: account.lastChunkIndex + 1,
      };
    } catch (error) {
      throw new Error(`Failed to fetch merchant ID account: ${error}`);
    }
  }

  /**
   * 获取ID块信息
   */
  async getIdChunk(merchantKey: PublicKey, chunkIndex: number): Promise<IdChunkInfo> {
    const [chunkPda] = this.getIdChunkPda(merchantKey, chunkIndex);

    try {
      const chunk = await this.program.account.idChunk.fetch(chunkPda);
      const used = this.countUsedIds(chunk.bitmap);

      // 正确处理BN类型转换
      const startId =
        typeof chunk.startId === "object" && chunk.startId && "toNumber" in chunk.startId
          ? (chunk.startId as any).toNumber()
          : chunk.startId;
      const endId =
        typeof chunk.endId === "object" && chunk.endId && "toNumber" in chunk.endId
          ? (chunk.endId as any).toNumber()
          : chunk.endId;
      const merchantId =
        typeof chunk.merchantId === "object" && chunk.merchantId && "toNumber" in chunk.merchantId
          ? (chunk.merchantId as any).toNumber()
          : chunk.merchantId;
      const nextAvailable =
        typeof chunk.nextAvailable === "object" &&
        chunk.nextAvailable &&
        "toNumber" in chunk.nextAvailable
          ? (chunk.nextAvailable as any).toNumber()
          : chunk.nextAvailable;

      const total = endId - startId + 1;

      return {
        merchantId: merchantId,
        chunkIndex: chunk.chunkIndex,
        startId: startId,
        endId: endId,
        nextAvailable: nextAvailable,
        utilization: (used / total) * 100,
      };
    } catch (error) {
      throw new Error(`Failed to fetch ID chunk: ${error}`);
    }
  }

  /**
   * 获取全局ID根PDA
   */
  getGlobalIdRootPda(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      this.program.programId
    );
  }

  /**
   * 获取商户ID账户PDA
   */
  getMerchantIdAccountPda(merchantKey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchantKey.toBuffer()],
      this.program.programId
    );
  }

  /**
   * 获取ID块PDA
   * 注意：register_merchant时initial_chunk(chunkIndex=0)使用payer.key()，其他时候使用merchant_id
   */
  getIdChunkPda(merchantKey: PublicKey, chunkIndex: number): [PublicKey, number] {
    const chunkIndexBuffer = Buffer.alloc(4);
    chunkIndexBuffer.writeUInt32LE(chunkIndex, 0);

    // 所有chunks都使用merchantKey作为第二个seed
    return PublicKey.findProgramAddressSync(
      [Buffer.from("id_chunk"), merchantKey.toBuffer(), chunkIndexBuffer],
      this.program.programId
    );
  }

  /**
   * 检查商户是否已注册ID系统
   */
  async isMerchantRegisteredForIds(merchant: Keypair): Promise<boolean> {
    try {
      await this.getMerchantIdAccount(merchant);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * 计算ID应该属于哪个块
   */
  calculateChunkIndexForId(globalId: number, merchantId: number): number {
    const merchantStartId = merchantId * 10000;
    const offsetInMerchant = globalId - merchantStartId;
    const chunkSize = 1000; // 每个块有1000个ID
    return Math.floor(offsetInMerchant / chunkSize);
  }

  /**
   * 从全局ID计算本地ID
   */
  calculateLocalIdFromGlobalId(globalId: number, merchantId: number): number {
    const merchantStartId = merchantId * 10000;
    const offsetInMerchant = globalId - merchantStartId;
    const chunkSize = 1000;
    return offsetInMerchant % chunkSize; // 在块内的偏移
  }

  /**
   * 批量创建测试商户
   */
  async batchRegisterMerchants(merchants: Keypair[]): Promise<
    Array<{
      merchant: Keypair;
      success: boolean;
      merchantId?: number;
      error?: string;
      idRange?: { start: number; end: number };
    }>
  > {
    const results = [];

    for (const merchant of merchants) {
      try {
        const result = await this.registerMerchant(merchant);
        results.push({
          merchant,
          success: true,
          merchantId: result.merchantId,
          idRange: result.idRange,
        });
      } catch (error) {
        results.push({
          merchant,
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    return results;
  }

  /**
   * 性能测试：大量ID生成
   */
  async performanceTest(
    merchant: Keypair,
    idsToGenerate: number,
    batchSize: number = 10
  ): Promise<{
    totalTime: number;
    averageTimePerBatch: number;
    idsGenerated: number;
    chunksUsed: number;
  }> {
    const startTime = Date.now();
    let totalIds = 0;
    const chunksUsed = new Set<number>();

    const batches = Math.ceil(idsToGenerate / batchSize);

    for (let i = 0; i < batches; i++) {
      const currentBatchSize = Math.min(batchSize, idsToGenerate - totalIds);

      try {
        const result = await this.batchGenerateIds(merchant, currentBatchSize);
        totalIds += result.productIds.length;
        result.chunksUsed.forEach((chunk) => chunksUsed.add(chunk));

        // 在批次之间添加短暂延时，避免重复交易错误
        if (i < batches - 1) {
          await new Promise((resolve) => setTimeout(resolve, 50));
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        console.warn(`Batch ${i + 1} failed:`, errorMessage);
        // 如果是重复交易错误，尝试重试一次
        if (errorMessage.includes("already been processed")) {
          try {
            await new Promise((resolve) => setTimeout(resolve, 100));
            const result = await this.batchGenerateIds(merchant, currentBatchSize);
            totalIds += result.productIds.length;
            result.chunksUsed.forEach((chunk) => chunksUsed.add(chunk));
          } catch (retryError) {
            console.warn(
              `Batch ${i + 1} retry failed:`,
              retryError instanceof Error ? retryError.message : String(retryError)
            );
            break;
          }
        } else {
          break;
        }
      }
    }

    const totalTime = Date.now() - startTime;

    return {
      totalTime,
      averageTimePerBatch: totalTime / batches,
      idsGenerated: totalIds,
      chunksUsed: chunksUsed.size,
    };
  }

  /**
   * 从交易日志中提取产品ID
   */
  private extractProductIdFromLogs(logs: string[]): number {
    for (const log of logs) {
      const match = log.match(/全局ID[：:]\s*(\d+)/);
      if (match) {
        return parseInt(match[1], 10);
      }
    }
    throw new Error("Product ID not found in transaction logs");
  }

  /**
   * 从交易日志中提取批量产品ID
   */
  private extractBatchProductIdsFromLogs(logs: string[]): number[] {
    const productIds: number[] = [];

    for (const log of logs) {
      const match = log.match(/全局ID[：:]\s*(\d+)/g);
      if (match) {
        match.forEach((m) => {
          const numMatch = m.match(/\d+/);
          if (numMatch) {
            const id = parseInt(numMatch[0], 10);
            productIds.push(id);
          }
        });
      }
    }

    return productIds;
  }

  /**
   * 计算位图中已使用的ID数量
   */
  private countUsedIds(bitmap: number[]): number {
    let count = 0;
    for (const byte of bitmap) {
      // 计算字节中的1的数量
      let n = byte;
      while (n) {
        count += n & 1;
        n >>= 1;
      }
    }
    return count;
  }

  /**
   * 验证ID生成的一致性
   */
  async validateIdConsistency(
    merchant: Keypair,
    generatedIds: number[]
  ): Promise<{
    valid: boolean;
    duplicates: number[];
    invalidIds: number[];
    errors: string[];
  }> {
    const errors: string[] = [];
    const duplicates: number[] = [];
    const invalidIds: number[] = [];

    // 检查重复
    const idSet = new Set<number>();
    const duplicateSet = new Set<number>();

    for (const id of generatedIds) {
      if (idSet.has(id)) {
        duplicateSet.add(id);
      } else {
        idSet.add(id);
      }
    }

    duplicates.push(...Array.from(duplicateSet));

    // 检查ID有效性
    const merchantAccount = await this.getMerchantIdAccount(merchant);
    const merchantStartId = merchantAccount.merchantId * 10000;

    for (const id of generatedIds) {
      if (id < merchantStartId || id >= merchantStartId + 10000) {
        invalidIds.push(id);
      }
    }

    if (duplicates.length > 0) {
      errors.push(`Found ${duplicates.length} duplicate IDs`);
    }

    if (invalidIds.length > 0) {
      errors.push(`Found ${invalidIds.length} invalid IDs`);
    }

    return {
      valid: errors.length === 0,
      duplicates,
      invalidIds,
      errors,
    };
  }
}
