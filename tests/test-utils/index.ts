/**
 * 统一测试框架入口
 * 整合所有测试辅助功能，提供一致的API接口
 */

// 基础辅助类
export {
  EnvironmentHelper,
  PerformanceHelper,
  PDAHelper,
  TransactionHelper,
  ValidationHelper,
  ErrorHelper,
} from "./helpers";

// 现代测试辅助类
export { BankrunHelper } from "./bankrun-helper";

// 系统级辅助类
export { SystemHelper } from "./system-helper";

// 业务级辅助类
export { MerchantHelper } from "./merchant-helper";
export { ProductHelper } from "./product-helper";

export { SearchHelper } from "./search-helper";

// ID生成系统辅助类
export { IdGeneratorHelper, type IdChunkInfo, type MerchantIdInfo } from "./id-generator-helper";

// 索引管理辅助类
export {
  IndexManagementHelper,
  type KeywordIndexInfo,
  type KeywordShardInfo,
  type PriceIndexInfo,
  type SalesIndexInfo,
} from "./index-management-helper";

// 工作流程辅助类
export { WorkflowHelper } from "./workflow_helpers";
export type { WorkflowConfig, WorkflowResult } from "./workflow_helpers";

// 统一测试框架
export { TestFramework } from "./framework";

// 类型定义
export * from "./types";
