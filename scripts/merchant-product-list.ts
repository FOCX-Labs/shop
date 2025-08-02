import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { SolanaECommerce } from "../target/types/solana_e_commerce";
import fs from "fs";

// Merchant product query parameters interface
interface MerchantProductQueryParams {
  merchant: PublicKey;
  page?: number;
  pageSize?: number;
  sortBy?: "created_at" | "updated_at" | "price" | "sales" | "inventory";
  sortOrder?: "asc" | "desc";
  isActive?: boolean;
  priceRange?: { min: number; max: number };
  keyword?: string;
}

// Product details interface
interface ProductWithDetails {
  id: number;
  merchant: string;
  name: string;
  description: string;
  price: string;
  keywords: string[];
  inventory: number;
  sales: number;
  isActive: boolean;
  createdAt: number;
  updatedAt: number;
  paymentToken: string;
  shippingLocation: string;
  productPDA: string;
  keywordArray?: string[];
}

// Paginated product list interface
interface PaginatedProductList {
  products: ProductWithDetails[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasNext: boolean;
  hasPrev: boolean;
  sortBy?: string;
  sortOrder?: "asc" | "desc";
}

// Merchant product query service class
class MerchantProductQueryService {
  private program: Program<SolanaECommerce>;
  private connection: Connection;

  constructor(program: Program<SolanaECommerce>, connection: Connection) {
    this.program = program;
    this.connection = connection;
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
   * Get merchant product list using new seed rules
   */
  async getMerchantProducts(params: MerchantProductQueryParams): Promise<PaginatedProductList> {
    const { merchant, page = 0, pageSize = 20, sortBy = "created_at", sortOrder = "desc" } = params;

    console.log(`🔍 Querying merchant product list using new seed rules:`);
    console.log(`   Merchant: ${merchant.toString()}`);
    console.log(`   Page: ${page}, Page size: ${pageSize}`);
    console.log(`   Sort: ${sortBy} ${sortOrder}`);

    try {
      // 1. First get merchant info to find product_count
      const merchantInfoPDA = this.calculatePDA(["merchant_info", merchant.toBuffer()])[0];

      console.log(`🏪 Merchant info PDA: ${merchantInfoPDA.toString()}`);

      let merchantInfo: any;
      try {
        merchantInfo = await this.program.account.merchant.fetch(merchantInfoPDA);
        console.log(
          `✅ Found merchant info, product_count: ${merchantInfo.productCount.toNumber()}`
        );
      } catch (error) {
        console.log(`⚠️ Merchant info not found, assuming no products`);
        return {
          products: [],
          totalCount: 0,
          page,
          pageSize,
          hasNext: false,
          hasPrev: false,
          sortBy,
          sortOrder,
        };
      }

      const productCount = merchantInfo.productCount.toNumber();
      console.log(`📊 Total products for merchant: ${productCount}`);

      // 2. Query each product using new PDA calculation
      const products: ProductWithDetails[] = [];

      for (let i = 0; i < productCount; i++) {
        try {
          // Calculate product PDA using new seed rules: ["product", merchant_pubkey, product_count]
          const productCountBytes = new anchor.BN(i).toArray("le", 8);
          const [productPDA] = this.calculatePDA([
            "product",
            merchant.toBuffer(),
            Buffer.from(productCountBytes),
          ]);

          console.log(`🔍 Querying product ${i}: ${productPDA.toString()}`);

          // Try to fetch the product account
          const productData = await this.program.account.productBase.fetch(productPDA);
          const formattedProduct = this.formatProductData(productData, productPDA);
          products.push(formattedProduct);

          console.log(`✅ Successfully loaded product ${i}: ${productData.name}`);
        } catch (error) {
          console.warn(`⚠️ Failed to load product ${i}:`, error);
          // Continue to next product instead of failing completely
        }
      }

      console.log(`✅ Successfully loaded ${products.length} out of ${productCount} products`);

      // 3. Apply filter conditions
      let filteredProducts = this.applyFilters(products, params);
      console.log(`🔍 Product count after filtering: ${filteredProducts.length}`);

      // 4. Sort
      filteredProducts = this.sortProducts(filteredProducts, sortBy, sortOrder);

      // 5. Pagination
      const startIndex = page * pageSize;
      const endIndex = startIndex + pageSize;
      const paginatedProducts = filteredProducts.slice(startIndex, endIndex);

      console.log(
        `📄 Pagination result: ${startIndex}-${endIndex}, actually returned: ${paginatedProducts.length}`
      );

      return {
        products: paginatedProducts,
        totalCount: filteredProducts.length,
        page,
        pageSize,
        hasNext: endIndex < filteredProducts.length,
        hasPrev: page > 0,
        sortBy,
        sortOrder,
      };
    } catch (error) {
      console.error(`❌ Failed to get merchant products:`, error);
      throw error;
    }
  }

  /**
   * Format product data
   */
  private formatProductData(productData: any, productPDA: PublicKey): ProductWithDetails {
    // Handle keywords - they might be stored as an array or string
    let keywordArray: string[] = [];
    if (productData.keywords) {
      if (Array.isArray(productData.keywords)) {
        keywordArray = productData.keywords.filter((k: string) => k && k.trim());
      } else if (typeof productData.keywords === "string") {
        keywordArray = productData.keywords.split(",").filter((k: string) => k.trim());
      }
    }

    return {
      id: productData.id ? productData.id.toNumber() : 0,
      merchant: productData.merchant.toString(),
      name: productData.name || "Unknown Product",
      description: productData.description || "",
      price: productData.price ? productData.price.toString() : "0",
      keywords: keywordArray,
      inventory: productData.inventory ? productData.inventory.toNumber() : 0,
      sales: productData.sales || 0,
      isActive: productData.isActive !== undefined ? productData.isActive : true,
      createdAt: productData.createdAt ? productData.createdAt.toNumber() : 0,
      updatedAt: productData.updatedAt ? productData.updatedAt.toNumber() : 0,
      paymentToken: productData.paymentToken ? productData.paymentToken.toString() : "",
      shippingLocation: productData.shippingLocation || "",
      productPDA: productPDA.toString(),
      keywordArray,
    };
  }

  /**
   * Apply filter conditions
   */
  private applyFilters(
    products: ProductWithDetails[],
    params: MerchantProductQueryParams
  ): ProductWithDetails[] {
    let filtered = products;

    // Active status filter
    if (params.isActive !== undefined) {
      filtered = filtered.filter((product) => product.isActive === params.isActive);
    }

    // Price range filter
    if (params.priceRange) {
      filtered = filtered.filter((product) => {
        const price = parseInt(product.price);
        return price >= params.priceRange!.min && price <= params.priceRange!.max;
      });
    }

    // Keyword filter
    if (params.keyword) {
      const keyword = params.keyword.toLowerCase();
      filtered = filtered.filter(
        (product) =>
          product.name.toLowerCase().includes(keyword) ||
          product.description.toLowerCase().includes(keyword) ||
          product.keywordArray?.some((k) => k.toLowerCase().includes(keyword))
      );
    }

    return filtered;
  }

  /**
   * Sort products
   */
  private sortProducts(
    products: ProductWithDetails[],
    sortBy: string,
    sortOrder: "asc" | "desc"
  ): ProductWithDetails[] {
    return products.sort((a, b) => {
      let comparison = 0;

      switch (sortBy) {
        case "created_at":
          comparison = a.createdAt - b.createdAt;
          break;
        case "updated_at":
          comparison = a.updatedAt - b.updatedAt;
          break;
        case "price":
          comparison = parseInt(a.price) - parseInt(b.price);
          break;
        case "sales":
          comparison = a.sales - b.sales;
          break;
        case "inventory":
          comparison = a.inventory - b.inventory;
          break;
        default:
          comparison = a.createdAt - b.createdAt;
      }

      return sortOrder === "desc" ? -comparison : comparison;
    });
  }

  /**
   * 获取商户产品统计信息
   */
  async getMerchantProductStats(merchant: PublicKey) {
    console.log(`📊 获取商户产品统计信息: ${merchant.toString()}`);

    try {
      // 获取所有产品用于统计
      const allProducts = await this.getMerchantProducts({
        merchant,
        pageSize: 1000, // 获取所有产品
      });

      const stats = {
        totalProducts: allProducts.totalCount,
        activeProducts: allProducts.products.filter((p) => p.isActive).length,
        inactiveProducts: allProducts.products.filter((p) => !p.isActive).length,
        totalInventory: allProducts.products.reduce((sum, p) => sum + p.inventory, 0),
        totalSales: allProducts.products.reduce((sum, p) => sum + p.sales, 0),
        averagePrice:
          allProducts.products.length > 0
            ? allProducts.products.reduce((sum, p) => sum + parseInt(p.price), 0) /
              allProducts.products.length
            : 0,
        priceRange: {
          min:
            allProducts.products.length > 0
              ? Math.min(...allProducts.products.map((p) => parseInt(p.price)))
              : 0,
          max:
            allProducts.products.length > 0
              ? Math.max(...allProducts.products.map((p) => parseInt(p.price)))
              : 0,
        },
      };

      console.log(`✅ 统计信息:`, stats);
      return stats;
    } catch (error) {
      console.error(`❌ 获取统计信息失败:`, error);
      throw error;
    }
  }
}

// 格式化Token金额显示
function formatTokenAmount(amount: string): string {
  const num = parseInt(amount);
  return (num / 1000000).toFixed(6); // 假设6位小数
}

// 格式化时间显示
function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

// 显示产品详情
function displayProduct(product: ProductWithDetails, index: number) {
  console.log(`\n📦 产品 ${index + 1}:`);
  console.log(`   ID: ${product.id}`);
  console.log(`   名称: ${product.name}`);
  console.log(`   描述: ${product.description}`);
  console.log(`   价格: ${formatTokenAmount(product.price)} TOKEN`);
  console.log(`   库存: ${product.inventory}`);
  console.log(`   销量: ${product.sales}`);
  console.log(`   状态: ${product.isActive ? "✅ 激活" : "❌ 停用"}`);
  console.log(`   关键词: ${product.keywords.join(", ")}`);
  console.log(`   发货地址: ${product.shippingLocation}`);
  console.log(`   创建时间: ${formatTimestamp(product.createdAt)}`);
  console.log(`   更新时间: ${formatTimestamp(product.updatedAt)}`);
  console.log(`   PDA地址: ${product.productPDA}`);
}

// 主函数
async function main() {
  console.log("🚀 商户产品列表查询测试开始");

  // 设置网络代理
  process.env.https_proxy = "http://127.0.0.1:7890";
  process.env.http_proxy = "http://127.0.0.1:7890";

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

  const program = anchor.workspace.SolanaECommerce as Program<SolanaECommerce>;
  const queryService = new MerchantProductQueryService(program, connection);

  try {
    // 使用enhanced-business-flow.ts中创建的商户地址
    const merchantPublicKey = new PublicKey("EqmeCvUSfz3puTw4LdsYNEVMrHn7fuUtDc3REW8LoxTv");

    console.log(`👤 商户地址: ${merchantPublicKey.toString()}`);

    // 1. 获取商户产品统计信息
    console.log("\n" + "=".repeat(60));
    console.log("📊 商户产品统计信息");
    console.log("=".repeat(60));

    const stats = await queryService.getMerchantProductStats(merchantPublicKey);

    console.log(`📈 产品统计:`);
    console.log(`   总产品数: ${stats.totalProducts}`);
    console.log(`   激活产品: ${stats.activeProducts}`);
    console.log(`   停用产品: ${stats.inactiveProducts}`);
    console.log(`   总库存: ${stats.totalInventory}`);
    console.log(`   总销量: ${stats.totalSales}`);
    console.log(`   平均价格: ${formatTokenAmount(stats.averagePrice.toString())} TOKEN`);
    console.log(
      `   价格范围: ${formatTokenAmount(stats.priceRange.min.toString())} - ${formatTokenAmount(
        stats.priceRange.max.toString()
      )} TOKEN`
    );

    if (stats.totalProducts === 0) {
      console.log("\n⚠️ 该商户暂无产品，请先创建产品后再测试查询功能");
      return;
    }

    // 2. 基础产品列表查询（第一页）
    console.log("\n" + "=".repeat(60));
    console.log("📋 基础产品列表查询（第一页，按创建时间降序）");
    console.log("=".repeat(60));

    const basicQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 5,
      sortBy: "created_at",
      sortOrder: "desc",
    });

    console.log(`\n📄 分页信息:`);
    console.log(`   当前页: ${basicQuery.page + 1}`);
    console.log(`   页大小: ${basicQuery.pageSize}`);
    console.log(`   总数量: ${basicQuery.totalCount}`);
    console.log(`   有下一页: ${basicQuery.hasNext ? "是" : "否"}`);
    console.log(`   有上一页: ${basicQuery.hasPrev ? "是" : "否"}`);

    basicQuery.products.forEach((product, index) => {
      displayProduct(product, index);
    });

    // 3. 按价格排序查询
    console.log("\n" + "=".repeat(60));
    console.log("💰 按价格升序排序查询");
    console.log("=".repeat(60));

    const priceQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 3,
      sortBy: "price",
      sortOrder: "asc",
    });

    priceQuery.products.forEach((product, index) => {
      console.log(
        `\n💰 产品 ${index + 1}: ${product.name} - ${formatTokenAmount(product.price)} TOKEN`
      );
    });

    // 4. 按销量排序查询
    console.log("\n" + "=".repeat(60));
    console.log("🔥 按销量降序排序查询");
    console.log("=".repeat(60));

    const salesQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 3,
      sortBy: "sales",
      sortOrder: "desc",
    });

    salesQuery.products.forEach((product, index) => {
      console.log(`\n🔥 产品 ${index + 1}: ${product.name} - 销量: ${product.sales}`);
    });

    // 5. 激活状态过滤查询
    console.log("\n" + "=".repeat(60));
    console.log("✅ 只查询激活状态的产品");
    console.log("=".repeat(60));

    const activeQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      isActive: true,
      pageSize: 10,
    });

    console.log(`✅ 激活产品数量: ${activeQuery.totalCount}`);
    activeQuery.products.slice(0, 3).forEach((product, index) => {
      console.log(`   ${index + 1}. ${product.name} (${product.isActive ? "激活" : "停用"})`);
    });

    console.log("\n🎉 商户产品列表查询测试完成！");
  } catch (error) {
    console.error("❌ 测试失败:", error);
    process.exit(1);
  }
}

// 运行主函数
if (require.main === module) {
  main().catch(console.error);
}

export {
  MerchantProductQueryService,
  type MerchantProductQueryParams,
  type ProductWithDetails,
  type PaginatedProductList,
};
