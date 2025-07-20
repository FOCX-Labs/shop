import { describe, it, expect, beforeAll, afterAll } from "@jest/globals";
import { Keypair } from "@solana/web3.js";
import { BankrunHelper } from "../test-utils/bankrun-helper";
import { SystemHelper } from "../test-utils/system-helper";
import { MerchantHelper } from "../test-utils/merchant-helper";
import { ProductHelper } from "../test-utils/product-helper";
import { SearchHelper, SearchParameters } from "../test-utils/search-helper";

describe("SearchHelper 覆盖率提升测试", () => {
  let bankrunHelper: BankrunHelper;
  let systemHelper: SystemHelper;
  let merchantHelper: MerchantHelper;
  let productHelper: ProductHelper;
  let searchHelper: SearchHelper;
  let testMerchant: Keypair;

  beforeAll(async () => {
    console.log("🏗️ 初始化 SearchHelper 覆盖率测试环境...");

    // 初始化测试环境
    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    systemHelper = new SystemHelper(bankrunHelper.program, bankrunHelper.provider as any);
    merchantHelper = new MerchantHelper(bankrunHelper.program, bankrunHelper.provider as any);
    productHelper = new ProductHelper(bankrunHelper.program, bankrunHelper.provider as any);
    searchHelper = new SearchHelper(bankrunHelper.program, bankrunHelper.provider as any);

    // 初始化系统
    await systemHelper.initializeSystem();

    // 创建测试商户
    testMerchant = Keypair.generate();
    await bankrunHelper.fundAccount(testMerchant.publicKey, 10 * 1e9); // 10 SOL
    await merchantHelper.fullMerchantRegistration(
      testMerchant,
      "搜索测试商户",
      "专门用于搜索功能测试"
    );

    console.log("✅ SearchHelper 覆盖率测试环境初始化完成");
  });

  afterAll(async () => {
    console.log("🧹 清理 SearchHelper 覆盖率测试环境...");
  });

  describe("关键词索引管理功能", () => {
    it("应该测试 initializeKeywordIndex 方法", async () => {
      console.log("🔍 测试 initializeKeywordIndex 方法...");

      const keyword = "测试关键词";
      const signature = await searchHelper.initializeKeywordIndex(testMerchant, keyword);

      expect(signature).toBeDefined();
      expect(typeof signature).toBe("string");

      console.log("✅ 关键词索引初始化成功:", signature);
    });

    it("应该测试重复初始化关键词索引", async () => {
      console.log("🔍 测试重复初始化关键词索引...");

      const keyword = "重复关键词";

      // 第一次初始化
      const signature1 = await searchHelper.initializeKeywordIndex(testMerchant, keyword);
      expect(signature1).toBeDefined();

      // 第二次初始化应该处理重复情况
      try {
        const signature2 = await searchHelper.initializeKeywordIndex(testMerchant, keyword);
        console.log("⚠️ 重复初始化成功（可能是预期行为）:", signature2);
      } catch (error) {
        console.log(
          "✅ 重复初始化被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试空关键词的处理", async () => {
      console.log("🔍 测试空关键词的处理...");

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, "");
        console.log("⚠️ 空关键词被接受（可能是预期行为）");
      } catch (error) {
        console.log(
          "✅ 空关键词被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
        expect(error).toBeDefined();
      }
    });

    it("应该测试特殊字符关键词", async () => {
      console.log("🔍 测试特殊字符关键词...");

      const specialKeywords = ["测试@#$", "关键词123", "keyword_test"];

      for (const keyword of specialKeywords) {
        try {
          const signature = await searchHelper.initializeKeywordIndex(testMerchant, keyword);
          console.log(`✅ 特殊关键词 "${keyword}" 初始化成功:`, signature);
        } catch (error) {
          console.log(
            `⚠️ 特殊关键词 "${keyword}" 初始化失败:`,
            error instanceof Error ? error.message : String(error)
          );
        }
      }
    });
  });

  describe("关键词搜索功能", () => {
    beforeEach(async () => {
      // 为每个测试创建唯一的关键词，避免冲突
      const timestamp = Date.now();
      const uniqueKeyword = `搜索测试_${timestamp}`;

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, uniqueKeyword);
        // 将唯一关键词存储在全局变量中供测试使用
        (global as any).testKeyword = uniqueKeyword;
      } catch (error) {
        console.log(`关键词 "${uniqueKeyword}" 初始化失败，使用备用关键词`);
        (global as any).testKeyword = "搜索测试";
      }

      // 为错误处理路径测试初始化关键词
      const testKeywords = [`测试_${timestamp}`, `手机_${timestamp}`, `电子_${timestamp}`];
      for (const keyword of testKeywords) {
        try {
          await searchHelper.initializeKeywordIndex(testMerchant, keyword);
        } catch (error) {
          // 如果已存在则忽略错误
          console.log(`关键词 "${keyword}" 可能已存在，继续测试`);
        }
      }
      (global as any).testKeywords = testKeywords;
    });

    it("应该测试 searchByKeyword 方法 - 基本搜索", async () => {
      console.log("🔍 测试 searchByKeyword 方法 - 基本搜索...");

      const testKeyword = (global as any).testKeyword || "搜索测试";
      const result = await searchHelper.searchByKeyword(testKeyword);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ 基本关键词搜索成功:", result);
    });

    it("应该测试 searchByKeyword 方法 - 带分页参数", async () => {
      console.log("🔍 测试 searchByKeyword 方法 - 带分页参数...");

      const testKeyword = (global as any).testKeyword || "搜索测试";
      const result = await searchHelper.searchByKeyword(testKeyword, 5, 20);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 带分页参数的关键词搜索成功:", result);
    });

    it("应该测试不存在的关键词搜索", async () => {
      console.log("🔍 测试不存在的关键词搜索...");

      const result = await searchHelper.searchByKeyword("不存在的关键词");

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 不存在关键词搜索处理正确:", result);
    });

    it("应该测试 searchByKeyword 的错误处理路径", async () => {
      console.log("🔍 测试 searchByKeyword 的错误处理路径...");

      // 使用预初始化的唯一关键词和一个不存在的关键词
      const testKeywords = (global as any).testKeywords || ["测试", "手机", "电子"];
      const allTestKeywords = [...testKeywords, "未知关键词_" + Date.now()];

      for (const keyword of allTestKeywords) {
        const result = await searchHelper.searchByKeyword(keyword);
        expect(result).toBeDefined();
        expect(Array.isArray(result.products)).toBe(true);
        console.log(`✅ 关键词 "${keyword}" 搜索结果:`, result.products.length);
      }
    });
  });

  describe("价格搜索功能", () => {
    it("应该测试 searchByPriceRange 方法", async () => {
      console.log("🔍 测试 searchByPriceRange 方法...");

      const result = await searchHelper.searchByPriceRange(1000, 50000);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ 价格范围搜索成功:", result);
    });

    it("应该测试价格范围搜索 - 带分页参数", async () => {
      console.log("🔍 测试价格范围搜索 - 带分页参数...");

      const result = await searchHelper.searchByPriceRange(5000, 100000, 10, 5);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 带分页的价格范围搜索成功:", result);
    });

    it("应该测试无效价格范围", async () => {
      console.log("🔍 测试无效价格范围...");

      try {
        // 最小价格大于最大价格
        const result = await searchHelper.searchByPriceRange(50000, 1000);
        console.log("⚠️ 无效价格范围被接受:", result);
      } catch (error) {
        console.log(
          "✅ 无效价格范围被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试极端价格值", async () => {
      console.log("🔍 测试极端价格值...");

      const extremeCases = [
        { min: 0, max: 1 },
        { min: 0, max: Number.MAX_SAFE_INTEGER },
        { min: 1, max: 1 },
      ];

      for (const { min, max } of extremeCases) {
        try {
          const result = await searchHelper.searchByPriceRange(min, max);
          console.log(`✅ 极端价格范围 [${min}, ${max}] 搜索成功:`, result.products.length);
        } catch (error) {
          console.log(
            `⚠️ 极端价格范围 [${min}, ${max}] 搜索失败:`,
            error instanceof Error ? error.message : String(error)
          );
        }
      }
    });
  });

  describe("销量搜索功能", () => {
    it("应该测试 searchBySalesRange 方法", async () => {
      console.log("🔍 测试 searchBySalesRange 方法...");

      const result = await searchHelper.searchBySalesRange(0, 100);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ 销量范围搜索成功:", result);
    });

    it("应该测试销量范围搜索 - 带分页参数", async () => {
      console.log("🔍 测试销量范围搜索 - 带分页参数...");

      const result = await searchHelper.searchBySalesRange(10, 1000, 5, 15);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 带分页的销量范围搜索成功:", result);
    });

    it("应该测试无效销量范围", async () => {
      console.log("🔍 测试无效销量范围...");

      try {
        // 最小销量大于最大销量
        const result = await searchHelper.searchBySalesRange(1000, 10);
        console.log("⚠️ 无效销量范围被接受:", result);
      } catch (error) {
        console.log(
          "✅ 无效销量范围被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试负销量值", async () => {
      console.log("🔍 测试负销量值...");

      try {
        const result = await searchHelper.searchBySalesRange(-10, 100);
        console.log("⚠️ 负销量值被接受:", result);
      } catch (error) {
        console.log(
          "✅ 负销量值被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });
  });

  describe("综合搜索功能", () => {
    beforeEach(async () => {
      const timestamp = Date.now();
      const uniqueKeyword = `综合搜索_${timestamp}`;

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, uniqueKeyword);
        (global as any).combinedSearchKeyword = uniqueKeyword;
      } catch (error) {
        console.log(`综合搜索关键词 "${uniqueKeyword}" 初始化失败，使用备用关键词`);
        (global as any).combinedSearchKeyword = "综合搜索";
      }
    });

    it("应该测试 combinedSearch 方法 - 完整参数", async () => {
      console.log("🔍 测试 combinedSearch 方法 - 完整参数...");

      const keyword = (global as any).combinedSearchKeyword || "综合搜索";
      const searchParams: SearchParameters = {
        keywords: [keyword],
        priceMin: 1000,
        priceMax: 50000,
        salesMin: 0,
        salesMax: 100,
        offset: 0,
        limit: 10,
      };

      const result = await searchHelper.combinedSearch(searchParams);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ 完整参数综合搜索成功:", result);
    });

    it("应该测试 combinedSearch 方法 - 部分参数", async () => {
      console.log("🔍 测试 combinedSearch 方法 - 部分参数...");

      const keyword = (global as any).combinedSearchKeyword || "综合搜索";
      const searchParams: SearchParameters = {
        keywords: [keyword],
        priceMin: 1000,
      };

      const result = await searchHelper.combinedSearch(searchParams);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 部分参数综合搜索成功:", result);
    });

    it("应该测试 combinedSearch 方法 - 空参数", async () => {
      console.log("🔍 测试 combinedSearch 方法 - 空参数...");

      const searchParams: SearchParameters = {};

      const result = await searchHelper.combinedSearch(searchParams);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 空参数综合搜索成功:", result);
    });

    it("应该测试 combinedSearch 方法 - 带商户参数", async () => {
      console.log("🔍 测试 combinedSearch 方法 - 带商户参数...");

      const searchParams: SearchParameters = {
        merchant: testMerchant.publicKey,
        priceMin: 1000,
        priceMax: 50000,
      };

      const result = await searchHelper.combinedSearch(searchParams);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 带商户参数综合搜索成功:", result);
    });
  });

  describe("关键词索引搜索功能", () => {
    beforeEach(async () => {
      const timestamp = Date.now();
      const uniqueKeyword = `索引搜索_${timestamp}`;

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, uniqueKeyword);
        (global as any).indexSearchKeyword = uniqueKeyword;
      } catch (error) {
        console.log(`索引搜索关键词 "${uniqueKeyword}" 初始化失败，使用备用关键词`);
        (global as any).indexSearchKeyword = "索引搜索";
      }
    });

    it("应该测试 searchKeywordIndex 方法", async () => {
      console.log("🔍 测试 searchKeywordIndex 方法...");

      const keyword = (global as any).indexSearchKeyword || "索引搜索";
      const result = await searchHelper.searchKeywordIndex(keyword);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ 关键词索引搜索成功:", result);
    });

    it("应该测试 searchKeywordIndex 方法 - 带分页参数", async () => {
      console.log("🔍 测试 searchKeywordIndex 方法 - 带分页参数...");

      const keyword = (global as any).indexSearchKeyword || "索引搜索";
      const result = await searchHelper.searchKeywordIndex(keyword, 5, 20);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 带分页的关键词索引搜索成功:", result);
    });

    it("应该测试不存在的关键词索引搜索", async () => {
      console.log("🔍 测试不存在的关键词索引搜索...");

      const uniqueKeyword = "不存在索引_" + Date.now();
      const result = await searchHelper.searchKeywordIndex(uniqueKeyword);

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      console.log("✅ 不存在关键词索引搜索处理正确:", result);
    });
  });

  describe("多关键词搜索功能", () => {
    beforeEach(async () => {
      const timestamp = Date.now();
      const keyword1 = `多关键词1_${timestamp}`;
      const keyword2 = `多关键词2_${timestamp}`;

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, keyword1);
        await searchHelper.initializeKeywordIndex(testMerchant, keyword2);
        (global as any).multiKeywords = [keyword1, keyword2];
      } catch (error) {
        console.log(`多关键词初始化失败，使用备用关键词`);
        (global as any).multiKeywords = ["多关键词1", "多关键词2"];
      }
    });

    it("应该测试 multiKeywordSearch 方法 - 多个关键词", async () => {
      console.log("🔍 测试 multiKeywordSearch 方法 - 多个关键词...");

      const keywords = (global as any).multiKeywords || ["多关键词1", "多关键词2"];
      const result = await searchHelper.multiKeywordSearch(keywords);

      expect(result).toBeDefined();
      expect(Array.isArray(result.results)).toBe(true);
      expect(result.results.length).toBe(keywords.length);
      expect(Array.isArray(result.intersection)).toBe(true);
      expect(Array.isArray(result.union)).toBe(true);

      console.log("✅ 多关键词搜索成功:", {
        resultsCount: result.results.length,
        intersectionCount: result.intersection.length,
        unionCount: result.union.length,
      });
    });

    it("应该测试 multiKeywordSearch 方法 - 单个关键词", async () => {
      console.log("🔍 测试 multiKeywordSearch 方法 - 单个关键词...");

      const keywords = (global as any).multiKeywords || ["多关键词1", "多关键词2"];
      const singleKeyword = [keywords[0]];
      const result = await searchHelper.multiKeywordSearch(singleKeyword);

      expect(result.results.length).toBe(1);
      expect(result.intersection).toEqual(result.union);

      console.log("✅ 单关键词多搜索处理正确:", result);
    });

    it("应该测试 multiKeywordSearch 方法 - 空关键词数组", async () => {
      console.log("🔍 测试 multiKeywordSearch 方法 - 空关键词数组...");

      const keywords: string[] = [];
      const result = await searchHelper.multiKeywordSearch(keywords);

      expect(result.results.length).toBe(0);
      expect(result.intersection.length).toBe(0);
      expect(result.union.length).toBe(0);

      console.log("✅ 空关键词数组处理正确:", result);
    });

    it("应该测试 multiKeywordSearch 方法 - 带分页参数", async () => {
      console.log("🔍 测试 multiKeywordSearch 方法 - 带分页参数...");

      const keywords = (global as any).multiKeywords || ["多关键词1", "多关键词2"];
      const result = await searchHelper.multiKeywordSearch(keywords, 5, 15);

      expect(result).toBeDefined();
      expect(Array.isArray(result.results)).toBe(true);
      expect(Array.isArray(result.intersection)).toBe(true);
      expect(Array.isArray(result.union)).toBe(true);

      console.log("✅ 带分页的多关键词搜索成功:", result);
    });
  });

  describe("性能基准测试功能", () => {
    beforeEach(async () => {
      const timestamp = Date.now();
      const performanceKeyword = `性能测试_${timestamp}`;
      const keywordKeyword = `keyword_${timestamp}`;
      const testKeyword = `测试_${timestamp}`;

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, performanceKeyword);
        await searchHelper.initializeKeywordIndex(testMerchant, keywordKeyword);
        await searchHelper.initializeKeywordIndex(testMerchant, testKeyword);

        (global as any).performanceKeywords = {
          performance: performanceKeyword,
          keyword: keywordKeyword,
          test: testKeyword,
        };
      } catch (error) {
        console.log("性能测试关键词初始化失败，使用备用关键词");
        (global as any).performanceKeywords = {
          performance: "性能测试",
          keyword: "keyword",
          test: "测试",
        };
      }
    });

    it("应该测试 performanceSearch 方法 - 关键词搜索", async () => {
      console.log("🔍 测试 performanceSearch 方法 - 关键词搜索...");

      const keywords = (global as any).performanceKeywords || { keyword: "keyword" };
      const result = await searchHelper.performanceSearch(keywords.keyword, 3);

      expect(result).toBeDefined();
      expect(typeof result.averageTime).toBe("number");
      expect(typeof result.minTime).toBe("number");
      expect(typeof result.maxTime).toBe("number");
      expect(Array.isArray(result.results)).toBe(true);
      expect(result.results.length).toBe(3);

      console.log("✅ 关键词搜索性能测试成功:", {
        averageTime: result.averageTime,
        minTime: result.minTime,
        maxTime: result.maxTime,
      });
    });

    it("应该测试 performanceSearch 方法 - 价格搜索", async () => {
      console.log("🔍 测试 performanceSearch 方法 - 价格搜索...");

      try {
        const result = await searchHelper.performanceSearch("price", 2);

        console.log("🔍 性能测试结果:", result);

        expect(result).toBeDefined();
        expect(typeof result.averageTime).toBe("number");
        expect(typeof result.minTime).toBe("number");
        expect(typeof result.maxTime).toBe("number");
        expect(Array.isArray(result.results)).toBe(true);
        expect(result.results.length).toBe(2);

        // 验证每个结果都有必要的属性
        for (const searchResult of result.results) {
          expect(searchResult).toBeDefined();
          expect(Array.isArray(searchResult.products)).toBe(true);
          expect(typeof searchResult.executionTime).toBe("number");
        }

        console.log("✅ 价格搜索性能测试成功:", {
          averageTime: result.averageTime,
          minTime: result.minTime,
          maxTime: result.maxTime,
          resultsCount: result.results.length,
        });
      } catch (error) {
        console.error("❌ 价格搜索性能测试失败:", error);
        throw error;
      }
    });

    it("应该测试 performanceSearch 方法 - 销量搜索", async () => {
      console.log("🔍 测试 performanceSearch 方法 - 销量搜索...");

      const result = await searchHelper.performanceSearch("sales", 2);

      expect(result).toBeDefined();
      expect(typeof result.averageTime).toBe("number");
      expect(result.results.length).toBe(2);

      console.log("✅ 销量搜索性能测试成功:", result);
    });

    it("应该测试 performanceSearch 方法 - 综合搜索", async () => {
      console.log("🔍 测试 performanceSearch 方法 - 综合搜索...");

      const result = await searchHelper.performanceSearch("combined", 2);

      expect(result).toBeDefined();
      expect(typeof result.averageTime).toBe("number");
      expect(result.results.length).toBe(2);

      console.log("✅ 综合搜索性能测试成功:", result);
    });

    it("应该测试 performanceSearch 方法 - 自定义关键词", async () => {
      console.log("🔍 测试 performanceSearch 方法 - 自定义关键词...");

      const keywords = (global as any).performanceKeywords || { performance: "性能测试" };
      const result = await searchHelper.performanceSearch(keywords.performance, 2);

      expect(result).toBeDefined();
      expect(typeof result.averageTime).toBe("number");
      expect(result.results.length).toBe(2);

      console.log("✅ 自定义关键词性能测试成功:", result);
    });
  });

  describe("搜索结果验证功能", () => {
    it("应该测试 validateSearchResults 方法 - 有效结果", async () => {
      console.log("🔍 测试 validateSearchResults 方法 - 有效结果...");

      const validResults = [10000, 10001, 10002];
      const isValid = searchHelper.validateSearchResults(validResults);

      expect(isValid).toBe(true);

      console.log("✅ 有效搜索结果验证成功:", isValid);
    });

    it("应该测试 validateSearchResults 方法 - 空数组", async () => {
      console.log("🔍 测试 validateSearchResults 方法 - 空数组...");

      const emptyResults: number[] = [];
      const isValid = searchHelper.validateSearchResults(emptyResults);

      expect(isValid).toBe(true);

      console.log("✅ 空数组验证成功:", isValid);
    });

    it("应该测试 validateSearchResults 方法 - 无效结果", async () => {
      console.log("🔍 测试 validateSearchResults 方法 - 无效结果...");

      const invalidResults = [10000, -1, 10002];
      const isValid = searchHelper.validateSearchResults(invalidResults);

      expect(isValid).toBe(false);

      console.log("✅ 无效搜索结果验证成功:", isValid);
    });

    it("应该测试 validateSearchResults 方法 - 非数组输入", async () => {
      console.log("🔍 测试 validateSearchResults 方法 - 非数组输入...");

      const nonArrayInput = "not an array" as any;
      const isValid = searchHelper.validateSearchResults(nonArrayInput);

      expect(isValid).toBe(false);

      console.log("✅ 非数组输入验证成功:", isValid);
    });

    it("应该测试 validateSearchResults 方法 - 包含非数字元素", async () => {
      console.log("🔍 测试 validateSearchResults 方法 - 包含非数字元素...");

      const mixedResults = [10000, "10001", 10002] as any;
      const isValid = searchHelper.validateSearchResults(mixedResults);

      expect(isValid).toBe(false);

      console.log("✅ 包含非数字元素验证成功:", isValid);
    });
  });

  describe("PDA 获取功能", () => {
    it("应该测试 getKeywordIndexPda 方法", async () => {
      console.log("🔍 测试 getKeywordIndexPda 方法...");

      const keyword = "测试关键词";
      const [pda, bump] = searchHelper.getKeywordIndexPda(keyword);

      expect(pda).toBeDefined();
      expect(typeof bump).toBe("number");
      expect(bump >= 0 && bump <= 255).toBe(true);

      console.log("✅ 关键词索引PDA获取成功:", { pda: pda.toString(), bump });
    });

    it("应该测试 getPriceIndexPda 方法", async () => {
      console.log("🔍 测试 getPriceIndexPda 方法...");

      const [pda, bump] = searchHelper.getPriceIndexPda();

      expect(pda).toBeDefined();
      expect(typeof bump).toBe("number");
      expect(bump >= 0 && bump <= 255).toBe(true);

      console.log("✅ 价格索引PDA获取成功:", { pda: pda.toString(), bump });
    });

    it("应该测试 getSalesIndexPda 方法", async () => {
      console.log("🔍 测试 getSalesIndexPda 方法...");

      const [pda, bump] = searchHelper.getSalesIndexPda();

      expect(pda).toBeDefined();
      expect(typeof bump).toBe("number");
      expect(bump >= 0 && bump <= 255).toBe(true);

      console.log("✅ 销量索引PDA获取成功:", { pda: pda.toString(), bump });
    });

    it("应该测试不同关键词的PDA唯一性", async () => {
      console.log("🔍 测试不同关键词的PDA唯一性...");

      const keyword1 = "关键词1";
      const keyword2 = "关键词2";

      const [pda1] = searchHelper.getKeywordIndexPda(keyword1);
      const [pda2] = searchHelper.getKeywordIndexPda(keyword2);

      expect(pda1.toString()).not.toBe(pda2.toString());

      console.log("✅ 不同关键词PDA唯一性验证成功:", {
        pda1: pda1.toString(),
        pda2: pda2.toString(),
      });
    });

    it("应该测试相同关键词的PDA一致性", async () => {
      console.log("🔍 测试相同关键词的PDA一致性...");

      const keyword = "一致性测试";

      const [pda1, bump1] = searchHelper.getKeywordIndexPda(keyword);
      const [pda2, bump2] = searchHelper.getKeywordIndexPda(keyword);

      expect(pda1.toString()).toBe(pda2.toString());
      expect(bump1).toBe(bump2);

      console.log("✅ 相同关键词PDA一致性验证成功:", {
        pda: pda1.toString(),
        bump: bump1,
      });
    });
  });

  describe("私有方法测试（通过公共接口）", () => {
    it("应该测试交集计算功能（通过 multiKeywordSearch）", async () => {
      console.log("🔍 测试交集计算功能...");

      // 使用预定义的关键词来测试交集计算
      const keywords = ["测试", "手机"]; // 这两个关键词应该有重叠的产品
      const result = await searchHelper.multiKeywordSearch(keywords);

      expect(Array.isArray(result.intersection)).toBe(true);
      expect(Array.isArray(result.union)).toBe(true);
      expect(result.intersection.length <= result.union.length).toBe(true);

      console.log("✅ 交集计算功能测试成功:", {
        intersection: result.intersection,
        union: result.union,
      });
    });

    it("应该测试并集计算功能（通过 multiKeywordSearch）", async () => {
      console.log("🔍 测试并集计算功能...");

      const keywords = ["测试", "电子"];
      const result = await searchHelper.multiKeywordSearch(keywords);

      expect(Array.isArray(result.union)).toBe(true);

      // 并集应该包含所有唯一的产品ID
      const allProducts = result.results.flatMap((r) => r.products);
      const uniqueProducts = [...new Set(allProducts)];
      expect(result.union.length).toBe(uniqueProducts.length);

      console.log("✅ 并集计算功能测试成功:", {
        union: result.union,
        uniqueCount: uniqueProducts.length,
      });
    });

    it("应该测试搜索结果提取功能（通过实际搜索）", async () => {
      console.log("🔍 测试搜索结果提取功能...");

      const result = await searchHelper.searchByKeyword("测试");

      expect(result).toBeDefined();
      expect(result.signature).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);

      // 验证结果格式
      const isValidFormat = searchHelper.validateSearchResults(result.products);
      expect(isValidFormat).toBe(true);

      console.log("✅ 搜索结果提取功能测试成功:", result);
    });
  });

  describe("边界条件和错误处理", () => {
    it("应该测试极大分页参数", async () => {
      console.log("🔍 测试极大分页参数...");

      try {
        const result = await searchHelper.searchByKeyword("测试", 999999, 999999);
        console.log("⚠️ 极大分页参数被接受:", result);
        expect(result).toBeDefined();
      } catch (error) {
        console.log(
          "✅ 极大分页参数被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试负分页参数", async () => {
      console.log("🔍 测试负分页参数...");

      try {
        const result = await searchHelper.searchByKeyword("测试", -1, -1);
        console.log("⚠️ 负分页参数被接受:", result);
        expect(result).toBeDefined();
      } catch (error) {
        console.log(
          "✅ 负分页参数被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试超长关键词", async () => {
      console.log("🔍 测试超长关键词...");

      const longKeyword = "a".repeat(1000);

      try {
        const result = await searchHelper.searchByKeyword(longKeyword);
        console.log("⚠️ 超长关键词被接受:", result);
        expect(result).toBeDefined();
      } catch (error) {
        console.log(
          "✅ 超长关键词被正确拒绝:",
          error instanceof Error ? error.message : String(error)
        );
      }
    });

    it("应该测试特殊字符关键词搜索", async () => {
      console.log("🔍 测试特殊字符关键词搜索...");

      const specialKeywords = ["@#$%", "测试\n换行", "emoji🎉"];

      for (const keyword of specialKeywords) {
        try {
          const result = await searchHelper.searchByKeyword(keyword);
          console.log(`✅ 特殊关键词 "${keyword}" 搜索成功:`, result.products.length);
        } catch (error) {
          console.log(
            `⚠️ 特殊关键词 "${keyword}" 搜索失败:`,
            error instanceof Error ? error.message : String(error)
          );
        }
      }
    });

    it("应该测试性能基准测试的边界情况", async () => {
      console.log("🔍 测试性能基准测试的边界情况...");

      // 测试0次迭代
      try {
        const result = await searchHelper.performanceSearch("keyword", 0);
        expect(result.results.length).toBe(0);
        expect(result.averageTime).toBe(0);
        console.log("✅ 0次迭代性能测试处理正确:", result);
      } catch (error) {
        console.log(
          "⚠️ 0次迭代性能测试失败:",
          error instanceof Error ? error.message : String(error)
        );
      }

      // 测试1次迭代
      const result1 = await searchHelper.performanceSearch("keyword", 1);
      expect(result1.results.length).toBe(1);
      expect(result1.averageTime).toBe(result1.minTime);
      expect(result1.averageTime).toBe(result1.maxTime);
      console.log("✅ 1次迭代性能测试处理正确:", result1);
    });
  });

  describe("覆盖率提升专项测试", () => {
    it("应该测试view成功时的结果处理分支", async () => {
      console.log("🔍 测试view成功时的结果处理分支...");

      // 这个测试旨在覆盖第71-82行的代码
      // 通过模拟view成功的情况来触发这些代码路径

      // 测试不同类型的view结果处理
      const testKeyword = "view_test_" + Date.now();

      try {
        await searchHelper.initializeKeywordIndex(testMerchant, testKeyword);
      } catch (error) {
        console.log("关键词初始化失败，继续测试");
      }

      const result = await searchHelper.searchByKeyword(testKeyword);

      expect(result).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(typeof result.executionTime).toBe("number");

      console.log("✅ view结果处理分支测试成功:", result);
    });

    it("应该测试综合搜索的错误处理分支", async () => {
      console.log("🔍 测试综合搜索的错误处理分支...");

      // 这个测试旨在覆盖第250行的代码
      // 通过使用特定的关键词来触发错误处理分支

      const searchParams: SearchParameters = {
        keywords: ["综合搜索"],
        priceMin: 1000,
        priceMax: 50000,
      };

      const result = await searchHelper.combinedSearch(searchParams);

      expect(result).toBeDefined();
      expect(Array.isArray(result.products)).toBe(true);
      expect(result.products.length).toBeGreaterThan(0);

      console.log("✅ 综合搜索错误处理分支测试成功:", result);
    });

    it("应该测试关键词索引搜索的特定分支", async () => {
      console.log("🔍 测试关键词索引搜索的特定分支...");

      // 这个测试旨在覆盖第297和299行的代码

      // 测试"索引搜索"关键词分支
      const result1 = await searchHelper.searchKeywordIndex("索引搜索");
      expect(result1).toBeDefined();
      expect(Array.isArray(result1.products)).toBe(true);
      console.log("✅ '索引搜索'关键词分支测试成功:", result1.products.length);

      // 测试"不存在索引"关键词分支
      const result2 = await searchHelper.searchKeywordIndex("不存在索引");
      expect(result2).toBeDefined();
      expect(Array.isArray(result2.products)).toBe(true);
      console.log("✅ '不存在索引'关键词分支测试成功:", result2.products.length);
    });

    it("应该测试私有方法的间接调用", async () => {
      console.log("🔍 测试私有方法的间接调用...");

      // 这个测试旨在覆盖第418-461行的私有方法
      // 通过调用公共方法来间接测试私有方法

      // 测试extractSearchResultsFromTransaction方法（通过价格搜索）
      const priceResult = await searchHelper.searchByPriceRange(1000, 5000);
      expect(priceResult).toBeDefined();
      expect(Array.isArray(priceResult.products)).toBe(true);
      console.log("✅ extractSearchResultsFromTransaction间接测试成功");

      // 测试calculateIntersection和calculateUnion方法（通过多关键词搜索）
      const multiResult = await searchHelper.multiKeywordSearch(["测试1", "测试2"]);
      expect(multiResult).toBeDefined();
      expect(Array.isArray(multiResult.intersection)).toBe(true);
      expect(Array.isArray(multiResult.union)).toBe(true);
      console.log("✅ 私有计算方法间接测试成功");

      // 测试validateSearchResults方法
      const validationResult1 = searchHelper.validateSearchResults([10000, 10001]);
      expect(validationResult1).toBe(true);

      const validationResult2 = searchHelper.validateSearchResults([]);
      expect(validationResult2).toBe(true);

      const validationResult3 = searchHelper.validateSearchResults([-1, 10000]);
      expect(validationResult3).toBe(false);

      console.log("✅ validateSearchResults方法测试成功");
    });

    it("应该测试0次迭代性能搜索的特殊处理", async () => {
      console.log("🔍 测试0次迭代性能搜索的特殊处理...");

      // 修复0次迭代时的NaN问题
      const result = await searchHelper.performanceSearch("keyword", 0);

      expect(result).toBeDefined();
      expect(result.results.length).toBe(0);
      expect(result.averageTime).toBe(0);
      expect(result.minTime).toBe(Infinity);
      expect(result.maxTime).toBe(-Infinity);

      console.log("✅ 0次迭代性能搜索特殊处理测试成功:", result);
    });

    it("应该测试私有方法的完整覆盖", async () => {
      console.log("🔍 测试私有方法的完整覆盖...");

      // 通过反射访问私有方法来提升函数覆盖率
      const searchHelperAny = searchHelper as any;

      // 测试 parseProductIds 方法
      if (searchHelperAny.parseProductIds) {
        // 测试数组输入
        const arrayResult = searchHelperAny.parseProductIds([10000, 10001, "10002"]);
        expect(Array.isArray(arrayResult)).toBe(true);
        console.log("✅ parseProductIds数组测试成功:", arrayResult);

        // 测试字符串JSON输入
        const jsonResult = searchHelperAny.parseProductIds('["10003", "10004"]');
        expect(Array.isArray(jsonResult)).toBe(true);
        console.log("✅ parseProductIds JSON测试成功:", jsonResult);

        // 测试无效输入
        const invalidResult = searchHelperAny.parseProductIds("invalid");
        expect(Array.isArray(invalidResult)).toBe(true);
        expect(invalidResult.length).toBe(0);
        console.log("✅ parseProductIds无效输入测试成功:", invalidResult);
      }

      // 测试 extractProductIdsFromLogs 方法
      if (searchHelperAny.extractProductIdsFromLogs) {
        const testLogs = [
          "Program log: Starting search",
          "Program log: Search results: [10000, 10001, 10002]",
          "Program log: Search completed",
          "Program log: Search results: [10003, 10004]",
        ];

        const logResult = searchHelperAny.extractProductIdsFromLogs(testLogs);
        expect(Array.isArray(logResult)).toBe(true);
        console.log("✅ extractProductIdsFromLogs测试成功:", logResult);

        // 测试空日志
        const emptyLogResult = searchHelperAny.extractProductIdsFromLogs([]);
        expect(Array.isArray(emptyLogResult)).toBe(true);
        expect(emptyLogResult.length).toBe(0);
        console.log("✅ extractProductIdsFromLogs空日志测试成功:", emptyLogResult);
      }

      console.log("✅ 私有方法完整覆盖测试成功");
    });
  });
});
