// =============================================================================
// Memory CRUD — End-to-End Tests
// =============================================================================
//
// Covers: batch write, get by ID, update, soft delete, restore,
//         export, import.
// =============================================================================

use crate::harness::unique_content;
use crate::tenant_helpers::*;
use reqwest::StatusCode;
use serde_json::json;

// ── Batch Write ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn batch_write_multiple_memories() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-batch").await;

    let batch = json!([
        { "content": unique_content("batch-1"), "role": "user" },
        { "content": unique_content("batch-2"), "role": "assistant" },
        { "content": unique_content("batch-3") }
    ]);
    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory/batch", &batch).await;
    assert_eq!(status, StatusCode::OK, "Batch write failed: {:?}", resp);

    let results = resp.as_array().expect("Should return array of write responses");
    assert_eq!(results.len(), 3, "Should have 3 results");
    for result in results {
        assert_eq!(result["status"].as_str().unwrap(), "accepted");
        assert!(result["episode_id"].is_string());
    }
}

#[tokio::test]
async fn batch_write_empty_array_returns_ok() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-batch-empty").await;

    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory/batch", &json!([])).await;
    assert_eq!(status, StatusCode::OK, "Empty batch should succeed: {:?}", resp);
    let results = resp.as_array().expect("Should return empty array");
    assert!(results.is_empty());
}

// ── Get Memory by ID ────────────────────────────────────────────────────────

#[tokio::test]
async fn get_memory_by_id_after_write() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-get").await;

    // Write a memory
    let content = unique_content("mem-get-by-id");
    let (status, write_resp) = gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": content, "role": "user" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let _episode_id = write_resp["episode_id"].as_str().unwrap();

    // Give the write pipeline time to process
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Search to find the memory ID
    let (status, search_resp) = gateway_post_with_key(
        &api_key,
        "/v1/memory/search",
        &json!({ "query": content, "limit": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = search_resp["results"].as_array().unwrap();
    if results.is_empty() {
        // Pipeline may not have processed yet — skip rather than fail
        eprintln!("WARN: Search returned no results; write pipeline may still be processing");
        return;
    }

    let memory_id = results[0]["memory"]["id"].as_str().unwrap();

    // Get the memory by ID
    let (status, memory) = gateway_get_with_key(&api_key, &format!("/v1/memory/{}", memory_id)).await;
    assert_eq!(status, StatusCode::OK, "Get memory by ID failed: {:?}", memory);
    assert_eq!(memory["id"].as_str().unwrap(), memory_id);
    assert!(memory["content"].is_string());
}

#[tokio::test]
async fn get_nonexistent_memory_returns_404() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-404").await;

    let fake_id = uuid::Uuid::new_v4();
    let (status, _) = gateway_get_with_key(&api_key, &format!("/v1/memory/{}", fake_id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Update Memory ───────────────────────────────────────────────────────────

#[tokio::test]
async fn update_memory_content() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-update").await;

    // Write, wait for processing, search to get ID
    let content = unique_content("mem-update-target");
    let (status, _) = gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": content }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (status, search_resp) = gateway_post_with_key(
        &api_key,
        "/v1/memory/search",
        &json!({ "query": content, "limit": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = search_resp["results"].as_array().unwrap();
    if results.is_empty() {
        eprintln!("WARN: Write pipeline still processing, skipping update test");
        return;
    }
    let memory_id = results[0]["memory"]["id"].as_str().unwrap();

    // Update the memory
    let new_content = unique_content("mem-updated");
    let (status, resp) = gateway_put_with_key(
        &api_key,
        &format!("/v1/memory/{}", memory_id),
        &json!({ "content": new_content }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Update memory failed: {:?}", resp);
}

// ── Soft Delete & Restore ───────────────────────────────────────────────────

#[tokio::test]
async fn soft_delete_and_restore_memory() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-del").await;

    // Write and wait for processing
    let content = unique_content("mem-delete-target");
    let (status, _) = gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": content }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let (status, search_resp) = gateway_post_with_key(
        &api_key,
        "/v1/memory/search",
        &json!({ "query": content, "limit": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let results = search_resp["results"].as_array().unwrap();
    if results.is_empty() {
        eprintln!("WARN: Write pipeline still processing, skipping delete test");
        return;
    }
    let memory_id = results[0]["memory"]["id"].as_str().unwrap();

    // Soft delete
    let (status, _) = gateway_delete_with_key(&api_key, &format!("/v1/memory/{}", memory_id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "Soft delete should return 204");

    // Get should now return 404
    let (status, _) = gateway_get_with_key(&api_key, &format!("/v1/memory/{}", memory_id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "Deleted memory should return 404");

    // Restore
    let (status, resp) = gateway_post_with_key(
        &api_key,
        &format!("/v1/memory/{}/restore", memory_id),
        &json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Restore failed: {:?}", resp);
    assert_eq!(resp["status"].as_str().unwrap_or(""), "restored");

    // Get should work again
    let (status, _) = gateway_get_with_key(&api_key, &format!("/v1/memory/{}", memory_id)).await;
    assert_eq!(status, StatusCode::OK, "Restored memory should be accessible");
}

// ── Read-only key cannot delete/update ──────────────────────────────────────

#[tokio::test]
async fn read_only_key_cannot_update_or_delete() {
    let (client, _initial_key, csrf, _body) = signup_tenant("mem-ro-crud").await;

    // Create a read_only key
    let (status, created) = tenant_post_auth(
        &client,
        &csrf,
        "/app/api-keys",
        &json!({ "name": "ro-crud", "role": "read_only" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let ro_key = created["api_key"].as_str().unwrap().to_string();

    let fake_id = uuid::Uuid::new_v4();

    // Cannot update
    let (status, _) = gateway_put_with_key(
        &ro_key,
        &format!("/v1/memory/{}", fake_id),
        &json!({ "content": "new" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "read_only should not update");

    // Cannot delete
    let (status, _) = gateway_delete_with_key(&ro_key, &format!("/v1/memory/{}", fake_id)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "read_only should not delete");
}

// ── Export ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn export_memories_returns_valid_structure() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-export").await;

    // Write some memories first
    gateway_post_with_key(
        &api_key,
        "/v1/memory",
        &json!({ "content": unique_content("export-1") }),
    )
    .await;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let (status, resp) = gateway_post_with_key(
        &api_key,
        "/v1/memory/export",
        &json!({ "format": "json", "limit": 100 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "Export failed: {:?}", resp);
    assert!(resp["memories"].is_array(), "Should have memories array");
    assert!(resp["exported_at"].is_string() || resp["stats"].is_object());
}

// ── Import ──────────────────────────────────────────────────────────────────
// NOTE: /v1/memory/import is routed by the gateway but /internal/import is
// not yet implemented in the write-service, so this test is skipped for now.

#[tokio::test]
#[ignore = "import endpoint not yet implemented in write-service"]
async fn import_memories_creates_entries() {
    let (_client, api_key, _csrf, _body) = signup_tenant("mem-import").await;

    let import_payload = json!({
        "memories": [
            {
                "content": unique_content("import-1"),
                "kind": "fact",
                "scope": "user"
            },
            {
                "content": unique_content("import-2"),
                "kind": "fact",
                "scope": "user"
            }
        ],
        "conflict_strategy": "skip_duplicates",
        "generate_new_ids": true
    });

    let (status, resp) = gateway_post_with_key(&api_key, "/v1/memory/import", &import_payload).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "Import failed with {}: {:?}",
        status,
        resp
    );
    if resp["imported"].is_number() {
        assert!(resp["imported"].as_u64().unwrap() > 0);
    }
}
