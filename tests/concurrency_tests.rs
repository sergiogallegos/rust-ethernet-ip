use rust_ethernet_ip::tag_manager::{TagManager, TagMetadata, TagPermissions, TagScope};
use rust_ethernet_ip::tag_path::TagPath;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[test]
fn tag_path_parse_is_thread_safe() {
    let inputs = vec![
        "Tag1",
        "Program:Main.Tag2",
        "MyArray[10]",
        "MyString.DATA[5]",
        "Nested.Struct.Member",
        "Program:Main.Array[2].Field",
    ];

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let inputs = inputs.clone();
            thread::spawn(move || {
                for _ in 0..200 {
                    for input in &inputs {
                        let parsed = TagPath::parse(input);
                        assert!(parsed.is_ok(), "failed to parse '{input}'");
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

#[tokio::test]
async fn tag_manager_cache_is_concurrent_safe() {
    let manager = Arc::new(TagManager::new());
    let mut tasks = Vec::new();

    for i in 0..50 {
        let manager = Arc::clone(&manager);
        tasks.push(tokio::spawn(async move {
            let metadata = TagMetadata {
                data_type: 0x00C4,
                size: 4,
                is_array: false,
                dimensions: vec![],
                permissions: TagPermissions {
                    readable: true,
                    writable: true,
                },
                scope: TagScope::Controller,
                last_access: Instant::now(),
                array_info: None,
                last_updated: Instant::now(),
            };

            let tag_name = format!("Tag{i}");
            manager
                .update_metadata(tag_name.clone(), metadata)
                .await
                .expect("metadata update should succeed");
            let cached = manager
                .get_metadata(&tag_name)
                .await
                .expect("metadata read should succeed");
            assert!(cached.is_some());
        }));
    }

    for task in tasks {
        task.await.expect("task failed");
    }
}
