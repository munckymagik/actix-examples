use std::env;

use actix::prelude::{Actor, SyncContext};
use actix_web::{error, Error};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

pub struct DbExecutor(pub PgPool);

impl DbExecutor {
    pub fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, Error> {
        self.0.get().map_err(|_| {
            error::ErrorInternalServerError("Failed to get connection from the pool")
        })
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
