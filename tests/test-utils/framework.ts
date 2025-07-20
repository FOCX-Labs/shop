import {
  TestSuiteConfig,
  TestCaseConfig,
  TestExecutionResult,
  TestResult,
  TestEnvironmentConfig,
  TestReportConfig,
  PerformanceMetrics,
} from "./types";
import { EnvironmentHelper, PerformanceHelper } from "./helpers";
import { SystemHelper } from "./system-helper";
import { MerchantHelper } from "./merchant-helper";
import { ProductHelper } from "./product-helper";

import { ValidationHelper } from "./helpers";
import { WorkflowHelper } from "./workflow_helpers";

/**
 * 统一测试框架
 * 提供完整的Solana电商平台测试解决方案
 */
export class TestFramework {
  private envHelper: EnvironmentHelper;
  private performanceHelper: PerformanceHelper;
  private systemHelper: SystemHelper;
  private merchantHelper: MerchantHelper;
  private productHelper: ProductHelper;

  private workflowHelper: WorkflowHelper;

  private testSuites: Map<string, TestSuite> = new Map();
  private globalConfig: TestEnvironmentConfig;
  private isInitialized: boolean = false;

  constructor(config: TestEnvironmentConfig = {}) {
    this.globalConfig = config;
    this.envHelper = new EnvironmentHelper();
    this.performanceHelper = new PerformanceHelper();

    // 初始化辅助类
    const program = this.envHelper.getProgram();
    const provider = this.envHelper.getProvider();

    this.systemHelper = new SystemHelper(program, provider);
    this.merchantHelper = new MerchantHelper(program, provider);
    this.productHelper = new ProductHelper(program, provider);

    this.workflowHelper = new WorkflowHelper();
  }

  /**
   * 初始化测试框架
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    console.log("🚀 初始化测试框架...");

    try {
      // 检查环境
      await this.envHelper.verifyEnvironment();

      // 可选：预初始化系统
      if (this.globalConfig.network === "localnet") {
        await this.systemHelper.initializeSystem();
      }

      this.isInitialized = true;
      console.log("✅ 测试框架初始化完成");
    } catch (error) {
      console.error("❌ 测试框架初始化失败:", error);
      throw error;
    }
  }

  /**
   * 注册测试套件
   */
  registerTestSuite(config: TestSuiteConfig): TestSuite {
    const suite = new TestSuite(config, this);
    this.testSuites.set(config.name, suite);
    return suite;
  }

  /**
   * 执行所有测试套件
   */
  async executeAll(
    options: {
      parallel?: boolean;
      reportConfig?: TestReportConfig;
    } = {}
  ): Promise<TestExecutionResult> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    console.log(`🧪 开始执行 ${this.testSuites.size} 个测试套件...`);

    const startTime = Date.now();
    const results: TestExecutionResult = {
      suiteResults: [],
      summary: {
        totalSuites: this.testSuites.size,
        passedSuites: 0,
        failedSuites: 0,
        totalTests: 0,
        passedTests: 0,
        failedTests: 0,
        skippedTests: 0,
        totalDuration: 0,
      },
    };

    try {
      const suiteArray = Array.from(this.testSuites.values());

      if (options.parallel) {
        // 并行执行
        const suiteResults = await Promise.allSettled(suiteArray.map((suite) => suite.execute()));

        suiteResults.forEach((result, index) => {
          if (result.status === "fulfilled") {
            results.suiteResults.push(result.value);
          } else {
            console.error(`套件 ${suiteArray[index].config.name} 执行失败:`, result.reason);
          }
        });
      } else {
        // 顺序执行
        for (const suite of suiteArray) {
          try {
            const suiteResult = await suite.execute();
            results.suiteResults.push(suiteResult);
          } catch (error) {
            console.error(`套件 ${suite.config.name} 执行失败:`, error);
          }
        }
      }

      // 计算汇总统计
      results.suiteResults.forEach((suiteResult) => {
        results.summary.totalTests += suiteResult.testResults.length;

        if (suiteResult.overallResult.success) {
          results.summary.passedSuites++;
        } else {
          results.summary.failedSuites++;
        }

        suiteResult.testResults.forEach((testResult) => {
          if (testResult.result.success) {
            results.summary.passedTests++;
          } else {
            results.summary.failedTests++;
          }
        });
      });

      results.summary.totalDuration = Date.now() - startTime;

      // 生成报告
      if (options.reportConfig) {
        await this.generateReport(results, options.reportConfig);
      }

      this.printSummary(results);
    } catch (error) {
      console.error("❌ 测试执行过程中发生错误:", error);
      throw error;
    }

    return results;
  }

  /**
   * 执行特定测试套件
   */
  async executeSuite(suiteName: string): Promise<any> {
    const suite = this.testSuites.get(suiteName);
    if (!suite) {
      throw new Error(`测试套件 '${suiteName}' 不存在`);
    }

    return await suite.execute();
  }

  /**
   * 获取系统辅助类
   */
  getSystemHelper(): SystemHelper {
    return this.systemHelper;
  }

  /**
   * 获取商户辅助类
   */
  getMerchantHelper(): MerchantHelper {
    return this.merchantHelper;
  }

  /**
   * 获取产品辅助类
   */
  getProductHelper(): ProductHelper {
    return this.productHelper;
  }

  /**
   * 获取工作流程辅助类
   */
  getWorkflowHelper(): WorkflowHelper {
    return this.workflowHelper;
  }

  /**
   * 获取性能辅助类
   */
  getPerformanceHelper(): PerformanceHelper {
    return this.performanceHelper;
  }

  /**
   * 生成测试报告
   */
  private async generateReport(
    results: TestExecutionResult,
    config: TestReportConfig
  ): Promise<void> {
    // 这里可以扩展不同格式的报告生成
    switch (config.format) {
      case "json":
        await this.generateJsonReport(results, config);
        break;
      case "html":
        await this.generateHtmlReport(results, config);
        break;
      case "console":
        this.generateConsoleReport(results);
        break;
      default:
        console.log("📄 生成基础测试报告...");
    }
  }

  /**
   * 生成JSON格式报告
   */
  private async generateJsonReport(
    results: TestExecutionResult,
    config: TestReportConfig
  ): Promise<void> {
    console.log("📄 生成JSON测试报告...");
    // 实现JSON报告生成逻辑
  }

  /**
   * 生成HTML格式报告
   */
  private async generateHtmlReport(
    results: TestExecutionResult,
    config: TestReportConfig
  ): Promise<void> {
    console.log("📄 生成HTML测试报告...");
    // 实现HTML报告生成逻辑
  }

  /**
   * 生成控制台报告
   */
  private generateConsoleReport(results: TestExecutionResult): void {
    console.log("\n📊 详细测试报告:");
    console.log("=====================================");

    results.suiteResults.forEach((suiteResult) => {
      console.log(`\n📋 测试套件: ${suiteResult.suite.name}`);
      console.log(`状态: ${suiteResult.overallResult.success ? "✅ 通过" : "❌ 失败"}`);
      console.log(`测试用例数: ${suiteResult.testResults.length}`);

      suiteResult.testResults.forEach((testResult) => {
        const status = testResult.result.success ? "✅" : "❌";
        console.log(`  ${status} ${testResult.testCase.name} (${testResult.result.duration}ms)`);

        if (testResult.result.errors.length > 0) {
          testResult.result.errors.forEach((error) => {
            console.log(`    🔴 ${error}`);
          });
        }
      });
    });
  }

  /**
   * 打印测试摘要
   */
  private printSummary(results: TestExecutionResult): void {
    const { summary } = results;
    const successRate = ((summary.passedTests / summary.totalTests) * 100).toFixed(1);

    console.log("\n🎯 测试执行摘要:");
    console.log("=====================================");
    console.log(
      `总测试套件: ${summary.totalSuites} (通过: ${summary.passedSuites}, 失败: ${summary.failedSuites})`
    );
    console.log(
      `总测试用例: ${summary.totalTests} (通过: ${summary.passedTests}, 失败: ${summary.failedTests})`
    );
    console.log(`成功率: ${successRate}%`);
    console.log(`总执行时间: ${summary.totalDuration}ms`);

    if (summary.failedTests === 0) {
      console.log("🎉 所有测试都已通过!");
    } else {
      console.log(`⚠️  ${summary.failedTests} 个测试失败`);
    }
  }

  /**
   * 清理测试数据
   */
  async cleanup(): Promise<void> {
    console.log("🧹 清理测试数据...");
    // 实现清理逻辑
  }
}

/**
 * 测试套件类
 */
export class TestSuite {
  public config: TestSuiteConfig;
  private framework: TestFramework;
  private testCases: Map<string, TestCase> = new Map();

  constructor(config: TestSuiteConfig, framework: TestFramework) {
    this.config = config;
    this.framework = framework;
  }

  /**
   * 添加测试用例
   */
  addTestCase(config: TestCaseConfig, testFunction: () => Promise<TestResult>): TestCase {
    const testCase = new TestCase(config, testFunction);
    this.testCases.set(config.name, testCase);
    return testCase;
  }

  /**
   * 执行测试套件
   */
  async execute(): Promise<any> {
    console.log(`📋 执行测试套件: ${this.config.name}`);

    const startTime = Date.now();
    const testResults: Array<{
      testCase: TestCaseConfig;
      result: TestResult;
    }> = [];

    try {
      // 执行设置
      if (this.config.setup) {
        await this.config.setup();
      }

      // 执行测试用例
      for (const testCase of this.testCases.values()) {
        if (testCase.config.skip) {
          console.log(`⏭️  跳过测试: ${testCase.config.name}`);
          continue;
        }

        const result = await testCase.execute();
        testResults.push({
          testCase: testCase.config,
          result,
        });
      }

      // 执行清理
      if (this.config.teardown) {
        await this.config.teardown();
      }
    } catch (error) {
      console.error(`❌ 测试套件 ${this.config.name} 执行失败:`, error);
    }

    const endTime = Date.now();
    const overallSuccess = testResults.every((tr) => tr.result.success);

    return {
      suite: this.config,
      testResults,
      overallResult: {
        success: overallSuccess,
        duration: endTime - startTime,
        errors: testResults.flatMap((tr) => tr.result.errors),
        warnings: testResults.flatMap((tr) => tr.result.warnings),
      },
    };
  }
}

/**
 * 测试用例类
 */
export class TestCase {
  public config: TestCaseConfig;
  private testFunction: () => Promise<TestResult>;

  constructor(config: TestCaseConfig, testFunction: () => Promise<TestResult>) {
    this.config = config;
    this.testFunction = testFunction;
  }

  /**
   * 执行测试用例
   */
  async execute(): Promise<TestResult> {
    console.log(`  🧪 执行测试: ${this.config.name}`);

    const startTime = Date.now();
    let retries = this.config.retries || 0;

    while (retries >= 0) {
      try {
        const result = await this.testFunction();
        const endTime = Date.now();

        return {
          ...result,
          duration: endTime - startTime,
        };
      } catch (error) {
        if (retries > 0) {
          console.log(`⚠️  测试失败，还有 ${retries} 次重试机会...`);
          retries--;
          continue;
        }

        const endTime = Date.now();
        return {
          success: false,
          duration: endTime - startTime,
          errors: [error instanceof Error ? error.message : String(error)],
          warnings: [],
        };
      }
    }

    // 这里不应该到达，但为了类型安全
    return {
      success: false,
      duration: Date.now() - startTime,
      errors: ["意外的执行路径"],
      warnings: [],
    };
  }
}
