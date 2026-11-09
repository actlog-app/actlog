use actlog::model::ActivitySession;
use actlog::ui::common::format_pt_time;
use actlog::ui::views::statistics::ActivityNode;
use std::collections::HashMap;

#[test]
fn test_format_pt_time_exact_parity() {
    assert_eq!(format_pt_time(0), "0:00:00");
    assert_eq!(format_pt_time(5), "0:00:05");
    assert_eq!(format_pt_time(15), "0:00:15");
    assert_eq!(format_pt_time(75), "0:01:15");
    assert_eq!(format_pt_time(460), "0:07:40");
    assert_eq!(format_pt_time(1090), "0:18:10");
    assert_eq!(format_pt_time(7200), "2:00:00");
    assert_eq!(format_pt_time(25660), "7:07:40");
    assert_eq!(format_pt_time(317120), "88:05:20");
    assert_eq!(format_pt_time(1446240), "401:44:00");
    assert_eq!(format_pt_time(1917495), "532:38:15");
    assert_eq!(format_pt_time(5185965), "1440:32:45");
}

#[test]
fn test_single_child_collapse_and_lowercasing() {
    let activities = vec![
        ActivitySession {
            id: Some(1),
            start_time: 1000,
            end_time: 1015,
            duration: 15,
            app_name: "PowerToys.PowerLauncher.exe".to_string(),
            exe_path: None,
            window_title: "PowerToys.PowerLauncher".to_string(),
            url: None,
            domain: None,
            tag_id: None,
            is_afk: false,
        },
        ActivitySession {
            id: Some(2),
            start_time: 1020,
            end_time: 1070,
            duration: 50,
            app_name: "Code.exe".to_string(),
            exe_path: None,
            window_title: "file1.rs".to_string(),
            url: None,
            domain: None,
            tag_id: Some(8),
            is_afk: false,
        },
        ActivitySession {
            id: Some(3),
            start_time: 1070,
            end_time: 1120,
            duration: 50,
            app_name: "Code.exe".to_string(),
            exe_path: None,
            window_title: "file2.rs".to_string(),
            url: None,
            domain: None,
            tag_id: Some(8),
            is_afk: false,
        },
    ];

    let overrides = HashMap::new();
    let tree = ActivityNode::build_tree(&activities, 1, 0, "", None, &overrides);

    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].name, "(root)");
    assert_eq!(tree[0].total_seconds, 115);
    assert_eq!(tree[0].children.len(), 2);

    // Top app should be code (100s) with 2 children, so it does not collapse
    let top = &tree[0].children[0];
    assert_eq!(top.name, "code");
    assert_eq!(top.total_seconds, 100);

    // Second app has only 1 child, so it collapses into: powertoys.powerlauncher - PowerToys.PowerLauncher
    let second = &tree[0].children[1];
    assert_eq!(
        second.name,
        "powertoys.powerlauncher - PowerToys.PowerLauncher"
    );
    assert_eq!(second.total_seconds, 15);
    assert!(second.children.is_empty());
}
