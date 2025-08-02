import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { SolanaECommerce } from "../target/types/solana_e_commerce";
import { PublicKey, Connection } from "@solana/web3.js";

// Setup network proxy and environment variables
process.env.https_proxy = "http://127.0.0.1:7890";
process.env.http_proxy = "http://127.0.0.1:7890";
process.env.ANCHOR_PROVIDER_URL =
  "https://devnet.helius-rpc.com/?api-key=48e26d41-1ec0-4a29-ac33-fa26d0112cef";
process.env.ANCHOR_WALLET = "/Users/liudong/.config/solana/id.json";

// Data type definitions
interface OrderWithDetails {
  // Basic order information
  buyer: string;
  merchant: string;
  productId: number;
  quantity: number;
  totalAmount: string;
  status: string;
  createdAt: number;

  // PDA information
  orderPDA: string;
  merchantOrderPDA?: string;

  // Sequence number information
  buyerSequence: number;
  merchantSequence?: number;
}

interface PaginatedOrderList {
  orders: OrderWithDetails[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasNext: boolean;
  hasPrev: boolean;
}

interface BuyerOrderQueryParams {
  buyer: PublicKey;
  page?: number;
  pageSize?: number;
  sortOrder?: "asc" | "desc";
}

interface MerchantOrderQueryParams {
  merchant: PublicKey;
  page?: number;
  pageSize?: number;
  sortOrder?: "asc" | "desc";
}

class OrderQueryService {
  private program: Program<SolanaECommerce>;
  private connection: Connection;
  private readonly BATCH_SIZE = 10; // 批量查询大小

  constructor() {
    const provider = AnchorProvider.env();
    anchor.setProvider(provider);
    this.program = anchor.workspace.SolanaECommerce as Program<SolanaECommerce>;
    this.connection = provider.connection;
  }

  /**
   * Calculate PDA using the same logic as the program
   */
  private calculatePDA(seeds: (string | Buffer | Uint8Array)[]): [PublicKey, number] {
    const seedBuffers = seeds.map((seed) => {
      if (typeof seed === "string") {
        return Buffer.from(seed, "utf8");
      } else if (seed instanceof Uint8Array) {
        return Buffer.from(seed);
      } else {
        return seed;
      }
    });

    return PublicKey.findProgramAddressSync(seedBuffers, this.program.programId);
  }

  /**
   * 获取买家订单列表 - 使用新的PDA种子规则
   */
  async getBuyerOrders(params: BuyerOrderQueryParams): Promise<PaginatedOrderList> {
    const { buyer, page = 0, pageSize = 20, sortOrder = "desc" } = params;

    console.log(`🔍 查询买家订单列表 (使用新PDA种子规则):`);
    console.log(`   买家: ${buyer.toString()}`);
    console.log(`   页码: ${page}, 页大小: ${pageSize}, 排序: ${sortOrder}`);

    try {
      // 1. 获取买家购买总数 - 使用新的PDA计算方式
      const [userPurchaseCountPDA] = this.calculatePDA(["user_purchase_count", buyer.toBuffer()]);

      console.log(`📊 买家购买计数PDA: ${userPurchaseCountPDA.toString()}`);

      let totalOrders = 0;
      try {
        const userPurchaseCount = await this.program.account.userPurchaseCount.fetch(
          userPurchaseCountPDA
        );
        totalOrders = userPurchaseCount.purchaseCount.toNumber();
        console.log(`✅ 买家总购买次数: ${totalOrders}`);
      } catch (error) {
        console.log(`⚠️ 买家购买计数账户不存在，总订单数为0`);
        return {
          orders: [],
          totalCount: 0,
          page,
          pageSize,
          hasNext: false,
          hasPrev: false,
        };
      }

      if (totalOrders === 0) {
        return {
          orders: [],
          totalCount: 0,
          page,
          pageSize,
          hasNext: false,
          hasPrev: false,
        };
      }

      // 2. 计算查询范围
      const startIndex =
        sortOrder === "desc" ? Math.max(0, totalOrders - (page + 1) * pageSize) : page * pageSize;

      const endIndex =
        sortOrder === "desc"
          ? Math.max(0, totalOrders - page * pageSize)
          : Math.min(totalOrders, (page + 1) * pageSize);

      console.log(`📋 查询范围: ${startIndex} - ${endIndex}`);

      // 3. 批量获取订单
      const orders: OrderWithDetails[] = [];

      for (let i = startIndex; i < endIndex; i += this.BATCH_SIZE) {
        const batchEnd = Math.min(i + this.BATCH_SIZE, endIndex);
        console.log(`🔄 获取批次: ${i} - ${batchEnd}`);

        const batchOrders = await this.fetchBuyerOrderBatch(
          buyer,
          i,
          batchEnd,
          sortOrder,
          totalOrders
        );
        orders.push(...batchOrders);
      }

      console.log(`✅ 成功获取 ${orders.length} 个订单`);

      return {
        orders,
        totalCount: totalOrders,
        page,
        pageSize,
        hasNext: (page + 1) * pageSize < totalOrders,
        hasPrev: page > 0,
      };
    } catch (error) {
      console.error(`❌ 获取买家订单失败:`, error);
      throw error;
    }
  }

  /**
   * 获取商家订单列表 - 使用新的PDA种子规则
   */
  async getMerchantOrders(params: MerchantOrderQueryParams): Promise<PaginatedOrderList> {
    const { merchant, page = 0, pageSize = 20, sortOrder = "desc" } = params;

    console.log(`🔍 查询商家订单列表 (使用新PDA种子规则):`);
    console.log(`   商家: ${merchant.toString()}`);
    console.log(`   页码: ${page}, 页大小: ${pageSize}, 排序: ${sortOrder}`);

    try {
      // 1. 获取商家订单总数 - 使用新的PDA计算方式
      const [merchantOrderCountPDA] = this.calculatePDA([
        "merchant_order_count",
        merchant.toBuffer(),
      ]);

      console.log(`📊 商家订单计数PDA: ${merchantOrderCountPDA.toString()}`);

      let totalOrders = 0;
      try {
        const merchantOrderCount = await this.program.account.merchantOrderCount.fetch(
          merchantOrderCountPDA
        );
        totalOrders = merchantOrderCount.totalOrders.toNumber();
        console.log(`✅ 商家总订单数: ${totalOrders}`);
      } catch (error) {
        console.log(`⚠️ 商家订单计数账户不存在，总订单数为0`);
        return {
          orders: [],
          totalCount: 0,
          page,
          pageSize,
          hasNext: false,
          hasPrev: false,
        };
      }

      if (totalOrders === 0) {
        return {
          orders: [],
          totalCount: 0,
          page,
          pageSize,
          hasNext: false,
          hasPrev: false,
        };
      }

      // 2. 计算查询范围
      const startIndex =
        sortOrder === "desc" ? Math.max(0, totalOrders - (page + 1) * pageSize) : page * pageSize;

      const endIndex =
        sortOrder === "desc"
          ? Math.max(0, totalOrders - page * pageSize)
          : Math.min(totalOrders, (page + 1) * pageSize);

      console.log(`📋 查询范围: ${startIndex} - ${endIndex}`);

      // 3. 批量获取商家订单索引
      const merchantOrders: any[] = [];

      for (let i = startIndex; i < endIndex; i += this.BATCH_SIZE) {
        const batchEnd = Math.min(i + this.BATCH_SIZE, endIndex);
        console.log(`🔄 获取商家订单批次: ${i} - ${batchEnd}`);

        const batchMerchantOrders = await this.fetchMerchantOrderBatch(
          merchant,
          i,
          batchEnd,
          sortOrder,
          totalOrders
        );
        merchantOrders.push(...batchMerchantOrders);
      }

      // 4. 通过商家订单获取完整的买家订单详情
      console.log(`🔄 获取完整订单详情...`);
      const detailedOrders = await this.fetchDetailedOrdersFromMerchantOrders(merchantOrders);

      console.log(`✅ 成功获取 ${detailedOrders.length} 个订单`);

      return {
        orders: detailedOrders,
        totalCount: totalOrders,
        page,
        pageSize,
        hasNext: (page + 1) * pageSize < totalOrders,
        hasPrev: page > 0,
      };
    } catch (error) {
      console.error(`❌ 获取商家订单失败:`, error);
      throw error;
    }
  }

  /**
   * 批量获取买家订单
   */
  private async fetchBuyerOrderBatch(
    buyer: PublicKey,
    start: number,
    end: number,
    sortOrder: "asc" | "desc",
    totalOrders: number
  ): Promise<OrderWithDetails[]> {
    const promises: Promise<OrderWithDetails | null>[] = [];

    for (let i = start; i < end; i++) {
      const sequence =
        sortOrder === "desc"
          ? totalOrders - i // 倒序：从最新开始
          : i + 1; // 正序：从1开始

      promises.push(this.fetchSingleBuyerOrder(buyer, sequence));
    }

    const results = await Promise.allSettled(promises);
    return results
      .filter((result) => result.status === "fulfilled" && result.value !== null)
      .map((result) => (result as PromiseFulfilledResult<OrderWithDetails>).value);
  }

  /**
   * 获取单个买家订单 - 使用新的PDA种子规则
   */
  private async fetchSingleBuyerOrder(
    buyer: PublicKey,
    sequence: number
  ): Promise<OrderWithDetails | null> {
    try {
      // 计算买家订单PDA - 使用新的PDA计算方式
      const sequenceBytes = new anchor.BN(sequence).toArray("le", 8);
      const [orderPDA] = this.calculatePDA([
        "buyer_order",
        buyer.toBuffer(),
        Buffer.from(sequenceBytes),
      ]);

      console.log(`🔍 查询买家订单 ${sequence}: ${orderPDA.toString()}`);

      // 获取订单详情
      const order = await this.program.account.order.fetch(orderPDA);

      return {
        buyer: order.buyer.toString(),
        merchant: order.merchant.toString(),
        productId: order.productId ? order.productId.toNumber() : 0,
        quantity: order.quantity || 0,
        totalAmount: order.totalAmount ? order.totalAmount.toString() : "0",
        status: Object.keys(order.status)[0] || "unknown", // 获取枚举的键
        createdAt: order.createdAt ? order.createdAt.toNumber() : 0,
        orderPDA: orderPDA.toString(),
        merchantOrderPDA: order.merchantOrderPda ? order.merchantOrderPda.toString() : "",
        buyerSequence: sequence,
      };
    } catch (error) {
      console.warn(`⚠️ 获取买家订单失败 (序列号: ${sequence}):`, (error as any).message || error);
      return null;
    }
  }

  /**
   * 批量获取商家订单索引
   */
  private async fetchMerchantOrderBatch(
    merchant: PublicKey,
    start: number,
    end: number,
    sortOrder: "asc" | "desc",
    totalOrders: number
  ): Promise<any[]> {
    const promises: Promise<any | null>[] = [];

    for (let i = start; i < end; i++) {
      const sequence =
        sortOrder === "desc"
          ? totalOrders - i // 倒序：从最新开始
          : i + 1; // 正序：从1开始

      promises.push(this.fetchSingleMerchantOrder(merchant, sequence));
    }

    const results = await Promise.allSettled(promises);
    return results
      .filter((result) => result.status === "fulfilled" && result.value !== null)
      .map((result) => (result as PromiseFulfilledResult<any>).value);
  }

  /**
   * 获取单个商家订单索引 - 使用新的PDA种子规则
   */
  private async fetchSingleMerchantOrder(
    merchant: PublicKey,
    sequence: number
  ): Promise<any | null> {
    try {
      // 计算商家订单PDA - 使用新的PDA计算方式
      const sequenceBytes = new anchor.BN(sequence).toArray("le", 8);
      const [merchantOrderPDA] = this.calculatePDA([
        "merchant_order",
        merchant.toBuffer(),
        Buffer.from(sequenceBytes),
      ]);

      console.log(`🔍 查询商家订单 ${sequence}: ${merchantOrderPDA.toString()}`);

      // 获取商家订单索引
      const merchantOrder = await this.program.account.merchantOrder.fetch(merchantOrderPDA);

      return {
        ...merchantOrder,
        merchantOrderPDA: merchantOrderPDA.toString(),
        merchantSequence: sequence,
      };
    } catch (error) {
      console.warn(`⚠️ 获取商家订单失败 (序列号: ${sequence}):`, (error as any).message || error);
      return null;
    }
  }

  /**
   * 通过商家订单获取完整的买家订单详情
   */
  private async fetchDetailedOrdersFromMerchantOrders(
    merchantOrders: any[]
  ): Promise<OrderWithDetails[]> {
    const promises = merchantOrders.map(async (merchantOrder) => {
      try {
        // 通过买家订单PDA获取完整订单详情
        const buyerOrder = await this.program.account.order.fetch(
          new PublicKey(merchantOrder.buyerOrderPda)
        );

        return {
          buyer: buyerOrder.buyer.toString(),
          merchant: buyerOrder.merchant.toString(),
          productId: buyerOrder.productId.toNumber(),
          quantity: buyerOrder.quantity,
          totalAmount: buyerOrder.totalAmount.toString(),
          status: Object.keys(buyerOrder.status)[0],
          createdAt: buyerOrder.createdAt.toNumber(),
          orderPDA: merchantOrder.buyerOrderPda.toString(),
          merchantOrderPDA: merchantOrder.merchantOrderPDA,
          buyerSequence: (buyerOrder as any).buyerPurchaseSequence?.toNumber() || 0,
          merchantSequence: merchantOrder.merchantSequence,
        };
      } catch (error) {
        console.warn(`⚠️ 获取详细订单失败:`, (error as any).message || error);
        return null;
      }
    });

    const results = await Promise.allSettled(promises);
    return results
      .filter((result) => result.status === "fulfilled" && result.value !== null)
      .map((result) => (result as PromiseFulfilledResult<OrderWithDetails>).value);
  }
}

export { OrderQueryService };
export type {
  OrderWithDetails,
  PaginatedOrderList,
  BuyerOrderQueryParams,
  MerchantOrderQueryParams,
};
