use rusqlite::{Connection, Result};

pub fn init_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS tags (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             name TEXT UNIQUE NOT NULL,
             color TEXT NOT NULL,
             productivity_score INTEGER DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS rules (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             pattern TEXT NOT NULL,
             match_field TEXT NOT NULL,
             tag_id INTEGER NOT NULL,
             priority INTEGER DEFAULT 0,
             FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS activities (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             start_time INTEGER NOT NULL,
             end_time INTEGER NOT NULL,
             duration INTEGER NOT NULL,
             app_name TEXT NOT NULL,
             exe_path TEXT,
             window_title TEXT NOT NULL,
             url TEXT,
             domain TEXT,
             tag_id INTEGER,
             is_afk INTEGER DEFAULT 0,
             FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE SET NULL
         );

         CREATE INDEX IF NOT EXISTS idx_activities_time ON activities(start_time, end_time);
         CREATE INDEX IF NOT EXISTS idx_activities_app ON activities(app_name);
         CREATE INDEX IF NOT EXISTS idx_activities_domain ON activities(domain);
         DELETE FROM activities WHERE LOWER(app_name) = 'actlog.exe';
        ",
    )?;

    seed_default_data(conn)?;
    Ok(())
}

fn seed_default_data(conn: &Connection) -> Result<()> {
    // Only seed if tags table is empty
    let tag_count: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))?;
    if tag_count > 0 {
        return Ok(());
    }

    let defaults = [
        ("UNTAGGED", "#D0D0D0", 0),
        ("work", "#60FF60", 2),
        ("games", "#6060FF", -2),
        ("surfing", "#60FFFF", 0),
        ("entertainment", "#FF60FF", -1),
        ("communication", "#FFFF60", 1),
        ("organization", "#FF6060", 1),
        ("project 1", "#0000FF", 2),
        ("project 2", "#00FF00", 2),
        ("project 3", "#FF0000", 2),
        ("project 4", "#B0B060", 2),
        ("project 5", "#B060B0", 2),
        ("project 6", "#60B0B0", 2),
        ("project 7", "#606060", 2),
        ("project 8", "#B0B0B0", 2),
    ];

    for (name, color, score) in defaults {
        conn.execute(
            "INSERT INTO tags (name, color, productivity_score) VALUES (?1, ?2, ?3)",
            (name, color, score),
        )?;
    }

    let default_rules = [
        ("code.exe", "app_name", "work", 10),
        ("devenv.exe", "app_name", "work", 10),
        ("cargo.exe", "app_name", "work", 10),
        ("slack.exe", "app_name", "communication", 5),
        ("discord.exe", "app_name", "communication", 5),
        ("steam.exe", "app_name", "games", 10),
        ("youtube.com", "domain", "entertainment", 8),
        ("twitch.tv", "domain", "entertainment", 8),
        ("reddit.com", "domain", "surfing", 8),
    ];

    for (pattern, match_field, tag_name, priority) in default_rules {
        let tag_id: Result<i64> = conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            [tag_name],
            |r| r.get(0),
        );
        if let Ok(tid) = tag_id {
            conn.execute(
                "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, ?2, ?3, ?4)",
                (pattern, match_field, tid, priority),
            )?;
        }
    }

    Ok(())
}

