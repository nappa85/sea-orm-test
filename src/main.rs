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

struct FakeTransaction<'x>(&'x DatabaseConnection);

#[async_trait::async_trait]
impl<'x> ConnectionTrait for FakeTransaction<'x> {
    fn as_mock_connection(&self) -> &sea_orm::MockDatabaseConnection {
        self.0.as_mock_connection()
    }

    async fn execute(&self, stmt: sea_orm::Statement) -> Result<sea_orm::ExecResult, sea_orm::DbErr> {
        self.0.execute(stmt).await
    }

    fn get_database_backend(&self) -> sea_orm::DbBackend {
        self.0.get_database_backend()
    }

    fn into_transaction_log(&self) -> Vec<sea_orm::Transaction> {
        self.0.into_transaction_log()
    }

    fn is_mock_connection(&self) -> bool {
        self.0.is_mock_connection()
    }

    async fn query_all(&self, stmt: sea_orm::Statement) -> Result<Vec<sea_orm::QueryResult>, sea_orm::DbErr> {
        self.0.query_all(stmt).await
    }

    async fn query_one(&self, stmt: sea_orm::Statement) -> Result<Option<sea_orm::QueryResult>, sea_orm::DbErr> {
        self.0.query_one(stmt).await
    }

    async fn transaction<'a, F, T, E>(&self, callback: F) -> Result<T, sea_orm::TransactionError<E>>
    where
            F: for<'c> FnOnce(&'c sea_orm::DatabaseTransaction<'_>) -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>> + 'a + Send + Sync,
            T: Send,
            E: std::error::Error + Send {
        self.0.transaction(callback).await
    }
}

async fn fake_transaction<'a: 'b, 'b, F, Fut, T, E>(conn: &'a DatabaseConnection, callback: F) -> Result<T, E>
where
    F: FnOnce(&'b FakeTransaction<'a>) -> Fut + 'a,
    Fut: Future<Output=Result<T, E>> + 'b,
{
    let transaction = FakeTransaction(conn);
    callback(&transaction).await
}

async fn transaction_nightmare<'a>(yaddayadda: &'a [Yadda]) -> Result<HashMap<&'a str, Money>, ()> {
    let conn = get_conn().await;
    let (foo, bar) = fake_transaction::<_, _, _, VoidError>(&conn, |txn| async move {
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
