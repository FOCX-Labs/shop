// Jest测试环境设置文件
import { jest } from "@jest/globals";

// 设置全局测试超时时间
jest.setTimeout(120000); // 2分钟

// 全局测试常量
export const TEST_CONSTANTS = {
  DEFAULT_TIMEOUT: 60000, // 1分钟
  LONG_TIMEOUT: 180000, // 3分钟
  PERFORMANCE_THRESHOLDS: {
    SYSTEM_INIT: 200, // 200ms - 更宽松的阈值，适应测试环境
    MERCHANT_REGISTER: 300, // 300ms
    PRODUCT_CREATE: 200, // 200ms
    SEARCH_KEYWORD: 300, // 300ms
    SEARCH_PRICE: 200, // 200ms
    SEARCH_COMBINED: 500, // 500ms
  },
  COMPUTE_UNIT_LIMITS: {
    SYSTEM_INIT: 20000,
    MERCHANT_REGISTER: 15000,
    PRODUCT_CREATE: 25000,
    SEARCH: 10000,
  },
};

// 全局错误处理
process.on("unhandledRejection", (reason, promise) => {
  console.error("Unhandled Rejection at:", promise, "reason:", reason);
});

process.on("uncaughtException", (error) => {
  console.error("Uncaught Exception:", error);
});

// 测试环境标识
process.env.NODE_ENV = "test";
process.env.ANCHOR_PROVIDER_URL = "http://localhost:8899";
process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";

console.log("🧪 Jest测试环境已初始化");
console.log("⚙️  测试配置:", {
  超时时间: TEST_CONSTANTS.DEFAULT_TIMEOUT + "ms",
  性能阈值: TEST_CONSTANTS.PERFORMANCE_THRESHOLDS,
  计算单元限制: TEST_CONSTANTS.COMPUTE_UNIT_LIMITS,
});
