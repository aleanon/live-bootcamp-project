I had a problem with random tests failing when running against the docker container, but the tests only failed when using nextest not cargo test. The wierd thing is it worked for a while, then suddenly started failing.

I tried removing and recreating the container, but this did not fix the issue.

The fix for me was using the testcontainers-modules crate, this lets you spin up containers in tests, and it will automatically clean up after the test is done. so you get a fresh container at every run. This can either be set up with a new container in each test or storing a container in a static for reuse by several tests in that run.
the static approach is a tiny bit more code, but the tests run faster.

so in case someone else needs/want to do it this way


This is how I did it for storing the container in a static
```rust
    use testcontainers_modules::{
        postgres,
        testcontainers::{ContainerAsync, runners::AsyncRunner},
    };
    use tokio::sync::{OnceCell, RwLock, RwLockReadGuard};

    static POSTGRES_CONTAINER: OnceCell<RwLock<ContainerAsync<postgres::Postgres>>> =
        OnceCell::const_new();

    async fn container() -> RwLockReadGuard<'static, ContainerAsync<postgres::Postgres>> {
        POSTGRES_CONTAINER
            .get_or_init(async || {
                let container = postgres::Postgres::default()
                    .start()
                    .await
                    .expect("Failed to start db container");
                RwLock::new(container)
            })
            .await
            .read()
            .await
    }

    async fn connect_test_db() -> PgPool {
        let container = container().await;

        let db_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get the mapped port of the container");

        let host = container
            .get_host()
            .await
            .expect("Failed to get the container host address");

        let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

        configure_postgresql(db_url).await // Use the old function for creating a new db with a unique name, but with a passed url instead of the constant
    }

```

Here is how it can be done with a new container pr test
```rust
    async fn setup_and_connect_db_container() -> (ContainerAsync<postgres::Postgres>, PgPool) {
        let container = postgres::Postgres::default()
            .start()
            .await
            .expect("Failed to start container");

        let db_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get the mapped port of the container");

        let host = container
            .get_host()
            .await
            .expect("Failed to get the container host address");

        let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

        let connection = get_postgres_pool(&db_url)
            .await
            .expect("Failed to connect to database");

        sqlx::migrate!()
            .run(&connection)
            .await
            .expect("Failed to migrate the database");

        (container, connection)
    }

    //In the test
    async fn some_test() {
        let (_container, pool) = setup_and_connect_db_container().await;

        //... Run the rest of the test as normal.
        // Note: the container must be kept alive until the test is complete, when the container is dropped it's shut down and cleaned up in docker
    }

```
