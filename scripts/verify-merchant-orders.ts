import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { SolanaECommerce } from "../target/types/solana_e_commerce";

/**
 * Verify merchant order query functionality
 */
class MerchantOrderVerifier {
  private connection: Connection;
  private program: anchor.Program<SolanaECommerce>;

  constructor() {
    // Connect to local network
    this.connection = new Connection("http://localhost:8899", "confirmed");

    // Set provider
    const provider = new anchor.AnchorProvider(
      this.connection,
      new anchor.Wallet(anchor.web3.Keypair.generate()),
      { commitment: "confirmed" }
    );
    anchor.setProvider(provider);

    // Initialize program
    this.program = anchor.workspace.SolanaECommerce as anchor.Program<SolanaECommerce>;
  }

  /**
   * Calculate PDA
   */
  private calculatePDA(seeds: (string | Buffer)[]): [PublicKey, number] {
    const seedBuffers = seeds.map((seed) =>
      typeof seed === "string" ? Buffer.from(seed, "utf8") : seed
    );
    return PublicKey.findProgramAddressSync(seedBuffers, this.program.programId);
  }

  /**
   * Query all merchant orders
   */
  async queryMerchantOrders(merchantPublicKey: PublicKey): Promise<void> {
    console.log(`\n🔍 Querying merchant orders: ${merchantPublicKey.toString()}`);
    console.log("=====================================");

    try {
      // 1. Query merchant order count account
      const [merchantOrderCountPDA] = this.calculatePDA([
        "merchant_order_count",
        merchantPublicKey.toBuffer(),
      ]);

      console.log(`📊 Merchant order count PDA: ${merchantOrderCountPDA.toString()}`);

      let totalOrders = 0;
      try {
        const merchantOrderCountAccount = await this.program.account.merchantOrderCount.fetch(
          merchantOrderCountPDA
        );
        totalOrders = merchantOrderCountAccount.totalOrders.toNumber();
        console.log(`📈 Total merchant orders: ${totalOrders}`);
      } catch (error) {
        console.log(`❌ Merchant order count account does not exist or cannot be retrieved`);
        return;
      }

      // 2. Query each merchant order
      for (let i = 1; i <= totalOrders; i++) {
        const [merchantOrderPDA] = this.calculatePDA([
          "merchant_order",
          merchantPublicKey.toBuffer(),
          new anchor.BN(i).toArrayLike(Buffer, "le", 8),
        ]);

        console.log(`\n🏪 Merchant order ${i}:`);
        console.log(`   PDA: ${merchantOrderPDA.toString()}`);

        try {
          const merchantOrderAccount = await this.program.account.merchantOrder.fetch(
            merchantOrderPDA
          );

          console.log(`   Merchant: ${merchantOrderAccount.merchant.toString()}`);
          console.log(`   Buyer: ${merchantOrderAccount.buyer.toString()}`);
          console.log(`   Sequence: ${merchantOrderAccount.merchantOrderSequence.toNumber()}`);
          console.log(`   Buyer order PDA: ${merchantOrderAccount.buyerOrderPda.toString()}`);
          console.log(`   Product ID: ${merchantOrderAccount.productId.toNumber()}`);
          console.log(
            `   Created at: ${new Date(
              merchantOrderAccount.createdAt.toNumber() * 1000
            ).toLocaleString()}`
          );

          // 3. Query corresponding buyer order details
          try {
            const buyerOrderAccount = await this.program.account.order.fetch(
              merchantOrderAccount.buyerOrderPda
            );

            console.log(`   📦 Buyer order details:`);
            console.log(`      Quantity: ${buyerOrderAccount.quantity}`);
            console.log(`      Price: ${buyerOrderAccount.price.toNumber()}`);
            console.log(`      Total amount: ${buyerOrderAccount.totalAmount.toNumber()}`);
            console.log(`      Status: ${JSON.stringify(buyerOrderAccount.status)}`);
            console.log(`      Shipping address: ${buyerOrderAccount.shippingAddress}`);
            console.log(`      Notes: ${buyerOrderAccount.notes}`);
            if (buyerOrderAccount.trackingNumber) {
              console.log(`      Tracking number: ${buyerOrderAccount.trackingNumber}`);
            }
          } catch (error) {
            console.log(`   ❌ Failed to fetch buyer order details: ${error}`);
          }
        } catch (error) {
          console.log(`   ❌ Failed to fetch merchant order: ${error}`);
        }
      }
    } catch (error) {
      console.log(`❌ Failed to query merchant orders: ${error}`);
    }
  }

  /**
   * Main verification function
   */
  async verify(): Promise<void> {
    console.log("🔍 Starting merchant order query verification");
    console.log("=====================================");

    // Use merchant public key from local test (from test log)
    const merchantPublicKey = new PublicKey("DxuLkWihYH5VCJZS8dmFqCCkHv7zmMYwUVPN5cv2Lw9Q");

    await this.queryMerchantOrders(merchantPublicKey);

    console.log("\n✅ Merchant order query verification completed");
  }
}

// Execute verification
async function main() {
  const verifier = new MerchantOrderVerifier();
  await verifier.verify();
}

main().catch(console.error);
