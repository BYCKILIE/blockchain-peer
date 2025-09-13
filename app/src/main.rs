use common::config::AppConfig;
use futures::future::join_all;
use tokio::runtime::Builder;
use tokio::task::JoinHandle;

fn main() {
    common::intro::print_info("./resources");

    let config = match AppConfig::new("./resources") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    };

    let thread_pool_size = 0
        + config.peer.thread_pool
        + config.nats.thread_pool
        + config.webserver.thread_pool
        + config.database.thread_pool;

    let runtime = Builder::new_multi_thread()
        .worker_threads(thread_pool_size as usize)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let peer_handle = peer::power_module_peer(config.peer).await.unwrap();
        let consumer_handle = consumer::power_module_consumer(config.nats).await.unwrap();
        let db_handle = db::power_module_db(config.webserver, config.database)
            .await
            .unwrap();

        let mut threads: Vec<JoinHandle<()>> = Vec::new();
        threads.extend(peer_handle);
        threads.extend(consumer_handle);
        threads.extend(db_handle);

        join_all(threads).await;
    });
}
