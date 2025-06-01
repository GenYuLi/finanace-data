use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use yahoo_finance_api::{
    self as yahoo,
    time::{Duration, OffsetDateTime},
};

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://witherslin@%2Fvar%2Frun%2Fpostgresql/finance_db")
        .await?;
    // 使用 yahoo_finance_api 撈取 AAPL 股票於指定期間的歷史股價資料
    let provider = yahoo::YahooConnector::new()?;
    let end_date = OffsetDateTime::now_utc();
    let start_date = end_date - Duration::days(7);
    let response = provider
        .get_quote_history_interval("AAPL", start_date, end_date, "1m")
        .await;
    let quotes = response?.quotes()?;
    dbg!(&quotes[0]);
    dbg!(&quotes.len());

    for q in quotes {
        let naive = DateTime::from_timestamp(q.timestamp as i64, 0).unwrap_or_default();
        let dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive.naive_utc(), Utc);
        sqlx::query!(
            "INSERT INTO stock_data (timestamp, open, high, low, close, volume)
             VALUES ($1, $2, $3, $4, $5, $6)",
            dt,
            q.open,
            q.high,
            q.low,
            q.close,
            q.volume as i64,
        )
        .execute(&pool)
        .await?;
    }
    let end_date = OffsetDateTime::now_utc();
    let start_date = end_date - Duration::days(50);
    let response = provider
        .get_quote_history_interval("AAPL", start_date, end_date, "5m")
        .await;
    // dbg!(&response);
    let quotes = response?.quotes()?;
    
    // 將每筆資料寫入資料庫
    for q in quotes {
        // dbg!(&q);
        // 轉換 timestamp (秒數) 為 chrono 的 DateTime<Utc>
        let naive = DateTime::from_timestamp(q.timestamp as i64, 0).unwrap_or_default();
        let dt = chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive.naive_utc(), Utc);
        sqlx::query!(
            "INSERT INTO stock_data (timestamp, open, high, low, close, volume)
             VALUES ($1, $2, $3, $4, $5, $6)",
            dt,
            q.open,
            q.high,
            q.low,
            q.close,
            q.volume as i64,
        )
        .execute(&pool)
        .await?;
    }
    Ok(())
}
