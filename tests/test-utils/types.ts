import { Keypair, PublicKey } from "@solana/web3.js";

/**
 * 基础测试类型定义
 */

// 测试环境配置
export interface TestEnvironmentConfig {
  network?: "localnet" | "devnet" | "testnet" | "mainnet";
  programId?: string;
  rpcUrl?: string;
  commitment?: "confirmed" | "finalized" | "processed";
  timeout?: number;
}

// 测试结果基础接口
export interface TestResult {
  success: boolean;
  duration: number;
  errors: string[];
  warnings: string[];
  metadata?: Record<string, any>;
}

// 性能测试结果
export interface PerformanceMetrics {
  startTime: number;
  endTime: number;
  duration: number;
  averageTime?: number;
  minTime?: number;
  maxTime?: number;
  throughput?: number;
}

/**
 * 系统相关类型
 */
export interface SystemTestResult extends TestResult {
  globalRootPda?: PublicKey;
  systemInfo?: SystemInfo;
}

export interface SystemInfo {
  version: string;
  totalMerchants: number;
  totalProducts: number;
  lastUpdated: number;
}

/**
 * 商户相关类型
 */
export interface MerchantTestData {
  merchant: Keypair;
  merchantPda: PublicKey;
  name: string;
  description: string;
  signature: string;
}

export interface MerchantTestResult extends TestResult {
  merchants: MerchantTestData[];
  totalRegistered: number;
}

/**
 * 产品相关类型
 */
export interface ProductTestData {
  productId: number;
  merchant: Keypair;
  name: string;
  description: string;
  price: number;
  keywords: string[];
  signature: string;
}

export interface ProductTestResult extends TestResult {
  products: ProductTestData[];
  totalCreated: number;
  averageCreationTime: number;
}

/**
 * 验证相关类型
 */
export interface ValidationRule {
  name: string;
  validate: (data: any) => ValidationResult;
  required?: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  score?: number;
}

export interface ValidationTestResult extends TestResult {
  validationResults: ValidationResult[];
  overallScore: number;
  failedRules: string[];
}

/**
 * 工作流程相关类型
 */
export interface WorkflowStep {
  name: string;
  description: string;
  execute: () => Promise<any>;
  validation?: ValidationRule[];
  timeout?: number;
}

export interface WorkflowTestResult extends TestResult {
  steps: Array<{
    step: WorkflowStep;
    result: TestResult;
    startTime: number;
    endTime: number;
  }>;
  totalSteps: number;
  completedSteps: number;
  failedSteps: number;
}

/**
 * 综合测试框架类型
 */
export interface TestSuiteConfig {
  name: string;
  description?: string;
  environment?: TestEnvironmentConfig;
  setup?: () => Promise<void>;
  teardown?: () => Promise<void>;
  timeout?: number;
  parallel?: boolean;
  retries?: number;
}

export interface TestCaseConfig {
  name: string;
  description?: string;
  tags?: string[];
  timeout?: number;
  retries?: number;
  skip?: boolean;
  dependencies?: string[];
}

export interface TestExecutionResult {
  suiteResults: Array<{
    suite: TestSuiteConfig;
    testResults: Array<{
      testCase: TestCaseConfig;
      result: TestResult;
    }>;
    overallResult: TestResult;
  }>;
  summary: {
    totalSuites: number;
    passedSuites: number;
    failedSuites: number;
    totalTests: number;
    passedTests: number;
    failedTests: number;
    skippedTests: number;
    totalDuration: number;
  };
}

/**
 * 扩展配置类型
 */
export interface TestReportConfig {
  format: "json" | "html" | "junit" | "console";
  outputPath?: string;
  includeDetails?: boolean;
  includePerformance?: boolean;
  includeValidation?: boolean;
}

export interface TestDataConfig {
  generateTestData?: boolean;
  testDataSize?: "small" | "medium" | "large";
  seedData?: any;
  cleanupAfterTest?: boolean;
}
