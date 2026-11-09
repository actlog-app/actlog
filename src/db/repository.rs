use crate::model::{ActivitySession, AppTotal, CategoryTotal, DaySummary, MatchField, Rule, Tag};
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct Repository;

impl Repository {
    pub fn save_or_merge_session(conn: &Connection, session: &ActivitySession) -> Result<i64> {
        // Look up the most recent activity record
        let last_record: Option<(i64, i64, i64, String, String, Option<String>, Option<i64>, bool)> = conn
            .query_row(
                "SELECT id, start_time, end_time, app_name, window_title, url, tag_id, is_afk 
                 FROM activities 
                 ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .optional()?;

        if let Some((last_id, _last_start, last_end, last_app, last_title, last_url, last_tag, last_afk)) =
            last_record
        {
            // If same app, title, url, afk status, and within 10s gap, extend existing row
            let time_gap = session.start_time - last_end;
            let same_context = last_app == session.app_name
                && last_title == session.window_title
                && last_url == session.url
                && last_tag == session.tag_id
                && last_afk == session.is_afk;

            if same_context && time_gap >= 0 && time_gap <= 10 {
                let new_end = session.end_time.max(last_end);
                conn.execute(
                    "UPDATE activities SET end_time = ?1, duration = (?1 - start_time) WHERE id = ?2",
                    params![new_end, last_id],
                )?;
                return Ok(last_id);
            }
        }

        // Otherwise insert new activity span
        conn.execute(
            "INSERT INTO activities (start_time, end_time, duration, app_name, exe_path, window_title, url, domain, tag_id, is_afk)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session.start_time,
                session.end_time,
                session.duration,
                session.app_name,
                session.exe_path,
                session.window_title,
                session.url,
                session.domain,
                session.tag_id,
                session.is_afk as i32
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_day_activities(
        conn: &Connection,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<ActivitySession>> {
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, duration, app_name, exe_path, window_title, url, domain, tag_id, is_afk
             FROM activities
             WHERE start_time >= ?1 AND start_time < ?2 AND LOWER(app_name) != 'actlog.exe'
             ORDER BY start_time ASC",
        )?;

        let rows = stmt.query_map(params![start_ts, end_ts], |row| {
            let is_afk_int: i32 = row.get(10)?;
            Ok(ActivitySession {
                id: Some(row.get(0)?),
                start_time: row.get(1)?,
                end_time: row.get(2)?,
                duration: row.get(3)?,
                app_name: row.get(4)?,
                exe_path: row.get(5)?,
                window_title: row.get(6)?,
                url: row.get(7)?,
                domain: row.get(8)?,
                tag_id: row.get(9)?,
                is_afk: is_afk_int != 0,
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    pub fn get_day_summary(
        conn: &Connection,
        date_str: &str,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<DaySummary> {
        let total_active_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration), 0) FROM activities WHERE start_time >= ?1 AND start_time < ?2 AND is_afk = 0 AND LOWER(app_name) != 'actlog.exe'",
                params![start_ts, end_ts],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_afk_seconds: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration), 0) FROM activities WHERE start_time >= ?1 AND start_time < ?2 AND is_afk != 0 AND LOWER(app_name) != 'actlog.exe'",
                params![start_ts, end_ts],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Top apps
        let mut app_stmt = conn.prepare(
            "SELECT app_name, SUM(duration) as total
             FROM activities
             WHERE start_time >= ?1 AND start_time < ?2 AND is_afk = 0 AND LOWER(app_name) != 'actlog.exe'
             GROUP BY app_name
             ORDER BY total DESC
             LIMIT 10",
        )?;
        let top_apps = app_stmt
            .query_map(params![start_ts, end_ts], |r| {
                Ok(AppTotal {
                    app_name: r.get(0)?,
                    total_seconds: r.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Categories / Tags
        let mut cat_stmt = conn.prepare(
            "SELECT COALESCE(t.name, 'UNTAGGED'), COALESCE(t.color, '#64748B'), SUM(a.duration) as total
             FROM activities a
             LEFT JOIN tags t ON a.tag_id = t.id
             WHERE a.start_time >= ?1 AND a.start_time < ?2 AND a.is_afk = 0 AND LOWER(a.app_name) != 'actlog.exe'
             GROUP BY a.tag_id
             ORDER BY total DESC",
        )?;
        let categories = cat_stmt
            .query_map(params![start_ts, end_ts], |r| {
                Ok(CategoryTotal {
                    tag_name: r.get(0)?,
                    color: r.get(1)?,
                    total_seconds: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DaySummary {
            date: date_str.to_string(),
            total_active_seconds,
            total_afk_seconds,
            top_apps,
            categories,
        })
    }

    pub fn get_all_rules(conn: &Connection) -> Result<Vec<Rule>> {
        let mut stmt = conn.prepare(
            "SELECT id, pattern, match_field, tag_id, priority FROM rules ORDER BY priority DESC, id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            let field_str: String = r.get(2)?;
            let field = MatchField::from_str(&field_str).unwrap_or(MatchField::AppName);
            Ok(Rule {
                id: Some(r.get(0)?),
                pattern: r.get(1)?,
                match_field: field,
                tag_id: r.get(3)?,
                priority: r.get(4)?,
            })
        })?;

        let mut rules = Vec::new();
        for r in rows {
            rules.push(r?);
        }
        Ok(rules)
    }

    pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, color, productivity_score FROM tags 
             ORDER BY CASE name 
                 WHEN 'UNTAGGED' THEN 0 
                 WHEN 'work' THEN 1 
                 WHEN 'games' THEN 2 
                 WHEN 'surfing' THEN 3 
                 WHEN 'entertainment' THEN 4 
                 WHEN 'communication' THEN 5 
                 WHEN 'organization' THEN 6 
                 WHEN 'project 1' THEN 7 
                 WHEN 'project 2' THEN 8 
                 WHEN 'project 3' THEN 9 
                 WHEN 'project 4' THEN 10 
                 WHEN 'project 5' THEN 11 
                 WHEN 'project 6' THEN 12 
                 WHEN 'project 7' THEN 13 
                 WHEN 'project 8' THEN 14 
                 ELSE 99 END, id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Tag {
                id: Some(r.get(0)?),
                name: r.get(1)?,
                color: r.get(2)?,
                productivity_score: r.get(3)?,
            })
        })?;

        let mut tags = Vec::new();
        for t in rows {
            tags.push(t?);
        }
        Ok(tags)
    }
}

