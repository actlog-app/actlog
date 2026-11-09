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
         DELETE FROM rules WHERE id NOT IN (
             SELECT MIN(id) FROM rules GROUP BY LOWER(pattern), match_field
         );
        ",
    )?;

    seed_default_data(conn)?;
    Ok(())
}

fn seed_default_data(conn: &Connection) -> Result<()> {
    let defaults = [
        ("UNTAGGED", "#D0D0D0", 0),
        ("work", "#60FF60", 2),
        ("games", "#FF6060", -2),
        ("surfing", "#FFFF60", 0),
        ("entertainment", "#FF60FF", -1),
        ("communication", "#60FFFF", 1),
        ("organization", "#6060FF", 1),
        ("project 1", "#FF0000", 2),
        ("project 2", "#00FF00", 2),
        ("project 3", "#0000FF", 2),
        ("project 4", "#60B0B0", 2),
        ("project 5", "#B060B0", 2),
        ("project 6", "#B0B060", 2),
        ("project 7", "#606060", 2),
        ("project 8", "#B0B0B0", 2),
    ];

    for (name, color, score) in defaults {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM tags WHERE LOWER(name) = LOWER(?1))",
                [name],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if !exists {
            let _ = conn.execute(
                "INSERT INTO tags (name, color, productivity_score) VALUES (?1, ?2, ?3)",
                (name, color, score),
            );
        } else {
            let _ = conn.execute(
                "UPDATE tags SET color = ?1 WHERE LOWER(name) = LOWER(?2)",
                (color, name),
            );
        }
    }

    // Migrate any legacy/non-canonical tags to canonical counterparts
    let _ = conn.execute(
        "UPDATE activities SET tag_id = (SELECT id FROM tags WHERE name = 'work')
         WHERE tag_id IN (SELECT id FROM tags WHERE name = 'Development')",
        [],
    );
    let _ = conn.execute(
        "UPDATE activities SET tag_id = (SELECT id FROM tags WHERE name = 'games')
         WHERE tag_id IN (SELECT id FROM tags WHERE name = 'Gaming')",
        [],
    );
    let _ = conn.execute(
        "UPDATE activities SET tag_id = (SELECT id FROM tags WHERE name = 'entertainment')
         WHERE tag_id IN (SELECT id FROM tags WHERE name = 'Media')",
        [],
    );
    // Delete obsolete non-canonical tags
    let _ = conn.execute(
        "DELETE FROM tags WHERE name NOT IN (
            'UNTAGGED', 'work', 'games', 'surfing', 'entertainment',
            'communication', 'organization', 'project 1', 'project 2',
            'project 3', 'project 4', 'project 5', 'project 6',
            'project 7', 'project 8'
        )",
        [],
    );

    let default_rules = [
        ("code.exe", "app_name", "work", 10),
        ("devenv.exe", "app_name", "work", 10),
        ("cargo.exe", "app_name", "work", 10),
        ("slack.exe", "app_name", "communication", 5),
        ("discord.exe", "app_name", "communication", 5),
        ("whatsapp", "app_name", "communication", 10),
        ("steam.exe", "app_name", "games", 10),
        ("roblox", "app_name", "games", 10),
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
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM rules WHERE LOWER(pattern) = LOWER(?1) AND match_field = ?2)",
                    (pattern, match_field),
                    |r| r.get(0),
                )
                .unwrap_or(false);

            if !exists {
                conn.execute(
                    "INSERT INTO rules (pattern, match_field, tag_id, priority) VALUES (?1, ?2, ?3, ?4)",
                    (pattern, match_field, tid, priority),
                )?;
            }
        }
    }

    let _ = conn.execute(
        "UPDATE activities SET tag_id = (SELECT id FROM tags WHERE name = 'games') 
         WHERE tag_id IS NULL AND LOWER(app_name) LIKE '%roblox%'",
        [],
    );
    let _ = conn.execute(
        "UPDATE activities SET tag_id = (SELECT id FROM tags WHERE name = 'communication') 
         WHERE tag_id IS NULL AND LOWER(app_name) LIKE '%whatsapp%'",
        [],
    );

    Ok(())
}



