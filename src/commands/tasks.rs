use crate::client::HubstaffClient;
use crate::error::CliError;
use crate::output::CompactOutput;
use std::collections::HashMap;

pub fn list(
    client: &mut HubstaffClient,
    project_id: u64,
    json: bool,
    page_start: Option<u64>,
    page_limit: Option<u64>,
) -> Result<(), CliError> {
    let mut params = HashMap::new();
    if let Some(ps) = page_start {
        params.insert("page_start_id".to_string(), ps.to_string());
    }
    if let Some(pl) = page_limit {
        params.insert("page_limit".to_string(), pl.to_string());
    }

    let data = client.get(&format!("/projects/{project_id}/tasks"), &params)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let out = CompactOutput::table(
        &data,
        "tasks",
        &[
            ("ID", "id"),
            ("SUMMARY", "summary"),
            ("STATUS", "status"),
            ("ASSIGNEE", "assignee_id"),
        ],
        "tasks",
        &format!("project:{project_id}"),
    );
    println!("{out}");
    Ok(())
}

pub fn show(client: &mut HubstaffClient, task_id: u64, json: bool) -> Result<(), CliError> {
    let data = client.get(&format!("/tasks/{task_id}"), &HashMap::new())?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    if let Some(task) = data.get("task") {
        let out = CompactOutput::details(
            task,
            &[
                ("ID", "id"),
                ("Summary", "summary"),
                ("Status", "status"),
                ("Assignee", "assignee_id"),
                ("Project", "project_id"),
                ("Created", "created_at"),
            ],
        );
        print!("{out}");
    }
    Ok(())
}

pub fn create(
    client: &mut HubstaffClient,
    project_id: u64,
    summary: &str,
    assignee_id: Option<u64>,
    json: bool,
) -> Result<(), CliError> {
    let mut body = serde_json::json!({ "summary": summary });
    if let Some(aid) = assignee_id {
        body["assignee_id"] = serde_json::json!(aid);
    }

    let data = client.post(&format!("/projects/{project_id}/tasks"), &body)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    if let Some(task) = data.get("task") {
        let out = CompactOutput::one_liner(
            "created",
            &[
                ("task", format!("{}", task["id"])),
                (
                    "summary",
                    task["summary"].as_str().unwrap_or("-").to_string(),
                ),
                ("project", project_id.to_string()),
            ],
        );
        println!("{out}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, Config};

    fn test_client(server_url: &str) -> HubstaffClient {
        let config = Config {
            api_url: server_url.to_string(),
            auth: AuthConfig {
                access_token: Some("test_token".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        HubstaffClient::new(config).expect("client should be constructed")
    }

    #[test]
    fn list_includes_pagination_params() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/projects/9/tasks")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page_start_id".into(), "11".into()),
                mockito::Matcher::UrlEncoded("page_limit".into(), "25".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"tasks":[]}"#)
            .create();

        let mut client = test_client(&server.url());
        list(&mut client, 9, true, Some(11), Some(25)).expect("list should succeed");

        mock.assert();
    }

    #[test]
    fn create_uses_project_id_in_path() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/projects/77/tasks")
            .match_body(mockito::Matcher::Regex(
                r#""summary"\s*:\s*"Write tests""#.into(),
            ))
            .with_status(201)
            .with_body(r#"{"task":{"id":1,"summary":"Write tests"}}"#)
            .create();

        let mut client = test_client(&server.url());
        create(&mut client, 77, "Write tests", None, true).expect("create should succeed");

        mock.assert();
    }
}
