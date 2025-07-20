module.exports = {
    preset: "ts-jest",
    testEnvironment: "node",
    testMatch: ["**/?(*.)+(spec|test).ts"],
    testTimeout: 60000,
    // 强制串行执行以避免Solana账户状态冲突
    maxWorkers: 1,
    coverageThreshold: {
        global: {
            branches: 80,
            functions: 80,
            lines: 80,
            statements: 80,
        },
    },
    setupFilesAfterEnv: ["<rootDir>/tests/setup.ts"],
    testPathIgnorePatterns: ["/node_modules/", "/dist/"],
    collectCoverageFrom: [
        "programs/**/*.{ts,js}",
        "tests/test-utils/**/*.{ts,js}",
        "!**/*.spec.{ts,js}",
        "!**/*.test.{ts,js}",
    ],
    moduleFileExtensions: ["ts", "tsx", "js", "jsx", "json"],
    transform: {
        "^.+\\.(ts|tsx)$": "ts-jest",
    },
    verbose: true,
    // 添加测试序列器以确保确定性的测试执行顺序
    testSequencer: "<rootDir>/tests/test-sequencer.js",
};
