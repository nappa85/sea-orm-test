use std::{collections::HashMap, future::Future};

use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

type Money = f64;

pub struct VoidError;

impl From<()> for VoidError {
    fn from(_: ()) -> Self {
        VoidError
    }
}

impl std::fmt::Display for VoidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::fmt::Debug for VoidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for VoidError {}

struct Yadda {
    currency: String,
    amount: Money,
}

async fn get_conn() -> DatabaseConnection {
    let url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL env var");
    Database::connect(&url).await.unwrap()
}

async fn do_something_with_ref<'a, C, I>(_conn: &C, iter: I) -> Result<HashMap<&'a str, Money>, ()>
where
    C: ConnectionTrait,
    I: Iterator<Item=&'a Yadda>,
{
    Ok(iter.map(|y| (y.currency.as_str(), y.amount)).collect())
}

async fn do_something_else_with_ref<'a, C, I>(_conn: &C, iter: I) -> Result<HashMap<&'a str, Money>, ()>
where
    C: ConnectionTrait,
    I: Iterator<Item=&'a Yadda>,
{
    Ok(iter.map(|y| (y.currency.as_str(), y.amount)).collect())
}

async fn fake_transaction<'a, C, F, Fut, T, E>(conn: &'a C, callback: F) -> Result<T, E>
where
    F: FnOnce(&'a C) -> Fut,
    Fut: Future<Output=Result<T, E>>,
{
    callback(conn).await
}

async fn transaction_nightmare<'a>(yaddayadda: &'a [Yadda]) -> Result<HashMap<&'a str, Money>, ()> {
    let conn = get_conn().await;
    let (foo, bar) = fake_transaction::<_, _, _, _, VoidError>(&conn, |txn| async move {
        let foo = do_something_with_ref(txn, yaddayadda.iter()).await?;
        let bar = do_something_else_with_ref(txn, yaddayadda.iter()).await?;
        Ok((foo, bar))
    }).await
        .map_err(|e| eprintln!("{}", e))?;
    let mut res = HashMap::new();
    for (k, v) in foo.into_iter().chain(bar.into_iter()) {
        let entry = res.entry(k).or_insert(0.0);
        *entry += v;
    }
    Ok(res)
}

#[tokio::main]
async fn main() {
    let temp = vec![];
    transaction_nightmare(&temp).await.unwrap();
}
