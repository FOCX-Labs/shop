import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { SolanaECommerce } from "../target/types/solana_e_commerce";

/**
 * 验证商户订单查询功能
 */
class MerchantOrderVerifier {
  private connection: Connection;
  private program: anchor.Program<SolanaECommerce>;

  constructor() {
    // 连接到本地网络
    this.connection = new Connection("http://localhost:8899", "confirmed");

    // 设置provider
    const provider = new anchor.AnchorProvider(
      this.connection,
      new anchor.Wallet(anchor.web3.Keypair.generate()),
      { commitment: "confirmed" }
    );
    anchor.setProvider(provider);

    // 初始化程序
    this.program = anchor.workspace.SolanaECommerce as anchor.Program<SolanaECommerce>;
  }

  /**
   * 计算PDA
   */
  private calculatePDA(seeds: (string | Buffer)[]): [PublicKey, number] {
    const seedBuffers = seeds.map((seed) =>
      typeof seed === "string" ? Buffer.from(seed, "utf8") : seed
    );
    return PublicKey.findProgramAddressSync(seedBuffers, this.program.programId);
  }

  /**
   * 查询商户的所有订单
   */
  async queryMerchantOrders(merchantPublicKey: PublicKey): Promise<void> {
    console.log(`\n🔍 查询商户订单: ${merchantPublicKey.toString()}`);
    console.log("=====================================");

    try {
      // 1. 查询商户订单计数账户
      const [merchantOrderCountPDA] = this.calculatePDA([
        "merchant_order_count",
        merchantPublicKey.toBuffer(),
      ]);

      console.log(`📊 商户订单计数PDA: ${merchantOrderCountPDA.toString()}`);

      let totalOrders = 0;
      try {
        const merchantOrderCountAccount = await this.program.account.merchantOrderCount.fetch(
          merchantOrderCountPDA
        );
        totalOrders = merchantOrderCountAccount.totalOrders.toNumber();
        console.log(`📈 商户总订单数: ${totalOrders}`);
      } catch (error) {
        console.log(`❌ 商户订单计数账户不存在或无法获取`);
        return;
      }

      // 2. 查询每个商户订单
      for (let i = 1; i <= totalOrders; i++) {
        const [merchantOrderPDA] = this.calculatePDA([
          "merchant_order",
          merchantPublicKey.toBuffer(),
          new anchor.BN(i).toArrayLike(Buffer, "le", 8),
        ]);

        console.log(`\n🏪 商户订单 ${i}:`);
        console.log(`   PDA: ${merchantOrderPDA.toString()}`);

        try {
          const merchantOrderAccount = await this.program.account.merchantOrder.fetch(
            merchantOrderPDA
          );

          console.log(`   商户: ${merchantOrderAccount.merchant.toString()}`);
          console.log(`   买家: ${merchantOrderAccount.buyer.toString()}`);
          console.log(`   序列号: ${merchantOrderAccount.merchantOrderSequence.toNumber()}`);
          console.log(`   买家订单PDA: ${merchantOrderAccount.buyerOrderPda.toString()}`);
          console.log(`   产品ID: ${merchantOrderAccount.productId.toNumber()}`);
          console.log(
            `   创建时间: ${new Date(
              merchantOrderAccount.createdAt.toNumber() * 1000
            ).toLocaleString()}`
          );

          // 3. 查询对应的买家订单详情
          try {
            const buyerOrderAccount = await this.program.account.order.fetch(
              merchantOrderAccount.buyerOrderPda
            );

            console.log(`   📦 买家订单详情:`);
            console.log(`      数量: ${buyerOrderAccount.quantity}`);
            console.log(`      价格: ${buyerOrderAccount.price.toNumber()}`);
            console.log(`      总金额: ${buyerOrderAccount.totalAmount.toNumber()}`);
            console.log(`      状态: ${JSON.stringify(buyerOrderAccount.status)}`);
            console.log(`      发货地址: ${buyerOrderAccount.shippingAddress}`);
            console.log(`      备注: ${buyerOrderAccount.notes}`);
            if (buyerOrderAccount.trackingNumber) {
              console.log(`      物流单号: ${buyerOrderAccount.trackingNumber}`);
            }
          } catch (error) {
            console.log(`   ❌ 无法获取买家订单详情: ${error}`);
          }
        } catch (error) {
          console.log(`   ❌ 无法获取商户订单: ${error}`);
        }
      }
    } catch (error) {
      console.log(`❌ 查询商户订单失败: ${error}`);
    }
  }

  /**
   * 主验证函数
   */
  async verify(): Promise<void> {
    console.log("🔍 开始验证商户订单查询功能");
    console.log("=====================================");

    // 使用本地测试中的商户公钥（从测试日志中获取）
    const merchantPublicKey = new PublicKey("DxuLkWihYH5VCJZS8dmFqCCkHv7zmMYwUVPN5cv2Lw9Q");

    await this.queryMerchantOrders(merchantPublicKey);

    console.log("\n✅ 商户订单查询验证完成");
  }
}

// 执行验证
async function main() {
  const verifier = new MerchantOrderVerifier();
  await verifier.verify();
}

main().catch(console.error);
