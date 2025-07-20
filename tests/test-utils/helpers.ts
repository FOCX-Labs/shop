import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Connection,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

/**
 * 环境管理辅助类
 * 提供统一的环境配置和账户管理功能
 */
export class EnvironmentHelper {
  private provider: anchor.AnchorProvider;
  private program: Program<SolanaECommerce>;
  private connection: Connection;

  constructor() {
    // 配置本地集群提供者
    this.provider = anchor.AnchorProvider.env();
    anchor.setProvider(this.provider);

    // 获取程序实例
    this.program = anchor.workspace.SolanaECommerce as Program<SolanaECommerce>;
    this.connection = this.provider.connection;
  }

  getProgram(): Program<SolanaECommerce> {
    return this.program;
  }

  getProvider(): anchor.AnchorProvider {
    return this.provider;
  }

  getConnection(): Connection {
    return this.connection;
  }

  async createFundedAccount(lamports: number = LAMPORTS_PER_SOL): Promise<Keypair> {
    const account = Keypair.generate();

    // 使用空投为账户提供资金
    const airdropSignature = await this.connection.requestAirdrop(account.publicKey, lamports);

    // 确认空投交易
    await this.connection.confirmTransaction(airdropSignature, "confirmed");

    return account;
  }

  async waitForConfirmation(signature: string): Promise<void> {
    await this.connection.confirmTransaction(signature, "confirmed");
  }

  /**
   * 验证环境是否正确配置
   */
  async verifyEnvironment(): Promise<void> {
    try {
      // 检查连接
      const version = await this.connection.getVersion();
      console.log(`Connected to Solana cluster version: ${version["solana-core"]}`);

      // 检查程序是否部署
      const programAccount = await this.connection.getAccountInfo(this.program.programId);
      if (!programAccount) {
        throw new Error(`Program ${this.program.programId.toString()} not found`);
      }

      // 检查钱包余额
      const balance = await this.connection.getBalance(this.provider.wallet.publicKey);
      if (balance < LAMPORTS_PER_SOL * 0.1) {
        console.warn(`Wallet balance is low: ${balance / LAMPORTS_PER_SOL} SOL`);
      }
    } catch (error) {
      throw new Error(
        `Environment verification failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  /**
   * 创建多个资金充足的测试账户
   */
  async createMultipleFundedAccounts(
    count: number,
    lamports: number = LAMPORTS_PER_SOL
  ): Promise<Keypair[]> {
    const accounts: Keypair[] = [];

    for (let i = 0; i < count; i++) {
      const account = await this.createFundedAccount(lamports);
      accounts.push(account);
    }

    return accounts;
  }
}

/**
 * PDA (Program Derived Address) 辅助类
 */
export class PDAHelper {
  constructor(private programId: PublicKey) {}

  findGlobalRootPDA(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync([Buffer.from("global_root")], this.programId);
  }

  findMerchantPDA(merchantKey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchantKey.toBuffer()],
      this.programId
    );
  }

  findMerchantIdRangePDA(merchantKey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_id_range"), merchantKey.toBuffer()],
      this.programId
    );
  }

  findProductPDA(merchantKey: PublicKey, productId: number): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("product"),
        merchantKey.toBuffer(),
        new BN(productId).toArrayLike(Buffer, "le", 8),
      ],
      this.programId
    );
  }

  findKeywordIndexPDA(keyword: string): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("keyword_index"), Buffer.from(keyword)],
      this.programId
    );
  }

  findPriceIndexPDA(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync([Buffer.from("price_index")], this.programId);
  }

  findSalesIndexPDA(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync([Buffer.from("sales_index")], this.programId);
  }
}

/**
 * 交易辅助类
 */
export class TransactionHelper {
  constructor(private program: Program<SolanaECommerce>, private provider: anchor.AnchorProvider) {}

  async buildAndSendTransaction(
    instructions: anchor.web3.TransactionInstruction[],
    signers: Keypair[] = []
  ): Promise<string> {
    const transaction = new Transaction();
    instructions.forEach((ix) => transaction.add(ix));

    // 获取最新区块哈希
    const { blockhash } = await this.provider.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = this.provider.wallet.publicKey;

    // 添加钱包作为签名者
    const allSigners = [this.provider.wallet.payer, ...signers].filter(
      (signer): signer is Keypair => signer !== undefined
    );

    return await sendAndConfirmTransaction(this.provider.connection, transaction, allSigners, {
      commitment: "confirmed",
    });
  }

  async getTransactionLogs(signature: string): Promise<string[]> {
    const tx = await this.provider.connection.getTransaction(signature, {
      commitment: "confirmed",
    });

    return tx?.meta?.logMessages || [];
  }

  extractEventDataFromLogs(logs: string[], eventName: string): any {
    for (const log of logs) {
      if (log.includes(`Program data: ${eventName}`)) {
        // 提取事件数据（实际实现需要根据具体的事件格式）
        const match = log.match(/Program data: (.+)/);
        if (match) {
          try {
            return JSON.parse(match[1]);
          } catch (e) {
            console.warn("Failed to parse event data:", e);
          }
        }
      }
    }
    return null;
  }
}

/**
 * 性能监控辅助类
 * 提供全面的性能测试和监控功能
 */
export class PerformanceHelper {
  private startTime: number = 0;
  private computeUnits: number = 0;
  private metrics: Map<string, number[]> = new Map();

  startTimer(): void {
    this.startTime = Date.now();
  }

  endTimer(): number {
    return Date.now() - this.startTime;
  }

  getElapsedTime(): number {
    return Date.now() - this.startTime;
  }

  async measureTransactionPerformance(transactionFn: () => Promise<string>): Promise<{
    signature: string;
    executionTime: number;
    computeUnits: number;
  }> {
    this.startTimer();
    const signature = await transactionFn();
    const executionTime = this.endTimer();

    // 获取计算单元使用情况（如果可用）
    // 注意：这需要根据实际的Solana节点配置
    this.computeUnits = 0; // 占位符

    return {
      signature,
      executionTime,
      computeUnits: this.computeUnits,
    };
  }

  /**
   * 记录性能指标
   */
  recordMetric(name: string, value: number): void {
    if (!this.metrics.has(name)) {
      this.metrics.set(name, []);
    }
    this.metrics.get(name)!.push(value);
  }

  /**
   * 获取性能统计
   */
  getMetricStats(name: string): {
    count: number;
    average: number;
    min: number;
    max: number;
    total: number;
  } | null {
    const values = this.metrics.get(name);
    if (!values || values.length === 0) {
      return null;
    }

    const total = values.reduce((sum, val) => sum + val, 0);
    const average = total / values.length;
    const min = Math.min(...values);
    const max = Math.max(...values);

    return {
      count: values.length,
      average,
      min,
      max,
      total,
    };
  }

  /**
   * 清理所有指标
   */
  clearMetrics(): void {
    this.metrics.clear();
  }

  /**
   * 批量性能测试
   */
  async batchPerformanceTest<T>(
    testName: string,
    testFunction: () => Promise<T>,
    iterations: number = 10
  ): Promise<{
    results: T[];
    stats: {
      averageTime: number;
      minTime: number;
      maxTime: number;
      totalTime: number;
    };
  }> {
    const results: T[] = [];
    const times: number[] = [];

    for (let i = 0; i < iterations; i++) {
      this.startTimer();
      const result = await testFunction();
      const time = this.endTimer();

      results.push(result);
      times.push(time);
      this.recordMetric(testName, time);
    }

    const totalTime = times.reduce((sum, time) => sum + time, 0);
    const averageTime = totalTime / times.length;
    const minTime = Math.min(...times);
    const maxTime = Math.max(...times);

    return {
      results,
      stats: {
        averageTime,
        minTime,
        maxTime,
        totalTime,
      },
    };
  }

  assertPerformance(
    executionTime: number,
    maxTime: number,
    computeUnits: number = 0,
    maxComputeUnits: number = 25000
  ): void {
    if (executionTime > maxTime) {
      throw new Error(`Performance test failed: execution time ${executionTime}ms > ${maxTime}ms`);
    }

    if (computeUnits > maxComputeUnits) {
      throw new Error(
        `Performance test failed: compute units ${computeUnits} > ${maxComputeUnits}`
      );
    }
  }
}

/**
 * 数据验证辅助类
 */
export class ValidationHelper {
  static validateMerchantData(merchantData: any): boolean {
    return !!(
      merchantData &&
      merchantData.authority &&
      merchantData.name &&
      merchantData.description !== undefined &&
      merchantData.totalProducts !== undefined &&
      merchantData.totalSales !== undefined
    );
  }

  static validateProductData(productData: any): boolean {
    return !!(
      productData &&
      productData.id !== undefined &&
      productData.merchant &&
      productData.name &&
      productData.description !== undefined &&
      productData.price !== undefined &&
      productData.keywords &&
      Array.isArray(productData.keywords) &&
      productData.status !== undefined &&
      productData.salesCount !== undefined
    );
  }

  static validateSearchResults(results: any[]): boolean {
    return (
      Array.isArray(results) &&
      results.every((result) => typeof result === "object" && result.productId !== undefined)
    );
  }

  static assertAccountExists(account: any, accountName: string): void {
    if (!account) {
      throw new Error(`${accountName} account does not exist`);
    }
  }

  static assertAccountBalance(balance: number, expectedMinimum: number): void {
    if (balance < expectedMinimum) {
      throw new Error(
        `Account balance ${balance} is less than expected minimum ${expectedMinimum}`
      );
    }
  }
}

/**
 * 错误处理辅助类
 */
export class ErrorHelper {
  static async expectError(
    fn: () => Promise<any>,
    expectedErrorCode?: string,
    expectedErrorMessage?: string
  ): Promise<void> {
    let error: any = null;

    try {
      await fn();
    } catch (e) {
      error = e;
    }

    if (!error) {
      throw new Error("Expected function to throw an error");
    }

    if (expectedErrorCode && error.error?.errorCode?.code !== expectedErrorCode) {
      throw new Error(
        `Expected error code ${expectedErrorCode}, got ${error.error?.errorCode?.code}`
      );
    }

    if (expectedErrorMessage && !error.message.includes(expectedErrorMessage)) {
      throw new Error(
        `Expected error message to contain "${expectedErrorMessage}", got "${error.message}"`
      );
    }
  }

  static logError(error: any, context: string): void {
    console.error(`Error in ${context}:`, {
      message: error.message,
      code: error.error?.errorCode?.code,
      logs: error.logs,
    });
  }
}

// 导出常用的测试常量
export const TEST_CONSTANTS = {
  DEFAULT_TIMEOUT: 30000,
  PERFORMANCE_THRESHOLDS: {
    SYSTEM_INIT: 20,
    MERCHANT_REGISTER: 100,
    PRODUCT_CREATE: 50,
    SEARCH_KEYWORD: 100,
    SEARCH_PRICE: 50,
    SEARCH_COMBINED: 200,
  },
  COMPUTE_UNIT_LIMITS: {
    SINGLE_INSTRUCTION: 25000,
    BATCH_OPERATION: 100000,
  },
  TEST_DATA: {
    MERCHANT_NAME: "测试商户",
    MERCHANT_DESCRIPTION: "专业电商测试商户",
    PRODUCT_NAMES: ["智能手机", "笔记本电脑", "无线耳机", "平板电脑"],
    KEYWORDS: ["电子", "数码", "智能", "便携", "高质量"],
    PRICE_RANGES: [
      { min: 0, max: 100000 }, // 1000元以下
      { min: 100000, max: 500000 }, // 1000-5000元
      { min: 500000, max: 1000000 }, // 5000-10000元
    ],
  },
};
