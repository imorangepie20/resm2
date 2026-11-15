use sqlx::{PgPool, query, query_scalar, AssertSqlSafe};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPool::connect(&database_url).await?;

    // Get all table names first
    let tables: Vec<String> = query_scalar("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
        .fetch_all(&pool)
        .await?;

    for table in tables {
        if table != "spatial_ref_sys" {
            let sql = format!("TRUNCATE TABLE \"{}\" RESTART IDENTITY CASCADE;", table);
            query(AssertSqlSafe(sql.as_str())).execute(&pool).await?;
        }
    }

    println!("Database cleaned");
    Ok(())
}