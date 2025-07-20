import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import { PublicKey, Keypair } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

export interface ProductData {
  name: string;
  description: string;
  price: number;
  keywords: string[];
}

export interface ProductUpdates {
  update_name: boolean;
  name: string;
  update_description: boolean;
  description: string;
  update_price: boolean;
  price: number | BN;
  update_keywords: boolean;
  keywords: string[];
  update_is_active: boolean;
  is_active: boolean;
}

export class ProductHelper {
  constructor(private program: Program<SolanaECommerce>, private provider: AnchorProvider) {}

  async createProductWithIndex(
    merchant: Keypair,
    productData: ProductData
  ): Promise<{ productId: number; signature: string }> {
    const [globalRootPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      this.program.programId
    );

    const [merchantIdAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    // 获取商户ID账户以获得当前的active_chunk
    const merchantIdAccount = await this.program.account.merchantIdAccount.fetch(
      merchantIdAccountPda
    );
    const activeChunkPda = merchantIdAccount.activeChunk;

    // 获取当前活跃chunk的信息以预测下一个产品ID
    const activeChunk = await this.program.account.idChunk.fetch(activeChunkPda);

    // 计算下一个产品ID (商户ID * 10000 + 下一个本地ID)
    const nextLocalId = activeChunk.nextAvailable;
    const predictedProductId = merchantIdAccount.merchantId * 10000 + nextLocalId;

    // 基于预测的产品ID生成PDA
    const productIdBytes = new BN(predictedProductId).toArray("le", 8);
    const [productAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), Buffer.from(productIdBytes)],
      this.program.programId
    );

    const signature = await this.program.methods
      .createProductWithIndex(
        productData.name,
        productData.description,
        new BN(productData.price),
        productData.keywords
      )
      .accounts({
        merchant: merchant.publicKey,
        globalRoot: globalRootPda,
        merchantIdAccount: merchantIdAccountPda,
        merchantInfo: merchantInfoPda,
        activeChunk: activeChunkPda,
        productAccount: productAccountPda,
        payer: merchant.publicKey,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });

    // 产品创建成功，使用预测的ID
    return { productId: predictedProductId, signature };
  }

  async createProductWithZeroGasEvent(
    merchant: Keypair,
    productData: ProductData
  ): Promise<{ productId: number; signature: string }> {
    const [globalRootPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global_id_root")],
      this.program.programId
    );

    const [merchantIdAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    // 获取商户ID账户以获得当前的active_chunk
    const merchantIdAccount = await this.program.account.merchantIdAccount.fetch(
      merchantIdAccountPda
    );
    const activeChunkPda = merchantIdAccount.activeChunk;

    // 获取当前活跃chunk的信息以预测下一个产品ID
    const activeChunk = await this.program.account.idChunk.fetch(activeChunkPda);

    // 计算下一个产品ID (商户ID * 10000 + 下一个本地ID)
    const nextLocalId = activeChunk.nextAvailable;
    const predictedProductId = merchantIdAccount.merchantId * 10000 + nextLocalId;

    // 基于预测的产品ID生成PDA
    const productIdBytes = new BN(predictedProductId).toArray("le", 8);
    const [productAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), Buffer.from(productIdBytes)],
      this.program.programId
    );

    const signature = await this.program.methods
      .createProductWithZeroGasEvent(
        productData.name,
        productData.description,
        new BN(productData.price),
        productData.keywords
      )
      .accounts({
        merchant: merchant.publicKey,
        globalRoot: globalRootPda,
        merchantIdAccount: merchantIdAccountPda,
        merchantInfo: merchantInfoPda,
        activeChunk: activeChunkPda,
        productAccount: productAccountPda,
        payer: merchant.publicKey,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });

    // BankrunProvider doesn't support getTransaction, so we'll simulate the product ID extraction
    // In a real application, this would parse the transaction logs
    const productId = this.simulateProductIdExtraction(merchant.publicKey);

    return { productId, signature };
  }

  async updateProduct(
    merchant: Keypair,
    productId: number,
    updates: ProductUpdates
  ): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    // 确保price字段是BN类型
    const processedUpdates = {
      ...updates,
      price: typeof updates.price === "number" ? new BN(updates.price) : updates.price,
    };

    return await this.program.methods
      .updateProduct(new BN(productId), processedUpdates as any)
      .accounts({
        merchant: merchant.publicKey,
        product: productAccountPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async updateProductWithZeroGasEvent(
    merchant: Keypair,
    productId: number,
    updates: ProductUpdates
  ): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    // 确保price字段是BN类型
    const processedUpdates = {
      ...updates,
      price: typeof updates.price === "number" ? new BN(updates.price) : updates.price,
    };

    return await this.program.methods
      .updateProductWithZeroGasEvent(new BN(productId), processedUpdates as any)
      .accounts({
        merchant: merchant.publicKey,
        product: productAccountPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async deleteProduct(
    merchant: Keypair,
    productId: number,
    hardDelete: boolean = false,
    beneficiary?: Keypair
  ): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);
    const [merchantInfoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("merchant_info"), merchant.publicKey.toBuffer()],
      this.program.programId
    );

    // 总是提供 beneficiary 账户，如果没有指定就使用商户自己
    const beneficiaryAccount = beneficiary ? beneficiary.publicKey : merchant.publicKey;

    return await this.program.methods
      .deleteProduct(new BN(productId), hardDelete)
      .accounts({
        merchant: merchant.publicKey,
        merchantInfo: merchantInfoPda,
        product: productAccountPda,
        beneficiary: beneficiaryAccount,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async getProduct(merchant: Keypair, productId: number): Promise<any> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    try {
      const product = await this.program.account.product.fetch(productAccountPda);

      // 转换BN类型的字段为number类型
      if (product && product.id && typeof product.id === "object" && "toNumber" in product.id) {
        (product as any).id = (product.id as any).toNumber();
      }
      if (
        product &&
        product.price &&
        typeof product.price === "object" &&
        "toNumber" in product.price
      ) {
        (product as any).price = (product.price as any).toNumber();
      }
      if (
        product &&
        product.sales &&
        typeof product.sales === "object" &&
        "toNumber" in product.sales
      ) {
        product.sales = (product.sales as any).toNumber();
      }

      return product;
    } catch (error) {
      throw new Error(`Failed to fetch product ${productId}: ${error}`);
    }
  }

  async updateProductStatus(merchant: Keypair, productId: number, status: any): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    return await this.program.methods
      .updateProductStatus(new BN(productId), status)
      .accounts({
        merchant: merchant.publicKey,
        product: productAccountPda,
      } as any)
      .signers([merchant])
      .rpc({ commitment: "confirmed" });
  }

  async updateSalesCount(
    authority: Keypair,
    productId: number,
    salesIncrement: number
  ): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    return await this.program.methods
      .updateSalesCount(new BN(productId), salesIncrement)
      .accounts({
        authority: authority.publicKey,
        product: productAccountPda,
      } as any)
      .signers([authority])
      .rpc({ commitment: "confirmed" });
  }

  /**
   * 更新产品销量 (别名方法，用于工作流程兼容性)
   */
  async updateProductSales(
    authority: Keypair,
    productId: number,
    salesIncrement: number
  ): Promise<string> {
    return await this.updateSalesCount(authority, productId, salesIncrement);
  }

  async purchaseProductWithZeroGasEvent(
    authority: Keypair,
    productId: number,
    quantity: number
  ): Promise<string> {
    const [productAccountPda] = this.getProductAccountPdaById(productId);

    return await this.program.methods
      .purchaseProductWithZeroGasEvent(new BN(productId), quantity)
      .accounts({
        authority: authority.publicKey,
        product: productAccountPda,
      } as any)
      .signers([authority])
      .rpc({ commitment: "confirmed" });
  }

  /**
   * 获取产品账户PDA
   */
  getProductAccountPda(merchantKey: PublicKey, productId: number): [PublicKey, number] {
    const productIdBytes = new BN(productId).toArray("le", 8);
    return PublicKey.findProgramAddressSync(
      [Buffer.from("product"), Buffer.from(productIdBytes)],
      this.program.programId
    );
  }

  /**
   * 仅通过产品ID获取产品账户PDA（用于UpdateSales等不需要商户信息的操作）
   */
  getProductAccountPdaById(productId: number): [PublicKey, number] {
    const productIdBytes = new BN(productId).toArray("le", 8);
    return PublicKey.findProgramAddressSync(
      [Buffer.from("product"), Buffer.from(productIdBytes)],
      this.program.programId
    );
  }

  /**
   * 检查产品是否存在
   */
  async isProductExists(merchant: Keypair, productId: number): Promise<boolean> {
    try {
      await this.getProduct(merchant, productId);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * 批量创建产品
   */
  async batchCreateProducts(
    merchant: Keypair,
    products: ProductData[]
  ): Promise<
    Array<{
      productData: ProductData;
      success: boolean;
      productId?: number;
      signature?: string;
      error?: string;
    }>
  > {
    const results = [];

    for (const productData of products) {
      try {
        const { productId, signature } = await this.createProductWithIndex(merchant, productData);

        results.push({
          productData,
          success: true,
          productId,
          signature,
        });
      } catch (error) {
        results.push({
          productData,
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    return results;
  }

  /**
   * 验证产品数据格式
   */
  validateProductData(data: any): boolean {
    return !!(
      data &&
      data.id !== undefined &&
      data.merchant &&
      data.name &&
      data.description !== undefined &&
      data.price !== undefined &&
      data.keywords &&
      Array.isArray(data.keywords) &&
      data.status !== undefined &&
      data.salesCount !== undefined
    );
  }

  /**
   * 模拟产品ID提取 (用于测试环境)
   */
  private simulateProductIdExtraction(merchantKey: PublicKey): number {
    // 在测试环境中，我们需要模拟产品ID的生成
    // 基于商户的公钥生成一个确定性的产品ID
    const merchantIdString = merchantKey.toString().slice(-8);
    const hashCode = merchantIdString.split("").reduce((a, b) => {
      a = (a << 5) - a + b.charCodeAt(0);
      return a & a;
    }, 0);

    // 确保产品ID在合理范围内
    return Math.abs(hashCode % 90000) + 10000; // 10000-99999范围
  }

  /**
   * 从交易日志中提取产品ID
   */
  private extractProductIdFromLogs(logs: string[]): number {
    for (const log of logs) {
      if (log.includes("Product created with ID:")) {
        const match = log.match(/Product created with ID: (\d+)/);
        if (match) {
          return parseInt(match[1]);
        }
      }
      // 也尝试匹配其他可能的格式
      if (log.includes("productId")) {
        const match = log.match(/productId[:\s]+(\d+)/);
        if (match) {
          return parseInt(match[1]);
        }
      }
    }
    return 0; // 默认返回0，实际使用中需要根据具体日志格式调整
  }

  /**
   * 生成测试产品数据
   */
  generateTestProductData(index: number = 0): ProductData {
    return {
      name: `测试产品 ${index + 1}`,
      description: `这是第 ${index + 1} 个测试产品的描述`,
      price: Math.floor(Math.random() * 100000) + 1000, // 1000-101000之间的随机价格
      keywords: [`产品${index + 1}`, "测试", "电商"],
    };
  }

  /**
   * 等待产品创建完成
   */
  async waitForProductCreation(
    merchant: Keypair,
    productId: number,
    timeoutMs: number = 30000
  ): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      if (await this.isProductExists(merchant, productId)) {
        return;
      }

      // 等待100ms后重试
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    throw new Error(`Product creation timed out after ${timeoutMs}ms`);
  }

  /**
   * 创建测试产品（兼容性方法）
   */
  async createTestProduct(
    merchant: Keypair,
    name: string
  ): Promise<{
    productId: number;
    signature: string;
    productData: ProductData;
  }> {
    const productData: ProductData = {
      name,
      description: `${name} 的详细描述`,
      price: Math.floor(Math.random() * 50000) + 10000, // 10000-60000之间的随机价格
      keywords: [name, "测试产品", "电商"],
    };

    const result = await this.createProductWithIndex(merchant, productData);

    return {
      ...result,
      productData,
    };
  }
}
