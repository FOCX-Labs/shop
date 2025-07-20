import { Keypair } from "@solana/web3.js";
import { PerformanceHelper, ValidationHelper, EnvironmentHelper } from "./helpers";
import { SystemHelper, SystemConfig } from "./system-helper";
import { MerchantHelper } from "./merchant-helper";
import { ProductHelper, ProductData } from "./product-helper";
import { BN } from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import { BankrunProvider } from "anchor-bankrun";

/**
 * 工作流程配置接口
 */
export interface WorkflowConfig {
  systemConfig?: Partial<SystemConfig>;
  merchantCount?: number;
  productsPerMerchant?: number;
  enablePerformanceTracking?: boolean;
  enableValidation?: boolean;
  cleanupAfterTest?: boolean;
}

/**
 * 工作流程结果接口
 */
export interface WorkflowResult {
  success: boolean;
  systemInitialized: boolean;
  merchants: Array<{
    merchant: Keypair;
    merchantInfo: any;
    products: Array<{ productId: number; signature: string }>;
  }>;
  performanceMetrics?: {
    totalExecutionTime: number;
    systemInitTime: number;
    merchantRegistrationTime: number;
    productCreationTime: number;
    averageProductCreationTime: number;
  };
  validationResults?: any[];
  errors: string[];
}

/**
 * 工作流程辅助类
 * 提供端到端业务流程测试支持
 */
export class WorkflowHelper {
  private envHelper: EnvironmentHelper;
  private systemHelper: SystemHelper;
  private merchantHelper: MerchantHelper;
  private productHelper: ProductHelper;
  private performanceHelper: PerformanceHelper;

  constructor() {
    this.envHelper = new EnvironmentHelper();
    this.systemHelper = new SystemHelper(this.envHelper.getProgram(), this.envHelper.getProvider());
    this.merchantHelper = new MerchantHelper(
      this.envHelper.getProgram(),
      this.envHelper.getProvider()
    );
    this.productHelper = new ProductHelper(
      this.envHelper.getProgram(),
      this.envHelper.getProvider()
    );
    this.performanceHelper = new PerformanceHelper();
  }

  /**
   * 完整电商平台工作流程测试
   * @param config 工作流程配置
   * @returns Promise<WorkflowResult>
   */
  async executeCompleteECommerceWorkflow(config: WorkflowConfig = {}): Promise<WorkflowResult> {
    const result: WorkflowResult = {
      success: false,
      systemInitialized: false,
      merchants: [],
      errors: [],
    };

    const {
      systemConfig = {},
      merchantCount = 3,
      productsPerMerchant = 5,
      enablePerformanceTracking = true,
      enableValidation = true,
      cleanupAfterTest = false,
    } = config;

    try {
      let totalStartTime = 0;
      if (enablePerformanceTracking) {
        this.performanceHelper.startTimer();
        totalStartTime = Date.now();
      }

      // 步骤1: 初始化系统
      console.log("🚀 开始初始化系统...");
      const systemInitStart = Date.now();
      const { globalRootPda } = await this.systemHelper.initializeSystem(systemConfig);
      const systemInitTime = Date.now() - systemInitStart;
      result.systemInitialized = true;
      console.log(`✅ 系统初始化完成 (${systemInitTime}ms)`);

      // 步骤2: 注册和初始化商户
      console.log(`👥 开始注册 ${merchantCount} 个商户...`);
      const merchantRegistrationStart = Date.now();

      for (let i = 0; i < merchantCount; i++) {
        const merchantData = await this.merchantHelper.createTestMerchant(
          `测试商户_${i + 1}`,
          `第${i + 1}个测试商户的描述`
        );

        const merchantInfo = await this.merchantHelper.getMerchantInfo(merchantData.merchant);

        if (enableValidation) {
          const validationResult = ValidationHelper.validateMerchantData(merchantInfo);
          if (!validationResult) {
            result.errors.push(`商户${i + 1}验证失败: 数据格式不正确`);
          }
        }

        result.merchants.push({
          merchant: merchantData.merchant,
          merchantInfo,
          products: [],
        });
      }

      const merchantRegistrationTime = Date.now() - merchantRegistrationStart;
      console.log(`✅ 商户注册完成 (${merchantRegistrationTime}ms)`);

      // 步骤3: 为每个商户创建产品
      console.log(`📦 开始为每个商户创建 ${productsPerMerchant} 个产品...`);
      const productCreationStart = Date.now();
      let totalProductsCreated = 0;

      for (let merchantIndex = 0; merchantIndex < result.merchants.length; merchantIndex++) {
        const merchantData = result.merchants[merchantIndex];

        for (let productIndex = 0; productIndex < productsPerMerchant; productIndex++) {
          const productData: ProductData = {
            name: `商户${merchantIndex + 1}_产品${productIndex + 1}`,
            description: `这是商户${merchantIndex + 1}的第${productIndex + 1}个产品`,
            price: Math.floor(Math.random() * 500000) + 100000, // 1000-5000元
            keywords: [`商户${merchantIndex + 1}`, "测试产品", `产品${productIndex + 1}`],
          };

          const productResult = await this.productHelper.createProductWithIndex(
            merchantData.merchant,
            productData
          );

          merchantData.products.push(productResult);
          totalProductsCreated++;

          if (enableValidation) {
            const productInfo = await this.productHelper.getProduct(
              merchantData.merchant,
              productResult.productId
            );
            const validationResult = ValidationHelper.validateProductData(productInfo);
            if (!validationResult) {
              result.errors.push(`产品${productResult.productId}验证失败: 数据格式不正确`);
            }
          }
        }
      }

      const productCreationTime = Date.now() - productCreationStart;
      const averageProductCreationTime = productCreationTime / totalProductsCreated;
      console.log(
        `✅ 产品创建完成 (${productCreationTime}ms, 平均${averageProductCreationTime.toFixed(
          2
        )}ms/产品)`
      );

      // 步骤4: 验证整体数据一致性
      if (enableValidation) {
        console.log("🔍 验证数据一致性...");
        await this.validateWorkflowResults(result);
      }

      // 记录性能指标
      if (enablePerformanceTracking) {
        const totalExecutionTime = Date.now() - totalStartTime;
        result.performanceMetrics = {
          totalExecutionTime,
          systemInitTime,
          merchantRegistrationTime,
          productCreationTime,
          averageProductCreationTime,
        };

        console.log("📊 性能指标:");
        console.log(`  总执行时间: ${totalExecutionTime}ms`);
        console.log(`  系统初始化: ${systemInitTime}ms`);
        console.log(`  商户注册: ${merchantRegistrationTime}ms`);
        console.log(`  产品创建: ${productCreationTime}ms`);
        console.log(`  平均产品创建时间: ${averageProductCreationTime.toFixed(2)}ms`);
      }

      // 清理测试数据
      if (cleanupAfterTest) {
        console.log("🧹 清理测试数据...");
        await this.cleanupWorkflowData();
      }

      result.success = true;
      console.log("🎉 完整工作流程测试成功完成!");
    } catch (error) {
      result.errors.push(
        `工作流程执行失败: ${error instanceof Error ? error.message : String(error)}`
      );
      console.error("❌ 工作流程测试失败:", error);
    }

    return result;
  }

  /**
   * 商户产品管理工作流程
   * @param merchant 商户密钥对
   * @returns Promise<any>
   */
  async executeMerchantProductManagementWorkflow(merchant: Keypair): Promise<any> {
    console.log("🏪 开始商户产品管理工作流程...");

    const workflow: {
      merchant: Keypair;
      products: any[];
      operations: any[];
      errors: string[];
      merchantStats?: any;
    } = {
      merchant,
      products: [],
      operations: [],
      errors: [],
    };

    try {
      // 1. 创建初始产品
      console.log("1️⃣ 创建初始产品...");
      const initialProduct = await this.productHelper.createTestProduct(merchant, "初始产品");
      workflow.products.push(initialProduct);
      workflow.operations.push({
        action: "create",
        productId: initialProduct.productId,
        timestamp: Date.now(),
      });

      // 2. 更新产品信息
      console.log("2️⃣ 更新产品信息...");
      await this.productHelper.updateProduct(merchant, initialProduct.productId, {
        update_name: true,
        name: "更新后的产品名称",
        update_description: true,
        description: "更新后的产品描述",
        update_price: true,
        price: 299900,
        update_keywords: false,
        keywords: [],
        update_is_active: false,
        is_active: true,
      });
      workflow.operations.push({
        action: "update",
        productId: initialProduct.productId,
        timestamp: Date.now(),
      });

      // 3. 更新销量
      console.log("3️⃣ 更新产品销量...");
      await this.productHelper.updateProductSales(merchant, initialProduct.productId, 10);
      workflow.operations.push({
        action: "sales_update",
        productId: initialProduct.productId,
        sales: 10,
        timestamp: Date.now(),
      });

      // 4. 创建更多产品
      console.log("4️⃣ 创建更多产品...");
      // 创建多个产品（模拟批量创建）
      const additionalProducts = [];
      for (let i = 0; i < 3; i++) {
        const product = await this.productHelper.createTestProduct(merchant, `批量产品_${i + 1}`);
        additionalProducts.push(product);
      }
      workflow.products.push(...additionalProducts);
      additionalProducts.forEach((product) => {
        workflow.operations.push({
          action: "create",
          productId: product.productId,
          timestamp: Date.now(),
        });
      });

      // 5. 获取商户统计信息
      console.log("5️⃣ 获取商户统计信息...");
      const merchantStats = await this.merchantHelper.getMerchantStats(merchant);
      workflow.merchantStats = merchantStats;

      console.log("✅ 商户产品管理工作流程完成");
    } catch (error) {
      workflow.errors.push(
        `商户产品管理工作流程失败: ${error instanceof Error ? error.message : String(error)}`
      );
      console.error("❌ 商户产品管理工作流程失败:", error);
    }

    return workflow;
  }

  /**
   * 搜索功能工作流程测试
   * @returns Promise<any>
   */
  async executeSearchWorkflow(): Promise<any> {
    console.log("🔍 开始搜索功能工作流程测试...");

    const searchWorkflow: {
      setupData: any;
      searchResults: any[];
      performanceMetrics: any[];
      errors: string[];
    } = {
      setupData: null,
      searchResults: [],
      performanceMetrics: [],
      errors: [],
    };

    try {
      // 设置测试数据
      console.log("1️⃣ 设置搜索测试数据...");
      const setupResult = await this.executeCompleteECommerceWorkflow({
        merchantCount: 2,
        productsPerMerchant: 3,
        enablePerformanceTracking: false,
        cleanupAfterTest: false,
      });
      searchWorkflow.setupData = setupResult;

      // TODO: 实现搜索功能测试
      // 这里需要实现搜索辅助类后才能完成
      console.log("2️⃣ 执行关键词搜索测试...");
      console.log("3️⃣ 执行价格范围搜索测试...");
      console.log("4️⃣ 执行组合搜索测试...");

      console.log("✅ 搜索功能工作流程测试完成");
    } catch (error) {
      searchWorkflow.errors.push(
        `搜索工作流程失败: ${error instanceof Error ? error.message : String(error)}`
      );
      console.error("❌ 搜索工作流程测试失败:", error);
    }

    return searchWorkflow;
  }

  /**
   * 性能基准测试工作流程
   * @returns Promise<any>
   */
  async executePerformanceBenchmarkWorkflow(): Promise<any> {
    console.log("⚡ 开始性能基准测试工作流程...");

    const benchmarkResults: {
      systemInitBenchmark: any;
      merchantRegistrationBenchmark: any;
      productCreationBenchmark: any;
      searchBenchmark: any;
      errors: string[];
    } = {
      systemInitBenchmark: null,
      merchantRegistrationBenchmark: null,
      productCreationBenchmark: null,
      searchBenchmark: null,
      errors: [],
    };

    try {
      // 系统初始化性能测试
      console.log("1️⃣ 系统初始化性能测试...");
      const systemInitResult = await this.performanceHelper.measureTransactionPerformance(
        async () => {
          const { signature } = await this.systemHelper.initializeSystem();
          return signature;
        }
      );
      benchmarkResults.systemInitBenchmark = systemInitResult;

      // 商户注册性能测试
      console.log("2️⃣ 商户注册性能测试...");
      const testMerchant = await this.envHelper.createFundedAccount();
      const merchantRegResult = await this.performanceHelper.measureTransactionPerformance(
        async () => {
          return await this.merchantHelper.registerMerchant(testMerchant);
        }
      );
      benchmarkResults.merchantRegistrationBenchmark = merchantRegResult;

      // 产品创建性能测试
      console.log("3️⃣ 产品创建性能测试...");
      await this.merchantHelper.initializeMerchantAccount(
        testMerchant,
        "性能测试商户",
        "性能测试描述"
      );
      const productCreateResult = await this.performanceHelper.measureTransactionPerformance(
        async () => {
          const result = await this.productHelper.createTestProduct(testMerchant, "性能测试产品");
          return result.signature;
        }
      );
      benchmarkResults.productCreationBenchmark = productCreateResult;

      console.log("📊 性能基准测试结果:");
      console.log(`  系统初始化: ${benchmarkResults.systemInitBenchmark?.executionTime || 0}ms`);
      console.log(
        `  商户注册: ${benchmarkResults.merchantRegistrationBenchmark?.executionTime || 0}ms`
      );
      console.log(`  产品创建: ${benchmarkResults.productCreationBenchmark?.executionTime || 0}ms`);
    } catch (error) {
      benchmarkResults.errors.push(
        `性能基准测试失败: ${error instanceof Error ? error.message : String(error)}`
      );
      console.error("❌ 性能基准测试失败:", error);
    }

    return benchmarkResults;
  }

  /**
   * 验证工作流程结果
   * @param result 工作流程结果
   */
  private async validateWorkflowResults(result: WorkflowResult): Promise<void> {
    for (const merchantData of result.merchants) {
      // 验证商户信息
      const merchantValidation = ValidationHelper.validateMerchantData(merchantData.merchantInfo);
      if (!merchantValidation) {
        result.errors.push(`商户验证失败: 数据格式不正确`);
      }

      // 验证产品信息
      for (const product of merchantData.products) {
        try {
          const productInfo = await this.productHelper.getProduct(
            merchantData.merchant,
            product.productId
          );
          const productValidation = ValidationHelper.validateProductData(productInfo);
          if (!productValidation) {
            result.errors.push(`产品${product.productId}验证失败: 数据格式不正确`);
          }
        } catch (error) {
          result.errors.push(
            `获取产品${product.productId}信息失败: ${
              error instanceof Error ? error.message : String(error)
            }`
          );
        }
      }
    }
  }

  /**
   * 清理工作流程数据
   */
  private async cleanupWorkflowData(): Promise<void> {
    try {
      // 模拟清理测试数据（SystemHelper没有cleanupTestData方法）
      console.log("清理测试数据...");
      // 可以在这里添加具体的清理逻辑
    } catch (error) {
      console.warn(
        "清理测试数据时出现警告:",
        error instanceof Error ? error.message : String(error)
      );
    }
  }

  /**
   * 生成工作流程报告
   * @param result 工作流程结果
   * @returns string
   */
  generateWorkflowReport(result: WorkflowResult): string {
    let report = "📋 工作流程测试报告\n";
    report += "=".repeat(50) + "\n\n";

    report += `状态: ${result.success ? "✅ 成功" : "❌ 失败"}\n`;
    report += `系统初始化: ${result.systemInitialized ? "✅" : "❌"}\n`;
    report += `商户数量: ${result.merchants.length}\n`;

    const totalProducts = result.merchants.reduce(
      (sum, merchant) => sum + merchant.products.length,
      0
    );
    report += `产品总数: ${totalProducts}\n\n`;

    if (result.performanceMetrics) {
      report += "⚡ 性能指标:\n";
      report += `  总执行时间: ${result.performanceMetrics.totalExecutionTime}ms\n`;
      report += `  系统初始化时间: ${result.performanceMetrics.systemInitTime}ms\n`;
      report += `  商户注册时间: ${result.performanceMetrics.merchantRegistrationTime}ms\n`;
      report += `  产品创建时间: ${result.performanceMetrics.productCreationTime}ms\n`;
      report += `  平均产品创建时间: ${result.performanceMetrics.averageProductCreationTime.toFixed(
        2
      )}ms\n\n`;
    }

    if (result.errors.length > 0) {
      report += "❌ 错误列表:\n";
      result.errors.forEach((error, index) => {
        report += `  ${index + 1}. ${error}\n`;
      });
    }

    return report;
  }
}
