use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::system_program;
use anchor_client::{Client, Cluster};
use solana_e_commerce::state::*;
use std::rc::Rc;

#[tokio::test]
async fn test_split_instructions_dynamic_keywords() {
    println!("🧪 开始拆分指令动态关键词Rust集成测试");

    // 设置客户端
    let payer = Keypair::new();
    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::confirmed(),
    );
    let program = client.program(solana_e_commerce::id()).unwrap();

    println!("✅ 客户端初始化完成");
    println!("📍 程序ID: {}", solana_e_commerce::id());
    println!("👤 付款人: {}", program.payer());

    // 空投SOL到测试账户
    let airdrop_amount = 1000_000_000_000; // 1000 SOL
    let signature = program
        .rpc()
        .request_airdrop(&program.payer(), airdrop_amount)
        .unwrap();
    program.rpc().confirm_transaction(&signature).unwrap();
    println!("💰 空投完成: {} SOL", airdrop_amount / 1_000_000_000);

    // 初始化系统
    let global_root_pda = Pubkey::find_program_address(&[b"global_id_root"], &program.id()).0;

    let init_config = SystemConfig {
        max_products_per_shard: 1000,
        max_keywords_per_product: 10,
        chunk_size: 10000,
        bloom_filter_size: 1024,
        cache_ttl: 3600,
    };

    let init_signature = program
        .request()
        .accounts(solana_e_commerce::accounts::InitializeSystem {
            global_root: global_root_pda,
            payer: program.payer(),
            system_program: system_program::id(),
        })
        .args(solana_e_commerce::instruction::InitializeSystem { config: init_config })
        .send()
        .unwrap();

    program.rpc().confirm_transaction(&init_signature).unwrap();
    println!("🔧 系统初始化完成: {}", init_signature);

    // 注册商户
    let merchant_id_pda = Pubkey::find_program_address(
        &[b"merchant", program.payer().as_ref()],
        &program.id(),
    ).0;

    let merchant_info_pda = Pubkey::find_program_address(
        &[b"merchant_info", program.payer().as_ref()],
        &program.id(),
    ).0;

    let register_signature = program
        .request()
        .accounts(solana_e_commerce::accounts::RegisterMerchantAtomic {
            merchant: program.payer(),
            payer: program.payer(),
            system_program: system_program::id(),
        })
        .args(solana_e_commerce::instruction::RegisterMerchantAtomic {
            name: "拆分指令测试商户".to_string(),
            description: "Rust集成测试商户".to_string(),
        })
        .send()
        .unwrap();

    program.rpc().confirm_transaction(&register_signature).unwrap();
    println!("🏪 商户注册完成: {}", register_signature);

    // 测试用例：不同关键词数量
    let test_cases = vec![
        (vec!["Rust单关键词".to_string()], "1个关键词测试"),
        (vec!["Rust双关键词1".to_string(), "Rust双关键词2".to_string()], "2个关键词测试"),
        (vec!["Rust三关键词1".to_string(), "Rust三关键词2".to_string(), "Rust三关键词3".to_string()], "3个关键词测试"),
    ];

    let payment_token = Pubkey::new_unique(); // 模拟USDC
    let token_decimals = 6;

    for (test_index, (keywords, description)) in test_cases.iter().enumerate() {
        println!("\n🧪 测试用例 {}: {}", test_index + 1, description);

        // 获取当前产品ID
        let global_root: GlobalIdRoot = program.account(global_root_pda).unwrap();
        let next_product_id = global_root.last_global_id + 1;

        println!("🆔 下一个产品ID: {}", next_product_id);

        // 计算产品PDA
        let product_account_pda = Pubkey::find_program_address(
            &[b"product", next_product_id.to_string().as_bytes()],
            &program.id(),
        ).0;

        // 1. 创建基础产品
        let product_data = ProductData {
            name: format!("Rust拆分指令-{}", description),
            description: format!("Rust集成测试-{}", description),
            price: 1_000_000_000 + (test_index as u64) * 100_000_000, // 1 + 0.1*index SOL
            keywords: keywords.clone(),
            payment_token: *payment_token,
            token_decimals,
            token_price: 10_000_000 + (test_index as u64) * 1_000_000, // 10 + index USDC
        };

        let create_product_signature = program
            .request()
            .accounts(solana_e_commerce::accounts::CreateProductBasic {
                merchant: program.payer(),
                global_root: global_root_pda,
                merchant_id_account: merchant_id_pda,
                merchant_info: merchant_info_pda,
                product_account: product_account_pda,
                payer: program.payer(),
                system_program: system_program::id(),
                rent: anchor_client::solana_sdk::sysvar::rent::id(),
                clock: anchor_client::solana_sdk::sysvar::clock::id(),
            })
            .args(solana_e_commerce::instruction::CreateProductBasic {
                name: product_data.name.clone(),
                description: product_data.description.clone(),
                price: product_data.price,
                keywords: product_data.keywords.clone(),
                payment_token: product_data.payment_token,
                token_decimals: product_data.token_decimals,
                token_price: product_data.token_price,
            })
            .send()
            .unwrap();

        program.rpc().confirm_transaction(&create_product_signature).unwrap();
        println!("📦 基础产品创建完成: {}", create_product_signature);

        // 2. 动态创建关键词索引
        for (keyword_index, keyword) in keywords.iter().enumerate() {
            let keyword_root_pda = Pubkey::find_program_address(
                &[b"keyword_root", keyword.as_bytes()],
                &program.id(),
            ).0;

            let keyword_shard_pda = Pubkey::find_program_address(
                &[b"keyword_shard", keyword.as_bytes(), &[0, 0, 0, 0]],
                &program.id(),
            ).0;

            let create_keyword_signature = program
                .request()
                .accounts(solana_e_commerce::accounts::CreateKeywordIndexForProduct {
                    merchant: program.payer(),
                    product_account: product_account_pda,
                    keyword_root: keyword_root_pda,
                    keyword_shard: keyword_shard_pda,
                    payer: program.payer(),
                    system_program: system_program::id(),
                    rent: anchor_client::solana_sdk::sysvar::rent::id(),
                })
                .args(solana_e_commerce::instruction::CreateKeywordIndexForProduct {
                    product_id: next_product_id,
                    keyword: keyword.clone(),
                })
                .send()
                .unwrap();

            program.rpc().confirm_transaction(&create_keyword_signature).unwrap();
            println!("🔑 关键词{}索引创建完成: {} - {}", keyword_index + 1, keyword, create_keyword_signature);
        }

        // 3. 创建价格索引
        let price_range_start = (product_data.token_price / 100_000_000) * 100_000_000;
        let price_range_end = price_range_start + 100_000_000 - 1;

        let price_index_pda = Pubkey::find_program_address(
            &[
                b"price_index",
                &price_range_start.to_le_bytes(),
                &price_range_end.to_le_bytes(),
                next_product_id.to_string().as_bytes(),
            ],
            &program.id(),
        ).0;

        let create_price_signature = program
            .request()
            .accounts(solana_e_commerce::accounts::CreatePriceIndexForProduct {
                merchant: program.payer(),
                product_account: product_account_pda,
                price_index: price_index_pda,
                payer: program.payer(),
                system_program: system_program::id(),
                rent: anchor_client::solana_sdk::sysvar::rent::id(),
            })
            .args(solana_e_commerce::instruction::CreatePriceIndexForProduct {
                product_id: next_product_id,
                token_price: product_data.token_price,
            })
            .send()
            .unwrap();

        program.rpc().confirm_transaction(&create_price_signature).unwrap();
        println!("💰 价格索引创建完成: {}", create_price_signature);

        // 4. 创建销量索引
        let sales_index_pda = Pubkey::find_program_address(
            &[
                b"sales_index",
                &0u64.to_le_bytes(),
                &1000u64.to_le_bytes(),
                next_product_id.to_string().as_bytes(),
            ],
            &program.id(),
        ).0;

        let create_sales_signature = program
            .request()
            .accounts(solana_e_commerce::accounts::CreateSalesIndexForProduct {
                merchant: program.payer(),
                product_account: product_account_pda,
                sales_index: sales_index_pda,
                payer: program.payer(),
                system_program: system_program::id(),
                rent: anchor_client::solana_sdk::sysvar::rent::id(),
            })
            .args(solana_e_commerce::instruction::CreateSalesIndexForProduct {
                product_id: next_product_id,
            })
            .send()
            .unwrap();

        program.rpc().confirm_transaction(&create_sales_signature).unwrap();
        println!("📊 销量索引创建完成: {}", create_sales_signature);

        // 5. 验证创建结果
        let product_account: Product = program.account(product_account_pda).unwrap();
        assert_eq!(product_account.id, next_product_id);
        assert_eq!(product_account.name, product_data.name);
        assert_eq!(product_account.keywords, *keywords);
        assert_eq!(product_account.is_active, true);

        println!("✅ 产品验证成功 - ID: {}, 名称: {}, 关键词数: {}", 
                 product_account.id, product_account.name, product_account.keywords.len());

        // 验证关键词索引
        for keyword in keywords {
            let keyword_root_pda = Pubkey::find_program_address(
                &[b"keyword_root", keyword.as_bytes()],
                &program.id(),
            ).0;

            let keyword_root: KeywordRoot = program.account(keyword_root_pda).unwrap();
            assert_eq!(keyword_root.keyword, *keyword);
            assert_eq!(keyword_root.total_products, 1);
            println!("✅ 关键词索引验证成功: {}", keyword);
        }

        // 验证价格索引
        let price_index: PriceIndexNode = program.account(price_index_pda).unwrap();
        assert_eq!(price_index.price_range_start, price_range_start);
        assert_eq!(price_index.price_range_end, price_range_end);
        assert!(price_index.product_ids.contains(&next_product_id));
        println!("✅ 价格索引验证成功: 范围 {}-{}", price_range_start, price_range_end);

        // 验证销量索引
        let sales_index: SalesIndexNode = program.account(sales_index_pda).unwrap();
        assert_eq!(sales_index.sales_range_start, 0);
        assert_eq!(sales_index.sales_range_end, 1000);
        assert!(sales_index.product_ids.contains(&next_product_id));
        println!("✅ 销量索引验证成功: 范围 0-1000");

        println!("🎯 测试用例 {} 完全成功！", test_index + 1);
    }

    println!("\n🎉 所有拆分指令Rust集成测试完成！");
    println!("✅ 动态关键词数量支持验证成功");
    println!("✅ 所有索引账户创建和验证成功");
    println!("✅ 拆分指令方案完全可行");
}

// 辅助结构体
struct ProductData {
    name: String,
    description: String,
    price: u64,
    keywords: Vec<String>,
    payment_token: Pubkey,
    token_decimals: u8,
    token_price: u64,
}
