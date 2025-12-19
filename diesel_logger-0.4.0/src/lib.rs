extern crate diesel;
#[macro_use]
extern crate log;

use std::fmt::Display;
use std::time::{Duration, Instant};

use diesel::backend::Backend;
use diesel::connection::{
    Connection, ConnectionSealed, LoadConnection, MultiConnectionHelper, SimpleConnection,
    TransactionManager, TransactionManagerStatus,
};
use diesel::debug_query;
use diesel::expression::QueryMetadata;
use diesel::migration::MigrationConnection;
use diesel::prelude::*;
use diesel::query_builder::{AsQuery, Query, QueryFragment, QueryId};
use diesel::r2d2::R2D2Connection;

/// Wraps a diesel `Connection` to time and log each query using
/// the configured logger for the `log` crate.
///
/// Currently, this produces a `debug` log on every query,
/// an `info` on queries that take longer than 1 second,
/// and a `warn`ing on queries that take longer than 5 seconds.
/// These thresholds will be configurable in a future version.
pub struct LoggingConnection<C: Connection> {
    connection: C,
    transaction_manager: LoggingTransactionManager,
}

impl<C> LoggingConnection<C>
where
    C: Connection,
{
    fn bench_query<F, R>(query: &dyn QueryFragment<C::Backend>, func: F) -> R
    where
        F: FnMut() -> R,
        C: 'static,
        <C as Connection>::Backend: std::default::Default,
        <C::Backend as Backend>::QueryBuilder: Default,
    {
        let debug_query = debug_query::<<LoggingConnection<C> as Connection>::Backend, _>(&query);
        Self::bench_query_str(&debug_query, func)
    }

    fn bench_query_str<F, R>(query: &dyn Display, mut func: F) -> R
    where
        F: FnMut() -> R,
    {
        let start_time = Instant::now();
        let result = func();
        let duration = start_time.elapsed();
        log_query(&query, duration);
        result
    }

    fn bench_query_begin() -> Instant {
        Instant::now()
    }

    fn bench_query_end(start_time: Instant, query: &dyn Display) {
        let duration = start_time.elapsed();
        log_query(&query, duration);
    }
}

impl<C> LoggingConnection<C>
where
    C: Connection,
{
    pub fn new(connection: C) -> Self {
        Self {
            connection,
            transaction_manager: Default::default(),
        }
    }
}

impl<C> Connection for LoggingConnection<C>
where
    C: Connection + 'static,
    <C as Connection>::Backend: std::default::Default,
    <C::Backend as Backend>::QueryBuilder: Default,
{
    type Backend = C::Backend;
    type TransactionManager = LoggingTransactionManager;

    fn establish(database_url: &str) -> ConnectionResult<Self> {
        Ok(LoggingConnection::new(C::establish(database_url)?))
    }

    fn begin_test_transaction(&mut self) -> QueryResult<()> {
        self.connection.begin_test_transaction()
    }

    fn execute_returning_count<T>(&mut self, source: &T) -> QueryResult<usize>
    where
        Self: Sized,
        T: QueryFragment<Self::Backend> + QueryId,
    {
        Self::bench_query(source, || self.connection.execute_returning_count(source))
    }

    fn transaction_state(&mut self) -> &mut <Self::TransactionManager as TransactionManager<LoggingConnection<C>>>::TransactionStateData{
        &mut self.transaction_manager
    }

    #[doc = " Get the instrumentation instance stored in this connection"]
    // #[cfg_attr(not(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes"),doc(hidden))]
    // #[cfg_attr(docsrs,doc(cfg(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes")))]
    fn instrumentation(&mut self) -> &mut dyn diesel::connection::Instrumentation {
        return self.connection.instrumentation();
    }

    fn set_instrumentation(&mut self, instrumentation: impl diesel::connection::Instrumentation) {
        self.connection.set_instrumentation(instrumentation);
    }
}

impl<B, C> LoadConnection<B> for LoggingConnection<C>
where
    C: LoadConnection<B> + 'static,
    <C as Connection>::Backend: std::default::Default,
    <C::Backend as Backend>::QueryBuilder: Default,
{
    type Cursor<'conn, 'query> = <C as LoadConnection<B>>::Cursor<'conn, 'query>;
    type Row<'conn, 'query> = <C as LoadConnection<B>>::Row<'conn, 'query>;

    fn load<'conn, 'query, T>(
        &'conn mut self,
        source: T,
    ) -> QueryResult<Self::Cursor<'conn, 'query>>
    where
        T: Query + QueryFragment<Self::Backend> + QueryId + 'query,
        Self::Backend: QueryMetadata<T::SqlType>,
    {
        let query = source.as_query();
        let debug_string =
            debug_query::<<LoggingConnection<C> as Connection>::Backend, _>(&query).to_string();

        let begin = Self::bench_query_begin();
        let res = self.connection.load(query);
        Self::bench_query_end(begin, &debug_string);
        res
    }
}

impl<C> SimpleConnection for LoggingConnection<C>
where
    C: SimpleConnection + Connection + 'static,
    <C::Backend as Backend>::QueryBuilder: Default,
{
    fn batch_execute(&mut self, query: &str) -> QueryResult<()> {
        Self::bench_query_str(&query, || self.connection.batch_execute(query))
    }
}

impl<C> R2D2Connection for LoggingConnection<C>
where
    C: R2D2Connection + Connection + 'static,
    <C::Backend as Backend>::QueryBuilder: Default,
    <C as Connection>::Backend: std::default::Default,
{
    fn ping(&mut self) -> QueryResult<()> {
        self.connection.ping()
    }
}

impl<C: diesel::Connection> ConnectionSealed for LoggingConnection<C> {}

impl<C> MigrationConnection for LoggingConnection<C>
where
    C: 'static + Connection + MigrationConnection,
    <C::Backend as Backend>::QueryBuilder: Default,
    <C as Connection>::Backend: std::default::Default,
{
    fn setup(&mut self) -> QueryResult<usize> {
        self.connection.setup()
    }
}

#[derive(Default)]
pub struct LoggingTransactionManager {}

impl<C> TransactionManager<LoggingConnection<C>> for LoggingTransactionManager
where
    C: 'static + Connection,
    <C::Backend as Backend>::QueryBuilder: Default,
    <C as Connection>::Backend: std::default::Default,
{
    type TransactionStateData = Self;

    fn begin_transaction(conn: &mut LoggingConnection<C>) -> QueryResult<()> {
        <<C as Connection>::TransactionManager as TransactionManager<C>>::begin_transaction(
            &mut conn.connection,
        )
    }

    fn rollback_transaction(conn: &mut LoggingConnection<C>) -> QueryResult<()> {
        <<C as Connection>::TransactionManager as TransactionManager<C>>::rollback_transaction(
            &mut conn.connection,
        )
    }

    fn commit_transaction(conn: &mut LoggingConnection<C>) -> QueryResult<()> {
        <<C as Connection>::TransactionManager as TransactionManager<C>>::commit_transaction(
            &mut conn.connection,
        )
    }

    fn transaction_manager_status_mut(
        conn: &mut LoggingConnection<C>,
    ) -> &mut TransactionManagerStatus {
        <<C as Connection>::TransactionManager as TransactionManager<C>>::transaction_manager_status_mut(&mut conn.connection)
    }
}

impl<C> MultiConnectionHelper for LoggingConnection<C>
where
    C: MultiConnectionHelper<Backend = Self::Backend>,
    Self: Connection,
{
    fn to_any<'a>(
        lookup: &mut <Self::Backend as diesel::sql_types::TypeMetadata>::MetadataLookup,
    ) -> &mut (dyn std::any::Any + 'a) {
        C::to_any(lookup)
    }

    fn from_any(
        lookup: &mut dyn std::any::Any,
    ) -> Option<&mut <Self::Backend as diesel::sql_types::TypeMetadata>::MetadataLookup> {
        C::from_any(lookup)
    }
}

fn log_query(query: &dyn Display, duration: Duration) {
    if duration.as_secs() >= 5 {
        warn!(
            "SLOW QUERY [{:.2} s]: {}",
            duration_to_secs(duration),
            query
        );
    } else if duration.as_secs() >= 1 {
        info!(
            "SLOW QUERY [{:.2} s]: {}",
            duration_to_secs(duration),
            query
        );
    } else {
        debug!("QUERY: [{:.1}ms]: {}", duration_to_ms(duration), query);
    }
}

const NANOS_PER_MILLI: u32 = 1_000_000;
const MILLIS_PER_SEC: u32 = 1_000;

fn duration_to_secs(duration: Duration) -> f32 {
    duration_to_ms(duration) / MILLIS_PER_SEC as f32
}

fn duration_to_ms(duration: Duration) -> f32 {
    (duration.as_secs() as u32 * 1000) as f32
        + (duration.subsec_nanos() as f32 / NANOS_PER_MILLI as f32)
}
