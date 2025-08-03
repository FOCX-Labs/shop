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
   * Get merchant product statistics
   */
  async getMerchantProductStats(merchant: PublicKey) {
    console.log(`📊 Getting merchant product statistics: ${merchant.toString()}`);

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

      console.log(`✅ Statistics:`, stats);
      return stats;
    } catch (error) {
      console.error(`❌ Failed to get statistics:`, error);
      throw error;
    }
  }
}

// Format token amount display
function formatTokenAmount(amount: string): string {
  const num = parseInt(amount);
  return (num / 1000000).toFixed(6); // 假设6位小数
}

// Format timestamp display
function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

// Display product details
function displayProduct(product: ProductWithDetails, index: number) {
  console.log(`\n📦 Product ${index + 1}:`);
  console.log(`   ID: ${product.id}`);
  console.log(`   Name: ${product.name}`);
  console.log(`   Description: ${product.description}`);
  console.log(`   Price: ${formatTokenAmount(product.price)} TOKEN`);
  console.log(`   Inventory: ${product.inventory}`);
  console.log(`   Sales: ${product.sales}`);
  console.log(`   Status: ${product.isActive ? "✅ Active" : "❌ Inactive"}`);
  console.log(`   Keywords: ${product.keywords.join(", ")}`);
  console.log(`   Shipping location: ${product.shippingLocation}`);
  console.log(`   Created at: ${formatTimestamp(product.createdAt)}`);
  console.log(`   Updated at: ${formatTimestamp(product.updatedAt)}`);
  console.log(`   PDA address: ${product.productPDA}`);
}

// Main function
async function main() {
  console.log("🚀 Merchant product list query test started");

  // Set network proxy
  process.env.https_proxy = "http://127.0.0.1:7890";
  process.env.http_proxy = "http://127.0.0.1:7890";

  // Set up connection
  const connection = new Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );

  // Create wallet and provider
  const wallet = new anchor.Wallet(Keypair.generate()); // Temporary wallet, for query only
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaECommerce as Program<SolanaECommerce>;
  const queryService = new MerchantProductQueryService(program, connection);

  try {
    // Use merchant address created in enhanced-business-flow.ts
    const merchantPublicKey = new PublicKey("EqmeCvUSfz3puTw4LdsYNEVMrHn7fuUtDc3REW8LoxTv");

    console.log(`👤 Merchant address: ${merchantPublicKey.toString()}`);

    // 1. Get merchant product statistics
    console.log("\n" + "=".repeat(60));
    console.log("📊 Merchant Product Statistics");
    console.log("=".repeat(60));

    const stats = await queryService.getMerchantProductStats(merchantPublicKey);

    console.log(`📈 Product statistics:`);
    console.log(`   Total products: ${stats.totalProducts}`);
    console.log(`   Active products: ${stats.activeProducts}`);
    console.log(`   Inactive products: ${stats.inactiveProducts}`);
    console.log(`   Total inventory: ${stats.totalInventory}`);
    console.log(`   Total sales: ${stats.totalSales}`);
    console.log(`   Average price: ${formatTokenAmount(stats.averagePrice.toString())} TOKEN`);
    console.log(
      `   Price range: ${formatTokenAmount(stats.priceRange.min.toString())} - ${formatTokenAmount(
        stats.priceRange.max.toString()
      )} TOKEN`
    );

    if (stats.totalProducts === 0) {
      console.log("\n⚠️ This merchant has no products yet, please create products first before testing query functionality");
      return;
    }

    // 2. Basic product list query (first page)
    console.log("\n" + "=".repeat(60));
    console.log("📋 Basic Product List Query (First Page, Sorted by Creation Time Descending)");
    console.log("=".repeat(60));

    const basicQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 5,
      sortBy: "created_at",
      sortOrder: "desc",
    });

    console.log(`\n📄 Pagination info:`);
    console.log(`   Current page: ${basicQuery.page + 1}`);
    console.log(`   Page size: ${basicQuery.pageSize}`);
    console.log(`   Total count: ${basicQuery.totalCount}`);
    console.log(`   Has next page: ${basicQuery.hasNext ? "Yes" : "No"}`);
    console.log(`   Has previous page: ${basicQuery.hasPrev ? "Yes" : "No"}`);

    basicQuery.products.forEach((product, index) => {
      displayProduct(product, index);
    });

    // 3. Query sorted by price
    console.log("\n" + "=".repeat(60));
    console.log("💰 Query Sorted by Price Ascending");
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
        `\n💰 Product ${index + 1}: ${product.name} - ${formatTokenAmount(product.price)} TOKEN`
      );
    });

    // 4. Query sorted by sales
    console.log("\n" + "=".repeat(60));
    console.log("🔥 Query Sorted by Sales Descending");
    console.log("=".repeat(60));

    const salesQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      page: 0,
      pageSize: 3,
      sortBy: "sales",
      sortOrder: "desc",
    });

    salesQuery.products.forEach((product, index) => {
      console.log(`\n🔥 Product ${index + 1}: ${product.name} - Sales: ${product.sales}`);
    });

    // 5. Active status filter query
    console.log("\n" + "=".repeat(60));
    console.log("✅ Only Query Active Products");
    console.log("=".repeat(60));

    const activeQuery = await queryService.getMerchantProducts({
      merchant: merchantPublicKey,
      isActive: true,
      pageSize: 10,
    });

    console.log(`✅ Number of active products: ${activeQuery.totalCount}`);
    activeQuery.products.slice(0, 3).forEach((product, index) => {
      console.log(`   ${index + 1}. ${product.name} (${product.isActive ? "Active" : "Inactive"})`);
    });

    console.log("\n🎉 Merchant product list query test completed!");
  } catch (error) {
    console.error("❌ Test failed:", error);
    process.exit(1);
  }
}

// Run main function
if (require.main === module) {
  main().catch(console.error);
}

export {
  MerchantProductQueryService,
  type MerchantProductQueryParams,
  type ProductWithDetails,
  type PaginatedProductList,
};
