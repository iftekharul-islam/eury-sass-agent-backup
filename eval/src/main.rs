use agent_store::actor::spawn_actor;
use agent_types::requests::{RunMode, RunRequest};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Eury Eval Harness");

    // 1. Setup isolated eval database
    let db_path = "eval_store.db";
    let _ = fs::remove_file(db_path).await; // Clean start

    let store_handle = spawn_actor(db_path)?;
    println!("Store initialized.");

    // 2. Initialize Stub Provider (via CerseiAdapter)
    // TODO: Create a specialized stub provider configuration for deterministic evals
    // For now, we'd initialize the normal stack with mocked endpoints
    // let adapter = CerseiAdapter::new(...);

    // 3. Define Eval Suite
    // (This is a skeleton for future eval implementations)
    let eval_tasks = ["Create a simple react component", "Explain this code snippet"];

    for (i, task) in eval_tasks.iter().enumerate() {
        println!("Running Eval Task {}: {}", i + 1, task);

        let _request = RunRequest {
            run_id: uuid::Uuid::new_v4(),
            conversation_id: uuid::Uuid::new_v4(),
            prompt: task.to_string(),
            history: vec![],
            attachments: vec![],
            workspace_id: Some("eval_ws".to_string()),
            mode: RunMode::Agent,
            model: agent_types::requests::ModelConfig {
                provider: "stub".to_string(),
                model: "stub".to_string(),
                max_tokens: Some(1000),
                temperature: 0.0,
            },
            workspace_root: Some("/tmp/eval_ws".into()),
            plan_context: None,
        };

        // Simulate storing run start
        store_handle
            .execute(
                "INSERT INTO runs (id, conversation_id, status, mode) VALUES (?, ?, ?, ?)"
                    .to_string(),
                vec![
                    rusqlite::types::Value::Text(format!("run_{i}")),
                    rusqlite::types::Value::Text("eval_conv".to_string()),
                    rusqlite::types::Value::Text("running".to_string()),
                    rusqlite::types::Value::Text("Agent".to_string()),
                ],
            )
            .await?;

        // let mut stream = adapter.run_stream(request, uuid::Uuid::new_v4(), None).await?;
        // while let Some(event) = stream.next().await {
        //     // Log event
        //     // Store event to DB for later verification
        // }
    }

    println!("Eval complete. Shutting down.");
    store_handle.shutdown().await;

    Ok(())
}
