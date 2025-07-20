import { TestFramework, TestSuiteConfig, TestCaseConfig, TestResult } from "./test-utils";

/**
 * 统一测试框架使用示例
 * 展示如何使用新的测试架构进行全面的电商平台测试
 */

async function runUnifiedTests() {
  // 1. 创建测试框架实例
  const framework = new TestFramework({
    network: "localnet",
    timeout: 30000,
    commitment: "confirmed",
  });

  // 2. 系统初始化测试套件
  const systemSuite = framework.registerTestSuite({
    name: "system-initialization",
    description: "系统初始化和配置测试",
    timeout: 60000,
    setup: async () => {
      console.log("准备系统测试环境...");
    },
    teardown: async () => {
      console.log("清理系统测试数据...");
    },
  });

  // 添加系统测试用例
  systemSuite.addTestCase(
    {
      name: "initialize-system",
      description: "测试系统初始化功能",
      timeout: 30000,
    },
    async (): Promise<TestResult> => {
      const systemHelper = framework.getSystemHelper();

      try {
        const result = await systemHelper.initializeSystem({
          version: "1.0.0",
          maxMerchants: 1000,
          maxProducts: 10000,
        });

        return {
          success: true,
          duration: 0,
          errors: [],
          warnings: [],
          metadata: { globalRootPda: result.globalRootPda.toString() },
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  systemSuite.addTestCase(
    {
      name: "verify-system-state",
      description: "验证系统状态",
      dependencies: ["initialize-system"],
    },
    async (): Promise<TestResult> => {
      const systemHelper = framework.getSystemHelper();

      try {
        const systemInfo = await systemHelper.getSystemInfo();
        const isValid = systemInfo && systemInfo.version === "1.0.0";

        return {
          success: isValid,
          duration: 0,
          errors: isValid ? [] : ["系统状态验证失败"],
          warnings: [],
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  // 3. 商户管理测试套件
  const merchantSuite = framework.registerTestSuite({
    name: "merchant-management",
    description: "商户注册和管理测试",
    timeout: 120000,
  });

  merchantSuite.addTestCase(
    {
      name: "register-merchants",
      description: "批量注册商户测试",
    },
    async (): Promise<TestResult> => {
      const merchantHelper = framework.getMerchantHelper();
      const errors: string[] = [];
      const merchants = [];

      try {
        // 注册多个测试商户
        for (let i = 0; i < 5; i++) {
          const merchant = await merchantHelper.createTestMerchant(
            `测试商户_${i + 1}`,
            `第${i + 1}个测试商户`
          );
          merchants.push(merchant);
        }

        // 验证所有商户都成功注册
        for (const merchant of merchants) {
          const info = await merchantHelper.getMerchantInfo(merchant.merchant);
          if (!info) {
            errors.push(`商户 ${merchant.merchant.publicKey.toString()} 信息获取失败`);
          }
        }

        return {
          success: errors.length === 0,
          duration: 0,
          errors,
          warnings: [],
          metadata: { merchantCount: merchants.length },
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  // 4. 产品管理测试套件
  const productSuite = framework.registerTestSuite({
    name: "product-management",
    description: "产品创建和管理测试",
  });

  productSuite.addTestCase(
    {
      name: "create-products",
      description: "批量创建产品测试",
    },
    async (): Promise<TestResult> => {
      const merchantHelper = framework.getMerchantHelper();
      const productHelper = framework.getProductHelper();

      try {
        // 创建测试商户
        const merchant = await merchantHelper.createTestMerchant("产品测试商户", "用于产品测试");

        // 创建多个产品
        const products = [];
        for (let i = 0; i < 10; i++) {
          const product = await productHelper.createTestProduct(merchant.merchant, `产品${i + 1}`);
          products.push(product);
        }

        // 验证产品创建
        for (const product of products) {
          const productInfo = await productHelper.getProduct(merchant.merchant, product.productId);
          if (!productInfo) {
            return {
              success: false,
              duration: 0,
              errors: [`产品 ${product.productId} 信息获取失败`],
              warnings: [],
            };
          }
        }

        return {
          success: true,
          duration: 0,
          errors: [],
          warnings: [],
          metadata: { productCount: products.length },
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  // 5. 完整工作流程测试套件
  const workflowSuite = framework.registerTestSuite({
    name: "complete-workflow",
    description: "端到端业务流程测试",
    timeout: 300000,
  });

  workflowSuite.addTestCase(
    {
      name: "e-commerce-workflow",
      description: "完整电商业务流程测试",
    },
    async (): Promise<TestResult> => {
      const workflowHelper = framework.getWorkflowHelper();

      try {
        const result = await workflowHelper.executeCompleteECommerceWorkflow({
          merchantCount: 3,
          productsPerMerchant: 5,
          enablePerformanceTracking: true,
          enableValidation: true,
          cleanupAfterTest: false,
        });

        return {
          success: result.success,
          duration: result.performanceMetrics?.totalExecutionTime || 0,
          errors: result.errors,
          warnings: [],
          metadata: {
            merchantCount: result.merchants.length,
            totalProducts: result.merchants.reduce((sum, m) => sum + m.products.length, 0),
            performanceMetrics: result.performanceMetrics,
          },
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  // 6. 性能测试套件
  const performanceSuite = framework.registerTestSuite({
    name: "performance-tests",
    description: "性能和压力测试",
  });

  performanceSuite.addTestCase(
    {
      name: "bulk-operations-performance",
      description: "批量操作性能测试",
    },
    async (): Promise<TestResult> => {
      const performanceHelper = framework.getPerformanceHelper();
      const merchantHelper = framework.getMerchantHelper();
      const productHelper = framework.getProductHelper();

      try {
        performanceHelper.startTimer();

        // 创建测试商户
        const merchant = await merchantHelper.createTestMerchant("性能测试商户", "性能测试");

        // 批量创建产品，测试性能
        const productCount = 100;
        const products = [];

        for (let i = 0; i < productCount; i++) {
          const product = await productHelper.createTestProduct(
            merchant.merchant,
            `性能测试产品${i + 1}`
          );
          products.push(product);
        }

        const totalTime = performanceHelper.getElapsedTime();
        const averageTime = totalTime / productCount;

        // 性能基准：每个产品创建应该在1秒内完成
        const performanceThreshold = 1000; // ms
        const success = averageTime < performanceThreshold;

        return {
          success,
          duration: totalTime,
          errors: success
            ? []
            : [`性能测试失败：平均创建时间 ${averageTime}ms 超过阈值 ${performanceThreshold}ms`],
          warnings: [],
          metadata: {
            productCount,
            totalTime,
            averageTime,
            threshold: performanceThreshold,
          },
        };
      } catch (error) {
        return {
          success: false,
          duration: 0,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }
  );

  // 7. 执行所有测试
  console.log("🚀 开始执行统一测试框架示例...");

  try {
    const results = await framework.executeAll({
      parallel: false, // 顺序执行以确保依赖关系
      reportConfig: {
        format: "console",
        includeDetails: true,
        includePerformance: true,
        includeValidation: true,
      },
    });

    console.log("\n🎯 测试执行完成！");
    console.log(
      `总体成功率: ${((results.summary.passedTests / results.summary.totalTests) * 100).toFixed(
        1
      )}%`
    );

    // 清理
    await framework.cleanup();

    return results;
  } catch (error) {
    console.error("❌ 测试执行失败:", error);
    throw error;
  }
}

// 导出运行函数
export { runUnifiedTests };

// 如果直接运行此文件
if (require.main === module) {
  runUnifiedTests()
    .then((results) => {
      console.log("✅ 所有测试完成");
      process.exit(results.summary.failedTests > 0 ? 1 : 0);
    })
    .catch((error) => {
      console.error("❌ 测试运行失败:", error);
      process.exit(1);
    });
}
