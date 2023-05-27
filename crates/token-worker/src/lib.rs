pub mod aws;
pub mod token_worker;
pub mod worker;

//pub type PgManager =
// deadpool_r2d2::Manager<PostgresConnectionManager<NoTls>>;

pub type PgPool =
    diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;
