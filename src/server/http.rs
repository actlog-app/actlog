use crate::server::handlers::ApiHandler;
use crate::tracker::engine::SharedTrackerState;
use std::sync::Arc;
use std::thread;
use tiny_http::{Method, Server};

pub struct HttpServer {
    state: Arc<SharedTrackerState>,
    db_path: String,
    port: u16,
}

impl HttpServer {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String, port: u16) -> Self {
        Self {
            state,
            db_path,
            port,
        }
    }

    pub fn start(self) -> thread::JoinHandle<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let server = Server::http(&addr).expect("Failed to bind local HTTP server");
        let handler = ApiHandler::new(self.state, self.db_path);

        thread::spawn(move || {
            for mut request in server.incoming_requests() {
                let url_str = request.url().to_string();
                let mut parts = url_str.splitn(2, '?');
                let path = parts.next().unwrap_or("/");
                let query = parts.next().unwrap_or("");

                let response = match (request.method(), path) {
                    (&Method::Options, _) => handler.handle_options(),
                    (&Method::Post, "/api/heartbeat/browser") => {
                        let mut body = Vec::new();
                        let _ = request.as_reader().read_to_end(&mut body);
                        handler.handle_browser_heartbeat(&body)
                    }
                    (&Method::Get, "/api/activities/current") => handler.handle_current_activity(),
                    (&Method::Get, "/api/stats/summary") | (&Method::Get, "/api/stats/today") => {
                        handler.handle_summary(query)
                    }
                    (&Method::Get, "/") | (&Method::Get, "/dashboard") => {
                        handler.handle_browser_dashboard()
                    }
                    _ => {
                        let mut resp = tiny_http::Response::from_string("{\"error\":\"Not Found\"}")
                            .with_status_code(tiny_http::StatusCode(404));
                        handler.handle_options(); // Just for headers
                        resp.add_header(
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"application/json"[..],
                            )
                            .unwrap(),
                        );
                        resp
                    }
                };

                let _ = request.respond(response);
            }
        })
    }
}
