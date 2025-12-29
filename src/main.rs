pub mod models;
pub mod schema;

use bloomfilter::Bloom;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use dotenv::dotenv;
use anyhow::Result;
use rand::Rng;

use schema::bloom_allowlist::dsl::*;



// --- CONFIGURATION ---
const EXPECTED_ITEMS: usize = 100_000;
const FALSE_POSITIVE_RATE: f64 = 0.0001;

/// The "Guard" holds the state
struct AllowlistGuard {
    // Diesel Async Pool
    pool: Pool<AsyncPgConnection>,
    filter: RwLock<Bloom<String>>,
}

impl AllowlistGuard {
    /// Initialize: Connect, Migrate Data, and Hydrate Filter
    async fn new(db_url: &str) -> Result<Arc<Self>> {
        // 1. Setup Connection Pool
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        let pool = Pool::builder(config).build()?;

        println!("Connected to Postgres via Diesel.");

        let guard = Arc::new(Self {
            pool,
            filter: RwLock::new(Bloom::new_for_fp_rate(EXPECTED_ITEMS, FALSE_POSITIVE_RATE)),
        });

        // 2. Run Migration (Add dummy data if needed)
        guard.migrate_dummy_data(500).await?;

        // 3. Hydrate the Bloom Filter from DB
        guard.hydrate().await?;

        Ok(guard)
    }

    /// Helper to populate the Bloom Filter from the DB
    async fn hydrate(&self) -> Result<()> {
        let mut conn = self.pool.get().await?;

        // Diesel Select Query
        let results = bloom_allowlist
            .select(models::AllowlistEntry::as_select())
            .load(&mut conn)
            .await?;

        // Write to Bloom Filter
        let mut filter = self.filter.write().await;
        for entry in &results {
            filter.set(&entry.wallet_address);
        }

        println!("🌊 Hydrated Bloom Filter with {} wallets.", results.len());
        Ok(())
    }

    /// THE REQUESTED FUNCTION: Adds N dummy wallets
    async fn migrate_dummy_data(&self, count: usize) -> Result<()> {
        let mut conn = self.pool.get().await?;

        // Check current count to avoid duplicates on restart
        let current_count: i64 = bloom_allowlist.count().get_result(&mut conn).await?;
        if current_count >= count as i64 {
            println!("⏩ Database already has data ({}), skipping migration.", current_count);
            return Ok(());
        }

        println!("🏗️ Migrating {} dummy wallets into DB...", count);

        let mut new_entries = Vec::new();
        for _ in 0..count {
            // Generate random hex wallet
            let random_bytes: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(40)
                .map(char::from)
                .collect();

            new_entries.push(models::NewEntry {
                wallet_address: format!("0x{}", random_bytes),
            });
        }

        // Batch Insert
        diesel::insert_into(bloom_allowlist)
            .values(&new_entries)
            .execute(&mut conn)
            .await?;

        println!("✅ Migration Complete.");
        Ok(())
    }

    /// The High-Performance Check Logic
    async fn check_access(&self, wallet_to_check: &str) -> bool {
        // Step 1: Check Bloom Filter (RAM)
        let probably_exists = self.filter.read().await.check(&wallet_to_check.to_string());

        if !probably_exists {
            println!("🛑 [Blocked by Filter] {} is NOT allowlisted.", wallet_to_check);
            return false;
        }

        // Step 2: Check Postgres (Disk)
        println!("⚠️ [Filter Passed] Checking DB for {}...", wallet_to_check);
        let mut conn = self.pool.get().await.expect("Failed to get DB connection");

        // Diesel Query: SELECT count(*) FROM bloom_allowlist WHERE wallet_address = $1
        let exists: bool = diesel::select(diesel::dsl::exists(
            bloom_allowlist.filter(wallet_address.eq(wallet_to_check))
        ))
            .get_result(&mut conn)
            .await
            .unwrap_or(false);

        if exists {
            println!("✅ [DB Confirmed] Access Granted.");
        } else {
            println!("❌ [False Positive] DB rejected the request.");
        }

        exists
    }

    /// Add a single user
    async fn add_user(&self, new_wallet: &str) -> Result<()> {
        let mut conn = self.pool.get().await?;

        // 1. Insert into DB
        diesel::insert_into(bloom_allowlist)
            .values(models::NewEntry { wallet_address: new_wallet.to_string() })
            .execute(&mut conn)
            .await?;

        // 2. Update Filter
        self.filter.write().await.set(&new_wallet.to_string());
        println!("➕ Added {} to DB and Bloom Filter.", new_wallet);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let guard = AllowlistGuard::new(&db_url).await?;

    println!("\n--- STARTING SIMULATION ---\n");

    // 1. Pick a random wallet from the dummy data to test "Success"
    let mut conn = guard.pool.get().await?;
    let random_valid_wallet: String = bloom_allowlist
        .select(wallet_address)
        .first(&mut conn)
        .await?;

    // Test Valid
    guard.check_access(&random_valid_wallet).await;

    // 2. Test Invalid
    guard.check_access("0xHackerBot99999").await;

    // 3. Add dynamic user
    let new_user = "0xVIPUserForAirdrop";
    guard.add_user(new_user).await?;
    guard.check_access(new_user).await;

    Ok(())
}