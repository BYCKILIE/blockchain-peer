use common::config::{db_config::DbConfig, AppConfig};
use tokio_postgres::NoTls;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("../../resources");
}

async fn run_migrations(db: DbConfig) {
    println!("Running DB migrations...");
    let connection_config = format!(
        "host={} port={} user={} password={} dbname={}",
        db.host, db.port, db.user, db.password, db.db_name
    );

    let (mut client, con) = tokio_postgres::connect(&connection_config, NoTls)
        .await
        .expect("Failed to connect");

    tokio::spawn(async move {
        if let Err(e) = con.await {
            eprintln!("connection error: {}", e);
        }
    });

    let migration_report = embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .expect("Failed to run migration");

    for migration in migration_report.applied_migrations() {
        println!(
            "Migration Applied -  Name: {}, Version: {}",
            migration.name(),
            migration.version()
        );
    }

    println!("DB migrations finished!");
}

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let settings = AppConfig::new("./resources")
        .expect("Failed to load settings");
    run_migrations(settings.database).await;
}
