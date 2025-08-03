module.exports = {
    preset: "ts-jest",
    testEnvironment: "node",
    testMatch: ["**/?(*.)+(spec|test).ts"],
    testTimeout: 60000,
    // Force serial execution to avoid Solana account state conflicts
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
    // Add test sequencer to ensure deterministic test execution order
    testSequencer: "<rootDir>/tests/test-sequencer.js",
};
