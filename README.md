# Bloom Allowlist Project Structure

## Step-by-Step Setup

### 1. Create Project Structure

```bash
# Create new Rust project
cargo new bloom-allowlist-guard
cd bloom-allowlist-guard

# Create migrations directory
mkdir -p migrations
```

### 2. Add Dependencies to Cargo.toml

```toml
[package]
name = "bloom-allowlist-guard"
description = "A high-throughput allowlist verification service for NFT mints and airdrops that uses in-memory Bloom filters to reject ineligible wallets before they impact the database."
authors = ["mrathod05 <mrathod05@github>"]
license = "MIT OR Apache-2.0"
version = "0.1.0"
edition = "2024"

[dependencies]
diesel = { version = "2.1", features = ["postgres", "r2d2"] }
diesel-async = { version = "0.4", features = ["postgres", "deadpool"] }
bloomfilter = "1.0"
tokio = { version = "1", features = ["full"] }
dotenv = "0.15"
anyhow = "1.0"
rand = "0.8"
```

### 3. Setup Environment

Create `.env` file:

```bash
DATABASE_URL=postgresql://username:password@localhost/bloom_allowlist
```

### 4. Install Diesel CLI

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 5. Initialize Diesel

```bash
# This creates diesel.toml and migrations/ directory
diesel setup
```

### 6. Create Migration

```bash
diesel migration generate create_bloom_allowlist
```

This creates a timestamped directory in `migrations/` with `up.sql` and `down.sql` files.

### 7. Add Migration SQL

Copy the SQL content into the generated migration files:

**up.sql** - Creates the table
**down.sql** - Drops the table

### 8. Run Migration

```bash
# Apply migrations
diesel migration run

# Verify
diesel migration list
```

### 9. Generate Schema

Diesel automatically generates `src/schema.rs`:

```bash
# Manual regeneration if needed
diesel print-schema > src/schema.rs
```

### 10. Run Application

```bash
cargo run
```

## Common Diesel Commands

```bash
# Setup database and run all migrations
diesel setup

# Create new migration
diesel migration generate <migration_name>

# Apply all pending migrations
diesel migration run

# Revert last migration
diesel migration revert

# Redo last migration (revert + run)
diesel migration redo

# List migration status
diesel migration list

# Regenerate schema.rs
diesel print-schema > src/schema.rs

# Drop database
diesel database reset
```

## Troubleshooting

### Issue: "could not find `schema` in the crate root"

**Solution**: Make sure `schema.rs` exists in `src/` directory. Run:
```bash
diesel migration run
```

### Issue: "Database does not exist"

**Solution**: Create the database first:
```bash
diesel setup
```

Or manually:
```bash
createdb bloom_allowlist_db
```

### Issue: Migration fails

**Solution**: Check PostgreSQL is running and credentials are correct:
```bash
psql $DATABASE_URL -c "SELECT 1;"
```

## .gitignore

Add to your `.gitignore`:

```
/target
.env
*.db
*.sqlite
```

## Database Connection Formats

```bash
# Local PostgreSQL
DATABASE_URL=postgresql://user:password@localhost/dbname

# PostgreSQL with port
DATABASE_URL=postgresql://user:password@localhost:5432/dbname

# PostgreSQL with host
DATABASE_URL=postgresql://user:password@host.example.com/dbname
```