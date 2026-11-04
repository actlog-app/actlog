use crate::db::models::BrowserHeartbeat;
use crate::db::repository::Repository;
use crate::tracker::engine::SharedTrackerState;
use chrono::{Local, NaiveDate, TimeZone};
use rusqlite::Connection;
use std::sync::Arc;
use tiny_http::{Header, Response, StatusCode};

pub struct ApiHandler {
    state: Arc<SharedTrackerState>,
    db_path: String,
}

impl ApiHandler {
    pub fn new(state: Arc<SharedTrackerState>, db_path: String) -> Self {
        Self { state, db_path }
    }

    pub fn handle_options(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let mut resp = Response::from_string("");
        self.add_cors_headers(&mut resp);
        resp
    }

    pub fn handle_browser_heartbeat(&self, body: &[u8]) -> Response<std::io::Cursor<Vec<u8>>> {
        let heartbeat: Result<BrowserHeartbeat, _> = serde_json::from_slice(body);

        match heartbeat {
            Ok(hb) => {
                {
                    let mut lock = self.state.current_browser_tab.lock().unwrap();
                    *lock = Some(hb);
                }
                let mut resp = Response::from_string("{\"status\":\"ok\"}");
                resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                self.add_cors_headers(&mut resp);
                resp
            }
            Err(e) => {
                let err_msg = format!("{{\"error\":\"{}\"}}", e);
                let mut resp = Response::from_string(err_msg).with_status_code(StatusCode(400));
                resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                self.add_cors_headers(&mut resp);
                resp
            }
        }
    }

    pub fn handle_current_activity(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let current = {
            let lock = self.state.current_activity.lock().unwrap();
            lock.clone()
        };

        let json = serde_json::to_string(&current).unwrap_or_else(|_| "{}".to_string());
        let mut resp = Response::from_string(json);
        resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
        self.add_cors_headers(&mut resp);
        resp
    }

    pub fn handle_summary(&self, query: &str) -> Response<std::io::Cursor<Vec<u8>>> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let date_str = extract_query_param(query, "date").unwrap_or(today);

        let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| Local::now().date_naive());

        let start_of_day = Local
            .from_local_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .timestamp();
        let end_of_day = start_of_day + 86400;

        let conn = match Connection::open(&self.db_path) {
            Ok(c) => c,
            Err(e) => {
                let mut resp = Response::from_string(format!("{{\"error\":\"{}\"}}", e))
                    .with_status_code(StatusCode(500));
                self.add_cors_headers(&mut resp);
                return resp;
            }
        };

        match Repository::get_day_summary(&conn, &date_str, start_of_day, end_of_day) {
            Ok(summary) => {
                let json = serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string());
                let mut resp = Response::from_string(json);
                resp.add_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                );
                self.add_cors_headers(&mut resp);
                resp
            }
            Err(e) => {
                let mut resp = Response::from_string(format!("{{\"error\":\"{}\"}}", e))
                    .with_status_code(StatusCode(500));
                self.add_cors_headers(&mut resp);
                resp
            }
        }
    }

    pub fn handle_browser_dashboard(&self) -> Response<std::io::Cursor<Vec<u8>>> {
        let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>ActLog - Browser Dashboard</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #121212; color: #E0E0E0; margin: 0; padding: 24px; }
    h1 { color: #FFFFFF; font-size: 22px; margin-bottom: 8px; }
    .card { background: #1E1E1E; border-radius: 8px; padding: 16px; margin-bottom: 16px; border: 1px solid #2C2C2C; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; }
    .stat-val { font-size: 28px; font-weight: bold; color: #4CAF50; }
    .stat-lbl { font-size: 13px; color: #888; text-transform: uppercase; margin-top: 4px; }
    table { width: 100%; border-collapse: collapse; margin-top: 12px; }
    th, td { text-align: left; padding: 8px 12px; border-bottom: 1px solid #2C2C2C; }
    th { color: #888; font-weight: 500; font-size: 13px; }
    .badge { padding: 3px 8px; border-radius: 12px; font-size: 12px; font-weight: 500; }
  </style>
</head>
<body>
  <h1>ActLog Activity Monitor</h1>
  <div id="live" class="card" style="border-left: 4px solid #2196F3;">
    <div style="font-size: 14px; color: #888;">CURRENT ACTIVE WINDOW</div>
    <div id="active-title" style="font-size: 18px; font-weight: 600; margin-top: 4px;">Loading...</div>
    <div id="active-app" style="font-size: 13px; color: #2196F3; margin-top: 2px;"></div>
  </div>

  <div class="grid">
    <div class="card">
      <div id="stat-active" class="stat-val">0h 0m</div>
      <div class="stat-lbl">Active Time Today</div>
    </div>
    <div class="card">
      <div id="stat-afk" class="stat-val" style="color: #FF9800;">0h 0m</div>
      <div class="stat-lbl">Idle / Away Time</div>
    </div>
  </div>

  <div class="card">
    <h3 style="margin-top:0;">Top Applications & Sites</h3>
    <table id="top-apps">
      <thead><tr><th>Application / Site</th><th>Duration</th></tr></thead>
      <tbody><tr><td colspan="2" style="color:#666;">No activity recorded yet</td></tr></tbody>
    </table>
  </div>

  <script>
    function fmtSec(sec) {
      const h = Math.floor(sec / 3600);
      const m = Math.floor((sec % 3600) / 60);
      return `${h}h ${m}m`;
    }

    async function refresh() {
      try {
        const curRes = await fetch('/api/activities/current');
        const cur = await curRes.json();
        if (cur) {
          document.getElementById('active-title').innerText = cur.window_title || 'Desktop';
          document.getElementById('active-app').innerText = `${cur.app_name} (${fmtSec(cur.duration)})`;
        }

        const sumRes = await fetch('/api/stats/summary');
        const sum = await sumRes.json();
        if (sum) {
          document.getElementById('stat-active').innerText = fmtSec(sum.total_active_seconds);
          document.getElementById('stat-afk').innerText = fmtSec(sum.total_afk_seconds);

          const tbody = document.querySelector('#top-apps tbody');
          if (sum.top_apps && sum.top_apps.length > 0) {
            tbody.innerHTML = sum.top_apps.map(a => `<tr><td>${a.app_name}</td><td>${fmtSec(a.total_seconds)}</td></tr>`).join('');
          }
        }
      } catch (err) {
        console.error(err);
      }
    }
    refresh();
    setInterval(refresh, 2000);
  </script>
</body>
</html>"#;

        let mut resp = Response::from_string(html);
        resp.add_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap());
        self.add_cors_headers(&mut resp);
        resp
    }

    fn add_cors_headers(&self, resp: &mut Response<std::io::Cursor<Vec<u8>>>) {
        resp.add_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
        resp.add_header(
            Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..])
                .unwrap(),
        );
        resp.add_header(
            Header::from_bytes(
                &b"Access-Control-Allow-Headers"[..],
                &b"Content-Type, Authorization"[..],
            )
            .unwrap(),
        );
    }
}

fn extract_query_param(query: &str, key: &str) -> Option<String> {
    for part in query.split('&') {
        let mut kv = part.split('=');
        if let Some(k) = kv.next() {
            if k == key {
                return kv.next().map(|s| s.to_string());
            }
        }
    }
    None
}
