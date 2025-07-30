import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { OrderQueryService } from "./order-query-service";

// 主函数
async function main() {
  console.log("🚀 订单查询系统测试开始");

  // 设置连接
  const connection = new Connection(
    "https://devnet.helius-rpc.com/?api-key=48e26d41-1ec0-4a29-ac33-fa26d0112cef",
    "confirmed"
  );

  // 创建钱包和provider
  const wallet = new anchor.Wallet(Keypair.generate()); // 临时钱包，仅用于查询
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const queryService = new OrderQueryService();

  try {
    // 使用从增强业务流程脚本中获得的实际地址
    const buyerPublicKey = new PublicKey("3DghDSAbedNuTJJo9eM5VuiykiaUQSfoKkYWAq31YAQS");
    const merchantPublicKey = new PublicKey("Jeq3FAX7JEZNAdX8SY9M1NW5a43GZKUeRokTH6Mj2eV");

    console.log(`👤 买家地址: ${buyerPublicKey.toString()}`);
    console.log(`🏪 商户地址: ${merchantPublicKey.toString()}`);

    // 1. 测试买家订单查询
    console.log("\n" + "=".repeat(60));
    console.log("📋 买家订单查询测试");
    console.log("=".repeat(60));

    const buyerOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📊 买家订单查询结果:`);
    console.log(`   总订单数: ${buyerOrders.totalCount}`);
    console.log(`   当前页: ${buyerOrders.page + 1}`);
    console.log(`   页大小: ${buyerOrders.pageSize}`);
    console.log(`   有下一页: ${buyerOrders.hasNext ? "是" : "否"}`);
    console.log(`   有上一页: ${buyerOrders.hasPrev ? "是" : "否"}`);

    if (buyerOrders.orders.length > 0) {
      buyerOrders.orders.forEach((order, index) => {
        console.log(`\n📦 订单 ${index + 1}:`);
        console.log(`   订单序列号: ${order.buyerSequence}`);
        console.log(`   商户: ${order.merchant}`);
        console.log(`   产品ID: ${order.productId}`);
        console.log(`   数量: ${order.quantity}`);
        console.log(`   总价: ${order.totalAmount} TOKEN`);
        console.log(`   状态: ${order.status}`);
        console.log(`   创建时间: ${new Date(order.createdAt * 1000).toLocaleString()}`);
        console.log(`   订单PDA: ${order.orderPDA}`);
      });
    } else {
      console.log("   📭 暂无订单");
    }

    // 2. 测试商户订单查询
    console.log("\n" + "=".repeat(60));
    console.log("🏪 商户订单查询测试");
    console.log("=".repeat(60));

    const merchantOrders = await queryService.getMerchantOrders({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📊 商户订单查询结果:`);
    console.log(`   总订单数: ${merchantOrders.totalCount}`);
    console.log(`   当前页: ${merchantOrders.page + 1}`);
    console.log(`   页大小: ${merchantOrders.pageSize}`);
    console.log(`   有下一页: ${merchantOrders.hasNext ? "是" : "否"}`);
    console.log(`   有上一页: ${merchantOrders.hasPrev ? "是" : "否"}`);

    if (merchantOrders.orders.length > 0) {
      merchantOrders.orders.forEach((order, index) => {
        console.log(`\n🛍️ 订单 ${index + 1}:`);
        console.log(`   商户序列号: ${order.merchantSequence}`);
        console.log(`   买家序列号: ${order.buyerSequence}`);
        console.log(`   买家: ${order.buyer}`);
        console.log(`   产品ID: ${order.productId}`);
        console.log(`   数量: ${order.quantity}`);
        console.log(`   总价: ${order.totalAmount} TOKEN`);
        console.log(`   状态: ${order.status}`);
        console.log(`   创建时间: ${new Date(order.createdAt * 1000).toLocaleString()}`);
        console.log(`   订单PDA: ${order.orderPDA}`);
      });
    } else {
      console.log("   📭 暂无订单");
    }

    // 3. 测试分页查询
    if (buyerOrders.totalCount > 1) {
      console.log("\n" + "=".repeat(60));
      console.log("📄 分页查询测试");
      console.log("=".repeat(60));

      const page1 = await queryService.getBuyerOrders({
        buyer: buyerPublicKey,
        page: 0,
        pageSize: 1,
        sortOrder: "desc",
      });

      const page2 = await queryService.getBuyerOrders({
        buyer: buyerPublicKey,
        page: 1,
        pageSize: 1,
        sortOrder: "desc",
      });

      console.log(`\n📄 第一页订单:`);
      if (page1.orders.length > 0) {
        console.log(`   订单序列号: ${page1.orders[0].buyerSequence}`);
        console.log(`   状态: ${page1.orders[0].status}`);
      }

      console.log(`\n📄 第二页订单:`);
      if (page2.orders.length > 0) {
        console.log(`   订单序列号: ${page2.orders[0].buyerSequence}`);
        console.log(`   状态: ${page2.orders[0].status}`);
      }
    }

    // 4. 测试排序功能
    console.log("\n" + "=".repeat(60));
    console.log("🔄 排序功能测试");
    console.log("=".repeat(60));

    const ascOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "asc",
    });

    console.log(`\n📈 升序排序结果:`);
    ascOrders.orders.forEach((order, index) => {
      console.log(`   ${index + 1}. 序列号: ${order.buyerSequence}, 状态: ${order.status}`);
    });

    const descOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📉 降序排序结果:`);
    descOrders.orders.forEach((order, index) => {
      console.log(`   ${index + 1}. 序列号: ${order.buyerSequence}, 状态: ${order.status}`);
    });

    console.log("\n🎉 订单查询系统测试完成！");
  } catch (error) {
    console.error("❌ 测试失败:", error);
    process.exit(1);
  }
}

// 运行主函数
if (require.main === module) {
  main().catch(console.error);
}
