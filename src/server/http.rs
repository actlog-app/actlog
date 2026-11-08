use crate::server::handlers::ApiHandler;
use crate::tracker::SharedTrackerState;
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

    pub fn start(self) -> Option<thread::JoinHandle<()>> {
        let mut server = None;
        let mut active_port = self.port;

        for p in self.port..(self.port + 10) {
            let addr = format!("127.0.0.1:{}", p);
            match Server::http(&addr) {
                Ok(s) => {
                    active_port = p;
                    server = Some(s);
                    break;
                }
                Err(e) => {
                    eprintln!("Port {} unavailable: {}. Trying next...", p, e);
                }
            }
        }

        let server = match server {
            Some(s) => s,
            None => {
                eprintln!("Warning: Could not bind any HTTP server port in range {}..{}. External integrations disabled.", self.port, self.port + 10);
                return None;
            }
        };

        self.state.bound_port.store(active_port, std::sync::atomic::Ordering::Relaxed);
        let handler = ApiHandler::new(self.state, self.db_path);

        Some(thread::spawn(move || {
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
        }))
    }
}
