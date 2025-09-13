use crate::http::info_http::{AppState, InfoHttp};
use crate::service::read_service::ReadService;
use common::config::webserver_config::WebServerConfig;
use common::logger::Logger;
use std::sync::Arc;

pub async fn start_server(webserver_config: WebServerConfig, read_service: ReadService) {
    let host = if webserver_config.host.to_lowercase() == "localhost" {
        format!("127.0.0.1:{}", webserver_config.port)
    } else {
        format!("{}:{}", webserver_config.host, webserver_config.port)
    };

    let state = Arc::new(AppState {
        read_service: Arc::new(read_service),
    });

    let app = InfoHttp::new(state);

    Logger::console("webserver", &format!("Listening on http://{}", &host));

    let res = axum_server::bind(host.parse().unwrap())
        .serve(app.into_make_service())
        .await;
    if let Err(err) = res {
        panic!("{}", err);
    }
}
