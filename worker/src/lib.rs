use apalis::prelude::*;
use shipwright_config::Config;
use shipwright_db::{Database, connect_pool, create_database_if_not_exists};
use shipwright_mailer::{EmailClient, EmailPayload};
use tokio::task::JoinHandle;

mod jobs;

pub use apalis::prelude::Storage;
pub use apalis_sql::sqlite::SqliteStorage as WorkerStorage;

pub struct WorkerInitializer {
    pub email_storage: WorkerStorage<EmailPayload>,
    pub monitor_task: JoinHandle<Result<(), std::io::Error>>,
}

impl WorkerInitializer {
    pub async fn init(config: &Config, email_client: EmailClient) -> Result<Self, Error> {
        create_database_if_not_exists(Database::Jobs, config).await?;

        let pool = connect_pool(Database::Jobs, config).await?;

        WorkerStorage::setup(&pool)
            .await
            .expect("unable to run migrations for sqlite worker storage");

        let email_storage: WorkerStorage<EmailPayload> = WorkerStorage::new(pool.clone());

        let email_storage_cloned = email_storage.clone();
        let monitor_task = tokio::task::spawn(async move {
            Monitor::new()
                .register({
                    WorkerBuilder::new("email-worker")
                        .concurrency(2)
                        .data(email_client)
                        .enable_tracing()
                        .backend(email_storage_cloned)
                        .build_fn(jobs::send_email::job)
                })
                .run()
                .await
                .unwrap();
            Ok::<(), std::io::Error>(())
        });
        Ok(Self {
            email_storage,
            monitor_task,
        })
    }
}

/// Errors that can occur as a result of a data layer operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occured while interacting with worker storage.
    ///
    /// Return `500 Internal Server Error` on a worker storage error.
    #[error("error interacting with worker storage")]
    WorkerStorage(#[from] apalis_sql::SqlError),
    /// An error occured while interacting with the database.
    ///
    /// Return `500 Internal Server Error` on a database error.
    #[error("error setting up database for worker")]
    DbSetup(#[from] shipwright_db::Error),
}
