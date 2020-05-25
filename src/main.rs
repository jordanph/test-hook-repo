use std::net::SocketAddr;
use warp::Filter;

#[macro_use]
extern crate vec1;

use handlers::check_run::handle_check_run_request;
use handlers::check_suite::handle_check_suite_request;
use handlers::pod_finished_successfully::handle_pod_finished_successfully_request;
use handlers::update_check_run::handle_update_check_run_request;
use handlers::{
    pipeline::handle_get_pipeline, pipelines::handle_get_pipelines, steps::handle_get_steps,
};
use routes::{
    check_run_route, check_suite_route, get_pipeline_route, get_pipeline_steps_route,
    get_pipelines_route, notify_pod_successfully_completed_route, update_check_run_route,
};

use pod_informer::poll_pods;

mod github;
mod handlers;
mod pipeline;
mod routes;
mod pod_informer;

#[tokio::main]
async fn main() {
    let _ = pretty_env_logger::try_init();

    let cors = warp::cors().allow_origin("http://localhost:3000");

    let check_suite_handler = check_suite_route().and_then(handle_check_suite_request);

    let check_run_handler = check_run_route().and_then(handle_check_run_request);

    let update_check_run_handler =
        update_check_run_route().and_then(handle_update_check_run_request);

    let pod_finished_successfully_handler = notify_pod_successfully_completed_route()
        .and_then(handle_pod_finished_successfully_request);

    let get_pipelines_handler = get_pipelines_route()
        .and_then(handle_get_pipelines)
        .with(cors);

    let pipeline_cors = warp::cors().allow_origin("http://localhost:3000");

    let get_pipeline_handler = get_pipeline_route()
        .and_then(handle_get_pipeline)
        .with(pipeline_cors);

    let steps_cors = warp::cors().allow_origin("http://localhost:3000");

    let get_pipeline_steps_handler = get_pipeline_steps_route()
        .and_then(handle_get_steps)
        .with(steps_cors);

    let app_routes = check_suite_handler
        .or(check_run_handler)
        .or(update_check_run_handler)
        .or(pod_finished_successfully_handler)
        .or(get_pipeline_steps_handler)
        .or(get_pipeline_handler)
        .or(get_pipelines_handler);

    let address = std::env::var("SOCKET_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".to_string());

    let socket_address: SocketAddr = format!("{}:{}", address, port)
        .parse()
        .expect("Unable to parse socket address");

    let tasks = vec![
        tokio::spawn(async move {  warp::serve(app_routes).run(socket_address).await }),
        tokio::spawn(async move { poll_pods().await }),
    ];
    
    futures::future::join_all(tasks).await;

    // warp::serve(app_routes).run(socket_address).await
}
