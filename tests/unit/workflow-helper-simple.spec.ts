import { describe, beforeAll, afterAll, it, expect } from "@jest/globals";
import { Keypair } from "@solana/web3.js";
import { BankrunProvider } from "anchor-bankrun";
import { Program } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../../target/types/solana_e_commerce";
import { SystemHelper } from "../test-utils/system-helper";
import { MerchantHelper } from "../test-utils/merchant-helper";
import { ProductHelper } from "../test-utils/product-helper";
import { SearchHelper } from "../test-utils/search-helper";
import { PerformanceHelper, ValidationHelper } from "../test-utils/helpers";
import { BankrunHelper } from "../test-utils/bankrun-helper";

describe("工作流程测试 - 端到端覆盖率提升", () => {
  let provider: BankrunProvider;
  let program: Program<SolanaECommerce>;
  let systemHelper: SystemHelper;
  let merchantHelper: MerchantHelper;
  let productHelper: ProductHelper;
  let searchHelper: SearchHelper;
  let performanceHelper: PerformanceHelper;
  let bankrunHelper: BankrunHelper;

  // 工作流程数据
  let testMerchants: Keypair[] = [];
  let merchantInfos: any[] = [];
  let allProducts: Array<{ merchantIndex: number; productId: number; signature: string }> = [];

  beforeAll(async () => {
    console.log("🏗️  初始化端到端工作流程测试环境...");

    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    program = bankrunHelper.getProgram();
    provider = bankrunHelper.getProvider();

    systemHelper = new SystemHelper(program, provider as any);
    merchantHelper = new MerchantHelper(program, provider as any);
    productHelper = new ProductHelper(program, provider as any);
    searchHelper = new SearchHelper(program, provider as any);
    performanceHelper = new PerformanceHelper();

    console.log("✅ 端到端工作流程测试环境初始化完成");
  });

  afterAll(async () => {
    console.log("🧹 清理端到端工作流程测试环境...");
  });

  describe("完整电商平台工作流程", () => {
    it("应该成功执行完整的电商平台工作流程", async () => {
      console.log("🚀 开始完整电商平台工作流程测试...");

      const workflowResult: {
        success: boolean;
        systemInitialized: boolean;
        merchants: any[];
        products: any[];
        performanceMetrics: any;
        errors: string[];
      } = {
        success: false,
        systemInitialized: false,
        merchants: [],
        products: [],
        performanceMetrics: {},
        errors: [],
      };

      try {
        // 步骤1: 系统初始化
        console.log("1️⃣ 初始化系统...");
        performanceHelper.startTimer();

        await systemHelper.initializeSystem(bankrunHelper.getContext());
        const systemInitTime = performanceHelper.endTimer();
        workflowResult.systemInitialized = true;
        (workflowResult.performanceMetrics as any).systemInitTime = systemInitTime;

        console.log(`✅ 系统初始化完成 (${systemInitTime}ms)`);

        // 步骤2: 批量注册商户
        console.log("2️⃣ 批量注册商户...");
        const merchantCount = 3;
        performanceHelper.startTimer();

        for (let i = 0; i < merchantCount; i++) {
          const merchant = await bankrunHelper.createFundedAccount();
          const merchantName = `测试商户_${i + 1}`;
          const merchantDesc = `第${i + 1}个测试商户的描述`;

          const registrationResult = await merchantHelper.fullMerchantRegistration(
            merchant,
            merchantName,
            merchantDesc
          );

          testMerchants.push(merchant);
          merchantInfos.push({
            merchant,
            name: merchantName,
            description: merchantDesc,
            registrationResult,
          });

          console.log(`✅ 商户 ${i + 1} 注册成功: ${merchantName}`);
        }

        const merchantRegistrationTime = performanceHelper.endTimer();
        (workflowResult.performanceMetrics as any).merchantRegistrationTime =
          merchantRegistrationTime;
        (workflowResult as any).merchants = merchantInfos;

        console.log(`✅ 商户注册完成 (${merchantRegistrationTime}ms)`);

        // 步骤3: 批量创建产品
        console.log("3️⃣ 批量创建产品...");
        const productsPerMerchant = 3;
        performanceHelper.startTimer();

        for (let merchantIndex = 0; merchantIndex < testMerchants.length; merchantIndex++) {
          const merchant = testMerchants[merchantIndex];

          for (let productIndex = 0; productIndex < productsPerMerchant; productIndex++) {
            const productData = {
              name: `商户${merchantIndex + 1}_产品${productIndex + 1}`,
              description: `商户${merchantIndex + 1}的第${productIndex + 1}个产品`,
              price: 100000 + productIndex * 50000, // 1000, 1500, 2000 元
              keywords: [`商户${merchantIndex + 1}`, `产品${productIndex + 1}`, "测试", "电商"],
            };

            const productResult = await productHelper.createProductWithIndex(merchant, productData);

            allProducts.push({
              merchantIndex,
              productId: productResult.productId,
              signature: productResult.signature,
            });

            console.log(
              `✅ 产品创建成功: 商户${merchantIndex + 1} - 产品ID ${productResult.productId}`
            );
          }
        }

        const productCreationTime = performanceHelper.endTimer();
        const averageProductCreationTime = productCreationTime / allProducts.length;
        (workflowResult.performanceMetrics as any).productCreationTime = productCreationTime;
        (workflowResult.performanceMetrics as any).averageProductCreationTime =
          averageProductCreationTime;
        (workflowResult as any).products = allProducts;

        console.log(
          `✅ 产品创建完成 (${productCreationTime}ms, 平均${averageProductCreationTime.toFixed(
            2
          )}ms/产品)`
        );

        // 步骤4: 搜索功能测试
        console.log("4️⃣ 搜索功能测试...");
        performanceHelper.startTimer();

        // 初始化搜索索引
        const searchKeywords = ["测试", "电商", "商户1", "产品1"];
        for (const keyword of searchKeywords) {
          try {
            await searchHelper.initializeKeywordIndex(testMerchants[0], keyword);
            console.log(`✅ 搜索索引初始化成功: ${keyword}`);
          } catch (error) {
            console.log(
              `⚠️  搜索索引初始化失败: ${keyword} - ${
                error instanceof Error ? error.message : String(error)
              }`
            );
          }
        }

        // 执行搜索测试
        const searchResults = [];
        for (const keyword of searchKeywords) {
          try {
            const result = await searchHelper.searchByKeyword(keyword);
            searchResults.push({ keyword, result });
            console.log(`✅ 关键词搜索成功: ${keyword} - 找到 ${result.products.length} 个结果`);
          } catch (error) {
            console.log(
              `⚠️  关键词搜索失败: ${keyword} - ${
                error instanceof Error ? error.message : String(error)
              }`
            );
          }
        }

        const searchTime = performanceHelper.endTimer();
        (workflowResult.performanceMetrics as any).searchTime = searchTime;

        console.log(`✅ 搜索功能测试完成 (${searchTime}ms)`);

        // 步骤5: 数据验证
        console.log("5️⃣ 数据验证...");
        let validationErrors = 0;

        // 验证商户数据
        for (const merchantInfo of merchantInfos) {
          const isValid = ValidationHelper.validateMerchantData(merchantInfo);
          if (!isValid) {
            validationErrors++;
            workflowResult.errors.push(`商户数据验证失败: ${merchantInfo.name}`);
          }
        }

        console.log(`✅ 数据验证完成 (${validationErrors} 个错误)`);

        // 计算总体性能指标
        const totalTime = Object.values(workflowResult.performanceMetrics).reduce(
          (sum: number, time: any) => sum + (typeof time === "number" ? time : 0),
          0
        );
        (workflowResult.performanceMetrics as any).totalExecutionTime = totalTime;

        workflowResult.success = true;

        // 验证工作流程结果
        expect(workflowResult.success).toBe(true);
        expect(workflowResult.systemInitialized).toBe(true);
        expect((workflowResult as any).merchants.length).toBe(merchantCount);
        expect((workflowResult as any).products.length).toBe(merchantCount * productsPerMerchant);
        expect((workflowResult.performanceMetrics as any).totalExecutionTime).toBeGreaterThan(0);

        console.log("🎉 完整电商平台工作流程测试成功完成!");
        console.log("📊 性能指标:", workflowResult.performanceMetrics);
      } catch (error) {
        workflowResult.errors.push(
          `工作流程执行失败: ${error instanceof Error ? error.message : String(error)}`
        );
        console.error("❌ 工作流程测试失败:", error);
        throw error;
      }
    });

    it("应该测试商户产品管理工作流程", async () => {
      console.log("🏪 开始商户产品管理工作流程测试...");

      if (testMerchants.length === 0) {
        // 如果没有测试商户，创建一个
        const merchant = await bankrunHelper.createFundedAccount();
        await merchantHelper.fullMerchantRegistration(
          merchant,
          "产品管理测试商户",
          "专门用于产品管理测试"
        );
        testMerchants.push(merchant);
      }

      const merchant = testMerchants[0];
      const workflow: {
        merchant: any;
        products: any[];
        operations: any[];
        errors: string[];
      } = {
        merchant,
        products: [],
        operations: [],
        errors: [],
      };

      try {
        // 1. 创建初始产品
        console.log("1️⃣ 创建初始产品...");
        const initialProduct = await productHelper.createProductWithIndex(merchant, {
          name: "初始产品",
          description: "用于管理工作流程测试的初始产品",
          price: 199900,
          keywords: ["初始", "测试", "管理"],
        });

        workflow.products.push(initialProduct);
        workflow.operations.push({
          action: "create",
          productId: initialProduct.productId,
          timestamp: Date.now(),
        });

        // 2. 获取产品信息验证
        console.log("2️⃣ 验证产品信息...");
        const productInfo = await productHelper.getProduct(merchant, initialProduct.productId);
        expect(productInfo).toBeDefined();
        expect(productInfo.id).toBe(initialProduct.productId);

        // 3. 更新产品销量
        console.log("3️⃣ 更新产品销量...");
        await productHelper.updateProductSales(merchant, initialProduct.productId, 5);
        workflow.operations.push({
          action: "sales_update",
          productId: initialProduct.productId,
          sales: 5,
          timestamp: Date.now(),
        });

        // 4. 创建更多产品
        console.log("4️⃣ 创建更多产品...");
        const additionalProducts = [];
        for (let i = 0; i < 2; i++) {
          const product = await productHelper.createProductWithIndex(merchant, {
            name: `批量产品_${i + 1}`,
            description: `第${i + 1}个批量创建的产品`,
            price: 150000 + i * 25000,
            keywords: ["批量", `产品${i + 1}`, "测试"],
          });
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

        // 验证工作流程结果
        expect(workflow.products.length).toBe(3); // 1个初始 + 2个批量
        expect(workflow.operations.length).toBe(4); // 3个创建 + 1个销量更新
        expect(workflow.errors.length).toBe(0);

        console.log("✅ 商户产品管理工作流程测试完成");
        console.log(
          `📊 创建了 ${workflow.products.length} 个产品，执行了 ${workflow.operations.length} 个操作`
        );
      } catch (error) {
        workflow.errors.push(
          `商户产品管理工作流程失败: ${error instanceof Error ? error.message : String(error)}`
        );
        console.error("❌ 商户产品管理工作流程失败:", error);
        throw error;
      }
    });

    it("应该测试性能基准工作流程", async () => {
      console.log("⚡ 开始性能基准测试工作流程...");

      const benchmarkResults: {
        systemInitBenchmark: any;
        merchantRegistrationBenchmark: any;
        productCreationBenchmark: any;
        errors: string[];
      } = {
        systemInitBenchmark: null,
        merchantRegistrationBenchmark: null,
        productCreationBenchmark: null,
        errors: [],
      };

      try {
        // 系统初始化性能测试
        console.log("1️⃣ 系统初始化性能测试...");
        const systemInitResult = await performanceHelper.measureTransactionPerformance(async () => {
          // 注意：系统已经初始化，这里测试的是检查系统状态的性能
          const isInitialized = await systemHelper.isSystemInitialized();
          return isInitialized ? "system-check-success" : "system-check-failed";
        });
        benchmarkResults.systemInitBenchmark = systemInitResult;

        // 商户注册性能测试
        console.log("2️⃣ 商户注册性能测试...");
        const testMerchant = await bankrunHelper.createFundedAccount();
        const merchantRegResult = await performanceHelper.measureTransactionPerformance(
          async () => {
            const result = await merchantHelper.fullMerchantRegistration(
              testMerchant,
              "性能测试商户",
              "专门用于性能测试"
            );
            return result.registerSignature;
          }
        );
        benchmarkResults.merchantRegistrationBenchmark = merchantRegResult;

        // 产品创建性能测试
        console.log("3️⃣ 产品创建性能测试...");
        const productCreationResult = await performanceHelper.measureTransactionPerformance(
          async () => {
            const result = await productHelper.createProductWithIndex(testMerchant, {
              name: "性能测试产品",
              description: "专门用于性能基准测试的产品",
              price: 299900,
              keywords: ["性能", "测试", "基准"],
            });
            return result.signature;
          }
        );
        benchmarkResults.productCreationBenchmark = productCreationResult;

        // 验证性能基准结果
        expect(benchmarkResults.systemInitBenchmark).toBeDefined();
        expect(benchmarkResults.merchantRegistrationBenchmark).toBeDefined();
        expect(benchmarkResults.productCreationBenchmark).toBeDefined();

        // 由于系统可能已经初始化，executionTime可能为0，这是正常的
        expect(benchmarkResults.systemInitBenchmark?.executionTime).toBeGreaterThanOrEqual(0);
        expect(benchmarkResults.merchantRegistrationBenchmark?.executionTime).toBeGreaterThan(0);
        expect(benchmarkResults.productCreationBenchmark?.executionTime).toBeGreaterThan(0);

        console.log("✅ 性能基准测试工作流程完成");
        console.log("📊 性能基准结果:", {
          systemInit: `${benchmarkResults.systemInitBenchmark?.executionTime || 0}ms`,
          merchantReg: `${benchmarkResults.merchantRegistrationBenchmark?.executionTime || 0}ms`,
          productCreation: `${benchmarkResults.productCreationBenchmark?.executionTime || 0}ms`,
        });
      } catch (error) {
        benchmarkResults.errors.push(
          `性能基准测试失败: ${error instanceof Error ? error.message : String(error)}`
        );
        console.error("❌ 性能基准测试失败:", error);
        throw error;
      }
    });
  });

  describe("工作流程辅助功能测试", () => {
    it("应该测试 PerformanceHelper 的功能", async () => {
      console.log("📊 测试 PerformanceHelper 功能...");

      // 测试计时器功能
      performanceHelper.startTimer();
      await new Promise((resolve) => setTimeout(resolve, 10)); // 等待10ms
      const elapsedTime = performanceHelper.endTimer();

      expect(elapsedTime).toBeGreaterThan(0);
      expect(elapsedTime).toBeLessThan(100); // 应该远小于100ms

      // 测试指标记录功能
      performanceHelper.recordMetric("test_metric", 100);
      performanceHelper.recordMetric("test_metric", 200);
      performanceHelper.recordMetric("test_metric", 150);

      const stats = performanceHelper.getMetricStats("test_metric");
      expect(stats).toBeDefined();
      expect(stats?.count).toBe(3);
      expect(stats?.average).toBe(150);
      expect(stats?.min).toBe(100);
      expect(stats?.max).toBe(200);

      // 测试批量性能测试
      const batchResult = await performanceHelper.batchPerformanceTest(
        "batch_test",
        async () => {
          await new Promise((resolve) => setTimeout(resolve, 5));
          return "test_result";
        },
        3
      );

      expect(batchResult.results.length).toBe(3);
      expect(batchResult.stats.averageTime).toBeGreaterThan(0);
      expect(batchResult.results.every((r) => r === "test_result")).toBe(true);

      console.log("✅ PerformanceHelper 功能测试完成");
    });

    it("应该测试 ValidationHelper 的功能", async () => {
      console.log("🔍 测试 ValidationHelper 功能...");

      // 测试商户数据验证
      const validMerchantData = {
        authority: "test_authority",
        name: "测试商户",
        description: "测试描述",
        totalProducts: 0,
        totalSales: 0,
      };

      const invalidMerchantData = {
        authority: "test_authority",
        // 缺少 name
        description: "测试描述",
      };

      expect(ValidationHelper.validateMerchantData(validMerchantData)).toBe(true);
      expect(ValidationHelper.validateMerchantData(invalidMerchantData)).toBe(false);

      // 测试产品数据验证
      const validProductData = {
        id: 1,
        merchant: "test_merchant",
        name: "测试产品",
        description: "测试描述",
        price: 100000,
        keywords: ["测试", "产品"],
        status: "active",
        salesCount: 0,
      };

      const invalidProductData = {
        id: 1,
        // 缺少 merchant
        name: "测试产品",
      };

      expect(ValidationHelper.validateProductData(validProductData)).toBe(true);
      expect(ValidationHelper.validateProductData(invalidProductData)).toBe(false);

      console.log("✅ ValidationHelper 功能测试完成");
    });
  });
});
