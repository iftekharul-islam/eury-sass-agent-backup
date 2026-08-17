use agent_sandbox::workspace::{TrustState, Workspace};
use agent_tools::Tool;
use agent_tools::fs::read::ReadFileTool;
use agent_tools::fs::write::WriteFileTool;
use serde_json::json;
use tokio::fs;
use uuid::Uuid;

#[tokio::test]
async fn test_write_and_read() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("agent_test_{}", Uuid::now_v7()));
    fs::create_dir_all(&temp_dir).await?;

    let workspace = Workspace {
        id: Uuid::now_v7(),
        root_path: temp_dir.clone(),
        trust_state: TrustState::Trusted,
    };

    let write_tool = WriteFileTool;
    let res = write_tool
        .execute(
            json!({
                "path": "test.txt",
                "content": "hello world"
            }),
            Some(&workspace),
        )
        .await;

    assert!(res.is_ok(), "Write should succeed");

    let read_tool = ReadFileTool;
    let res = read_tool
        .execute(
            json!({
                "path": "test.txt"
            }),
            Some(&workspace),
        )
        .await;

    assert!(res.is_ok(), "Read should succeed");
    let val = res.map_err(|e| format!("{e:?}"))?;
    assert_eq!(val["content"], "hello world");

    // Cleanup
    fs::remove_dir_all(&temp_dir).await?;
    Ok(())
}

#[tokio::test]
async fn test_path_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("agent_test_{}", Uuid::now_v7()));
    fs::create_dir_all(&temp_dir).await?;

    let workspace = Workspace {
        id: Uuid::now_v7(),
        root_path: temp_dir.clone(),
        trust_state: TrustState::Trusted,
    };

    let write_tool = WriteFileTool;
    let res = write_tool
        .execute(
            json!({
                "path": "../outside.txt",
                "content": "hacked"
            }),
            Some(&workspace),
        )
        .await;

    assert!(res.is_err(), "Write outside workspace should fail");

    // Cleanup
    fs::remove_dir_all(&temp_dir).await?;
    Ok(())
}

#[tokio::test]
async fn read_file_honors_offset_and_limit() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("agent_read_paging_{}", Uuid::now_v7()));
    fs::create_dir_all(&temp_dir).await?;

    let workspace = Workspace {
        id: Uuid::now_v7(),
        root_path: temp_dir.clone(),
        trust_state: TrustState::Trusted,
    };

    let body = (1..=10).map(|n| format!("line{n}")).collect::<Vec<_>>().join("\n");
    WriteFileTool
        .execute(json!({ "path": "paged.txt", "content": body }), Some(&workspace))
        .await?;

    // A window in the middle of the file reports its position and that more
    // lines remain — previously offset/limit were parsed and discarded, so the
    // whole file came back with `truncated` hardcoded false.
    let res = ReadFileTool
        .execute(json!({ "path": "paged.txt", "offset": 3, "limit": 4 }), Some(&workspace))
        .await?;

    assert_eq!(res["content"], "line3\nline4\nline5\nline6");
    assert_eq!(res["startLine"], 3);
    assert_eq!(res["endLine"], 6);
    assert_eq!(res["totalLines"], 10);
    assert_eq!(res["truncated"], true);

    // A window reaching the end is not truncated.
    let tail = ReadFileTool
        .execute(json!({ "path": "paged.txt", "offset": 9, "limit": 50 }), Some(&workspace))
        .await?;
    assert_eq!(tail["content"], "line9\nline10");
    assert_eq!(tail["truncated"], false);

    let _ = fs::remove_dir_all(&temp_dir).await;
    Ok(())
}
