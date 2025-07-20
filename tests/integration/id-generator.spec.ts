import { expect } from "@jest/globals";
import { BankrunHelper } from "../test-utils/bankrun-helper";
import { SystemHelper } from "../test-utils/system-helper";
import { IdGeneratorHelper } from "../test-utils/id-generator-helper";
import { PerformanceHelper } from "../test-utils/performance-helper";
import { TEST_CONSTANTS } from "../setup";

describe("ID生成系统集成测试", () => {
  let bankrunHelper: BankrunHelper;
  let systemHelper: SystemHelper;
  let idGeneratorHelper: IdGeneratorHelper;
  let performanceHelper: PerformanceHelper;

  beforeAll(async () => {
    console.log("🏗️  初始化ID生成系统测试环境...");

    bankrunHelper = new BankrunHelper();
    await bankrunHelper.initialize();

    const program = bankrunHelper.getProgram();
    const provider = bankrunHelper.getProvider();

    systemHelper = new SystemHelper(program, provider as any);
    idGeneratorHelper = new IdGeneratorHelper(program, provider as any);
    performanceHelper = new PerformanceHelper(program, provider as any);

    // 初始化系统
    await systemHelper.initializeSystem(bankrunHelper.getContext());

    console.log("✅ ID生成系统测试环境初始化完成");
  }, TEST_CONSTANTS.LONG_TIMEOUT);

  afterAll(async () => {
    console.log("🧹 清理ID生成系统测试环境...");
    performanceHelper.clearMetrics();
  });

  describe("商户注册与ID范围分配", () => {
    it(
      "应该成功注册商户并分配初始ID范围",
      async () => {
        console.log("🚀 测试商户注册和ID范围分配...");

        const merchant = await bankrunHelper.createFundedAccount();

        const registrationResult = await idGeneratorHelper.registerMerchant(merchant);

        // 调试输出
        console.log("🔍 调试 - registrationResult.idRange:", {
          idRange: registrationResult.idRange,
          startType: typeof registrationResult.idRange.start,
          startValue: registrationResult.idRange.start,
          endType: typeof registrationResult.idRange.end,
          endValue: registrationResult.idRange.end,
        });

        expect(registrationResult.signature).toBeDefined();
        expect(registrationResult.merchantId).toBeGreaterThan(0);
        expect(registrationResult.merchantAccountPda).toBeDefined();
        expect(registrationResult.initialChunkPda).toBeDefined();
        expect(Number(registrationResult.idRange.start)).toBeGreaterThanOrEqual(0);
        expect(Number(registrationResult.idRange.end)).toBeGreaterThan(
          Number(registrationResult.idRange.start)
        );

        console.log("✅ 商户注册成功", {
          merchantId: registrationResult.merchantId,
          idRange: registrationResult.idRange,
          signature: registrationResult.signature,
        });

        // 验证商户ID账户创建
        const merchantIdInfo = await idGeneratorHelper.getMerchantIdAccount(merchant);
        expect(merchantIdInfo.merchantId).toBe(registrationResult.merchantId);
        expect(merchantIdInfo.lastChunkIndex).toBe(0);
        expect(merchantIdInfo.totalChunks).toBe(1);

        console.log("✅ 商户ID账户验证通过", merchantIdInfo);
      },
      TEST_CONSTANTS.LONG_TIMEOUT
    );

    it("应该为不同商户分配不重叠的ID范围", async () => {
      console.log("🔄 测试多商户ID范围不重叠...");

      const merchants = [
        await bankrunHelper.createFundedAccount(),
        await bankrunHelper.createFundedAccount(),
        await bankrunHelper.createFundedAccount(),
      ];

      const registrationResults = [];
      for (const merchant of merchants) {
        const result = await idGeneratorHelper.registerMerchant(merchant);
        registrationResults.push(result);
      }

      // 验证商户ID不重复
      const merchantIds = registrationResults.map((r) => r.merchantId);
      const uniqueMerchantIds = new Set(merchantIds);
      expect(uniqueMerchantIds.size).toBe(merchantIds.length);

      // 验证ID范围不重叠
      for (let i = 0; i < registrationResults.length; i++) {
        for (let j = i + 1; j < registrationResults.length; j++) {
          const range1 = registrationResults[i].idRange;
          const range2 = registrationResults[j].idRange;

          const overlap = !(range1.end < range2.start || range2.end < range1.start);
          expect(overlap).toBe(false);
        }
      }

      console.log("✅ 多商户ID范围验证通过", {
        merchantCount: merchants.length,
        merchantIds,
        ranges: registrationResults.map((r) => r.idRange),
      });
    });

    it("应该拒绝重复注册同一商户", async () => {
      console.log("❌ 测试重复注册商户...");

      const merchant = await bankrunHelper.createFundedAccount();

      // 第一次注册应该成功
      await idGeneratorHelper.registerMerchant(merchant);

      // 第二次注册应该失败
      await expect(idGeneratorHelper.registerMerchant(merchant)).rejects.toThrow();

      console.log("✅ 重复注册拒绝验证通过");
    });
  });

  describe("产品ID生成", () => {
    let testMerchant: any;

    beforeAll(async () => {
      testMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(testMerchant);
    });

    it("应该成功生成单个产品ID", async () => {
      console.log("🆔 测试单个产品ID生成...");

      const idResult = await idGeneratorHelper.generateProductId(testMerchant);

      expect(idResult.signature).toBeDefined();
      expect(idResult.productId).toBeGreaterThanOrEqual(0);
      expect(idResult.localId).toBeGreaterThanOrEqual(0);
      expect(idResult.chunkIndex).toBeGreaterThanOrEqual(0);

      console.log("✅ 产品ID生成成功", idResult);

      // 验证ID存在性
      const existsResult = await idGeneratorHelper.isIdExists(testMerchant, idResult.productId);
      expect(existsResult.exists).toBe(true);
      expect(existsResult.chunkIndex).toBe(idResult.chunkIndex);

      console.log("✅ ID存在性验证通过", existsResult);
    });

    it("应该生成连续且唯一的产品ID", async () => {
      console.log("🔗 测试连续产品ID生成...");

      // 为此测试创建独立的merchant以避免状态干扰
      const freshMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(freshMerchant);

      const generatedIds = [];
      const idsToGenerate = 10;

      for (let i = 0; i < idsToGenerate; i++) {
        try {
          const idResult = await idGeneratorHelper.generateProductId(freshMerchant);
          generatedIds.push(idResult.productId);
          console.log(`✅ 成功生成ID ${i + 1}/${idsToGenerate}: ${idResult.productId}`);

          // 在ID生成之间添加短暂延时以避免重复交易错误
          if (i < idsToGenerate - 1) {
            await new Promise((resolve) => setTimeout(resolve, 10));
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          console.error(`❌ 生成第${i + 1}个ID时失败:`, errorMessage);

          // 如果是重复交易错误，尝试重试一次
          if (errorMessage.includes("already been processed")) {
            try {
              await new Promise((resolve) => setTimeout(resolve, 50));
              const idResult = await idGeneratorHelper.generateProductId(freshMerchant);
              generatedIds.push(idResult.productId);
              console.log(`🔄 重试成功生成ID ${i + 1}/${idsToGenerate}: ${idResult.productId}`);
            } catch (retryError) {
              console.error(
                `❌ 重试失败:`,
                retryError instanceof Error ? retryError.message : String(retryError)
              );
              throw retryError;
            }
          } else {
            throw error;
          }
        }
      }

      // 验证唯一性
      const uniqueIds = new Set(generatedIds);
      expect(uniqueIds.size).toBe(generatedIds.length);

      // 验证连续性（在同一块内应该是连续的）
      generatedIds.sort((a, b) => a - b);
      for (let i = 1; i < generatedIds.length; i++) {
        expect(generatedIds[i]).toBe(generatedIds[i - 1] + 1);
      }

      console.log("✅ 连续产品ID生成验证通过", {
        generatedIds,
        uniqueCount: uniqueIds.size,
        isConsecutive: true,
      });
    });

    it("应该支持批量ID生成", async () => {
      console.log("📦 测试批量ID生成...");

      const batchSize = 15;
      const batchResult = await idGeneratorHelper.batchGenerateIds(testMerchant, batchSize);

      expect(batchResult.signature).toBeDefined();
      expect(batchResult.productIds).toHaveLength(batchSize);
      expect(batchResult.chunksUsed.length).toBeGreaterThanOrEqual(1);

      // 验证生成的ID唯一性
      const uniqueIds = new Set(batchResult.productIds);
      expect(uniqueIds.size).toBe(batchResult.productIds.length);

      console.log("✅ 批量ID生成验证通过", {
        batchSize,
        generatedCount: batchResult.productIds.length,
        chunksUsed: batchResult.chunksUsed,
      });

      // 验证所有ID的一致性
      const consistencyResult = await idGeneratorHelper.validateIdConsistency(
        testMerchant,
        batchResult.productIds
      );
      expect(consistencyResult.valid).toBe(true);
      expect(consistencyResult.duplicates).toHaveLength(0);
      expect(consistencyResult.invalidIds).toHaveLength(0);

      console.log("✅ ID一致性验证通过", consistencyResult);
    });
  });

  describe("ID块管理", () => {
    let testMerchant: any;

    beforeAll(async () => {
      testMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(testMerchant);
    });

    it("应该能够分配新的ID块", async () => {
      console.log("🧱 测试新ID块分配...");

      const newChunkResult = await idGeneratorHelper.allocateNewChunk(testMerchant);

      expect(newChunkResult.signature).toBeDefined();
      expect(newChunkResult.chunkIndex).toBe(1); // 第二个块
      expect(newChunkResult.chunkPda).toBeDefined();
      expect(newChunkResult.idRange.start).toBeGreaterThan(0);
      expect(newChunkResult.idRange.end).toBeGreaterThan(newChunkResult.idRange.start);

      console.log("✅ 新ID块分配成功", newChunkResult);

      // 验证商户账户更新
      const merchantIdInfo = await idGeneratorHelper.getMerchantIdAccount(testMerchant);
      expect(merchantIdInfo.lastChunkIndex).toBe(1);
      expect(merchantIdInfo.totalChunks).toBe(2);

      console.log("✅ 商户块状态更新验证通过", merchantIdInfo);
    });

    it("应该能够获取ID块信息", async () => {
      console.log("📊 测试ID块信息获取...");

      const chunkInfo = await idGeneratorHelper.getIdChunk(testMerchant.publicKey, 0);

      expect(chunkInfo.merchantId).toBeGreaterThan(0);
      expect(chunkInfo.chunkIndex).toBe(0);
      expect(chunkInfo.startId).toBeGreaterThanOrEqual(0);
      expect(chunkInfo.endId).toBeGreaterThan(chunkInfo.startId);
      expect(chunkInfo.nextAvailable).toBeGreaterThanOrEqual(0);
      expect(chunkInfo.utilization).toBeGreaterThanOrEqual(0);

      console.log("✅ ID块信息获取验证通过", chunkInfo);
    });

    it("应该能够正确计算块索引和本地ID", async () => {
      console.log("🧮 测试ID计算功能...");

      const merchantIdInfo = await idGeneratorHelper.getMerchantIdAccount(testMerchant);
      const merchantId = merchantIdInfo.merchantId;

      // 测试块索引计算
      const testGlobalId = merchantId * 10000 + 1500; // 第二个块中的ID
      const calculatedChunkIndex = idGeneratorHelper.calculateChunkIndexForId(
        testGlobalId,
        merchantId
      );
      expect(calculatedChunkIndex).toBe(1); // 应该在第二个块中

      // 测试本地ID计算
      const calculatedLocalId = idGeneratorHelper.calculateLocalIdFromGlobalId(
        testGlobalId,
        merchantId
      );
      expect(calculatedLocalId).toBe(500); // 在块中的偏移

      console.log("✅ ID计算功能验证通过", {
        merchantId,
        testGlobalId,
        calculatedChunkIndex,
        calculatedLocalId,
      });
    });
  });

  describe("性能和压力测试", () => {
    let testMerchant: any;

    beforeAll(async () => {
      testMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(testMerchant);
    });

    it(
      "应该满足ID生成性能要求",
      async () => {
        console.log("⚡ 执行ID生成性能测试...");

        const idsToGenerate = 100;
        const batchSize = 10;

        const performanceResult = await idGeneratorHelper.performanceTest(
          testMerchant,
          idsToGenerate,
          batchSize
        );

        expect(performanceResult.idsGenerated).toBe(idsToGenerate);
        expect(performanceResult.totalTime).toBeLessThan(30000); // 30秒内完成
        expect(performanceResult.averageTimePerBatch).toBeLessThan(3000); // 每批3秒内

        console.log("✅ ID生成性能测试通过", {
          idsGenerated: performanceResult.idsGenerated,
          totalTime: `${performanceResult.totalTime}ms`,
          averageTimePerBatch: `${performanceResult.averageTimePerBatch}ms`,
          chunksUsed: performanceResult.chunksUsed,
        });

        // 记录性能指标
        performanceHelper.recordMetric("id_generation_performance", performanceResult.totalTime);
        performanceHelper.recordMetric(
          "id_generation_throughput",
          performanceResult.idsGenerated / (performanceResult.totalTime / 1000)
        );
      },
      TEST_CONSTANTS.LONG_TIMEOUT * 2
    );

    it("应该处理批量商户注册场景", async () => {
      console.log("👥 测试批量商户注册性能...");

      const merchantCount = 20;
      const merchants = [];
      for (let i = 0; i < merchantCount; i++) {
        merchants.push(await bankrunHelper.createFundedAccount());
      }

      const startTime = Date.now();
      const batchResults = await idGeneratorHelper.batchRegisterMerchants(merchants);
      const endTime = Date.now();

      const successCount = batchResults.filter((r) => r.success).length;
      expect(successCount).toBe(merchantCount);

      const totalTime = endTime - startTime;
      expect(totalTime).toBeLessThan(60000); // 60秒内完成

      console.log("✅ 批量商户注册性能测试通过", {
        merchantCount,
        successCount,
        totalTime: `${totalTime}ms`,
        averageTimePerMerchant: `${totalTime / merchantCount}ms`,
      });

      // 验证所有商户ID唯一
      const merchantIds = batchResults
        .filter((r) => r.success && r.merchantId)
        .map((r) => r.merchantId);
      const uniqueMerchantIds = new Set(merchantIds);
      expect(uniqueMerchantIds.size).toBe(merchantIds.length);

      console.log("✅ 批量注册商户ID唯一性验证通过");
    });
  });

  describe("错误处理和边界情况", () => {
    let testMerchant: any;

    beforeAll(async () => {
      testMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(testMerchant);
    });

    it("应该正确处理ID不存在的情况", async () => {
      console.log("❓ 测试不存在ID查询...");

      const nonExistentId = 999999999;
      const existsResult = await idGeneratorHelper.isIdExists(testMerchant, nonExistentId);

      expect(existsResult.exists).toBe(false);
      expect(existsResult.chunkIndex).toBeUndefined();
      expect(existsResult.localId).toBeUndefined();

      console.log("✅ 不存在ID处理验证通过", existsResult);
    });

    it("应该正确检查商户注册状态", async () => {
      console.log("✅ 测试商户注册状态检查...");

      const registeredMerchant = testMerchant;
      const unregisteredMerchant = await bankrunHelper.createFundedAccount();

      const isRegistered = await idGeneratorHelper.isMerchantRegisteredForIds(registeredMerchant);
      const isNotRegistered = await idGeneratorHelper.isMerchantRegisteredForIds(
        unregisteredMerchant
      );

      expect(isRegistered).toBe(true);
      expect(isNotRegistered).toBe(false);

      console.log("✅ 商户注册状态检查验证通过", {
        registeredStatus: isRegistered,
        unregisteredStatus: isNotRegistered,
      });
    });

    it("应该处理无效的块索引访问", async () => {
      console.log("❌ 测试无效块索引处理...");

      const invalidChunkIndex = 999;

      await expect(
        idGeneratorHelper.getIdChunk(testMerchant.publicKey, invalidChunkIndex)
      ).rejects.toThrow();

      console.log("✅ 无效块索引错误处理验证通过");
    });

    it("应该验证ID生成的一致性约束", async () => {
      console.log("🔒 测试ID一致性约束...");

      // 创建一个新的商户来避免状态干扰
      const freshMerchant = await bankrunHelper.createFundedAccount();
      await idGeneratorHelper.registerMerchant(freshMerchant);

      // 生成一些ID
      const validIds = [];
      for (let i = 0; i < 5; i++) {
        try {
          const result = await idGeneratorHelper.generateProductId(freshMerchant);
          validIds.push(result.productId);
          console.log(`Generated ID ${i + 1}: ${result.productId}`);

          // 添加延时避免重复交易错误
          if (i < 4) {
            await new Promise((resolve) => setTimeout(resolve, 10));
          }
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          console.error(`Failed to generate ID ${i + 1}:`, errorMessage);
          // 重试一次
          if (errorMessage.includes("already been processed")) {
            await new Promise((resolve) => setTimeout(resolve, 50));
            try {
              const result = await idGeneratorHelper.generateProductId(freshMerchant);
              validIds.push(result.productId);
              console.log(`Generated ID ${i + 1} (retry): ${result.productId}`);
            } catch (retryError) {
              console.error(
                `Retry failed for ID ${i + 1}:`,
                retryError instanceof Error ? retryError.message : String(retryError)
              );
              break;
            }
          } else {
            break;
          }
        }
      }

      console.log("Generated valid IDs:", validIds);

      // 添加一些无效ID进行测试
      const merchantIdInfo = await idGeneratorHelper.getMerchantIdAccount(freshMerchant);
      const merchantStartId = merchantIdInfo.merchantId * 10000;
      console.log("Merchant info:", { merchantId: merchantIdInfo.merchantId, merchantStartId });

      const invalidIds = [
        ...validIds,
        merchantStartId - 1, // 超出范围的ID
        merchantStartId + 20000, // 超出范围的ID
        validIds[0], // 重复ID
      ];

      console.log("Test IDs for validation:", invalidIds);

      const consistencyResult = await idGeneratorHelper.validateIdConsistency(
        freshMerchant,
        invalidIds
      );

      console.log("Validation result:", consistencyResult);

      expect(consistencyResult.valid).toBe(false);
      expect(consistencyResult.duplicates.length).toBeGreaterThan(0);
      expect(consistencyResult.invalidIds.length).toBeGreaterThan(0);
      expect(consistencyResult.errors.length).toBeGreaterThan(0);

      console.log("✅ ID一致性约束验证通过", {
        totalIds: invalidIds.length,
        duplicates: consistencyResult.duplicates.length,
        invalidIds: consistencyResult.invalidIds.length,
        errors: consistencyResult.errors,
      });
    });
  });
});
