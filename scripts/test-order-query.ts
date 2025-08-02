import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { OrderQueryService } from "./order-query-service";

// Main function
async function main() {
  console.log("🚀 Order query system test started");

  // Setup connection
  const connection = new Connection(
    "https://devnet.helius-rpc.com/?api-key=48e26d41-1ec0-4a29-ac33-fa26d0112cef",
    "confirmed"
  );

  // Create wallet and provider
  const wallet = new anchor.Wallet(Keypair.generate()); // Temporary wallet, only for queries
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const queryService = new OrderQueryService();

  try {
    // Use actual addresses obtained from enhanced business flow script (updated for new PDA rules)
    const buyerPublicKey = new PublicKey("E8f76tNTJZPW9AWLedsuresALsrrBVvHHznCLvaqJbFe");
    const merchantPublicKey = new PublicKey("EqmeCvUSfz3puTw4LdsYNEVMrHn7fuUtDc3REW8LoxTv");

    console.log(`👤 Buyer address: ${buyerPublicKey.toString()}`);
    console.log(`🏪 Merchant address: ${merchantPublicKey.toString()}`);

    // 1. Test buyer order query
    console.log("\n" + "=".repeat(60));
    console.log("📋 Buyer Order Query Test");
    console.log("=".repeat(60));

    const buyerOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📊 Buyer order query results:`);
    console.log(`   Total orders: ${buyerOrders.totalCount}`);
    console.log(`   Current page: ${buyerOrders.page + 1}`);
    console.log(`   Page size: ${buyerOrders.pageSize}`);
    console.log(`   Has next page: ${buyerOrders.hasNext ? "Yes" : "No"}`);
    console.log(`   Has previous page: ${buyerOrders.hasPrev ? "Yes" : "No"}`);

    if (buyerOrders.orders.length > 0) {
      buyerOrders.orders.forEach((order, index) => {
        console.log(`\n📦 Order ${index + 1}:`);
        console.log(`   Order sequence: ${order.buyerSequence}`);
        console.log(`   Merchant: ${order.merchant}`);
        console.log(`   Product ID: ${order.productId}`);
        console.log(`   Quantity: ${order.quantity}`);
        console.log(`   Total price: ${order.totalAmount} TOKEN`);
        console.log(`   Status: ${order.status}`);
        console.log(`   Created at: ${new Date(order.createdAt * 1000).toLocaleString()}`);
        console.log(`   Order PDA: ${order.orderPDA}`);
      });
    } else {
      console.log("   📭 No orders found");
    }

    // 2. Test merchant order query
    console.log("\n" + "=".repeat(60));
    console.log("🏪 Merchant Order Query Test");
    console.log("=".repeat(60));

    const merchantOrders = await queryService.getMerchantOrders({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📊 Merchant order query results:`);
    console.log(`   Total orders: ${merchantOrders.totalCount}`);
    console.log(`   Current page: ${merchantOrders.page + 1}`);
    console.log(`   Page size: ${merchantOrders.pageSize}`);
    console.log(`   Has next page: ${merchantOrders.hasNext ? "Yes" : "No"}`);
    console.log(`   Has previous page: ${merchantOrders.hasPrev ? "Yes" : "No"}`);

    if (merchantOrders.orders.length > 0) {
      merchantOrders.orders.forEach((order, index) => {
        console.log(`\n🛍️ Order ${index + 1}:`);
        console.log(`   Merchant sequence: ${order.merchantSequence}`);
        console.log(`   Buyer sequence: ${order.buyerSequence}`);
        console.log(`   Buyer: ${order.buyer}`);
        console.log(`   Product ID: ${order.productId}`);
        console.log(`   Quantity: ${order.quantity}`);
        console.log(`   Total price: ${order.totalAmount} TOKEN`);
        console.log(`   Status: ${order.status}`);
        console.log(`   Created at: ${new Date(order.createdAt * 1000).toLocaleString()}`);
        console.log(`   Order PDA: ${order.orderPDA}`);
      });
    } else {
      console.log("   📭 No orders found");
    }

    // 3. Test pagination query
    if (buyerOrders.totalCount > 1) {
      console.log("\n" + "=".repeat(60));
      console.log("📄 Pagination Query Test");
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

      console.log(`\n📄 First page orders:`);
      if (page1.orders.length > 0) {
        console.log(`   Order sequence: ${page1.orders[0].buyerSequence}`);
        console.log(`   Status: ${page1.orders[0].status}`);
      }

      console.log(`\n📄 Second page orders:`);
      if (page2.orders.length > 0) {
        console.log(`   Order sequence: ${page2.orders[0].buyerSequence}`);
        console.log(`   Status: ${page2.orders[0].status}`);
      }
    }

    // 4. Test sorting functionality
    console.log("\n" + "=".repeat(60));
    console.log("🔄 Sorting Functionality Test");
    console.log("=".repeat(60));

    const ascOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "asc",
    });

    console.log(`\n📈 Ascending sort results:`);
    ascOrders.orders.forEach((order, index) => {
      console.log(`   ${index + 1}. Sequence: ${order.buyerSequence}, Status: ${order.status}`);
    });

    const descOrders = await queryService.getBuyerOrders({
      buyer: buyerPublicKey,
      page: 0,
      pageSize: 10,
      sortOrder: "desc",
    });

    console.log(`\n📉 Descending sort results:`);
    descOrders.orders.forEach((order, index) => {
      console.log(`   ${index + 1}. Sequence: ${order.buyerSequence}, Status: ${order.status}`);
    });

    console.log("\n🎉 Order query system test completed!");
  } catch (error) {
    console.error("❌ Test failed:", error);
    process.exit(1);
  }
}

// Run main function
if (require.main === module) {
  main().catch(console.error);
}
