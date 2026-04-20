use crate::client::HubstaffClient;
use crate::error::CliError;
use crate::output::CompactOutput;

pub fn create(
    client: &mut HubstaffClient,
    project_id: u64,
    start: &str,
    stop: &str,
    json: bool,
) -> Result<(), CliError> {
    let body = serde_json::json!({
        "project_id": project_id,
        "started_at": normalize_timestamp(start),
        "stopped_at": normalize_timestamp(stop)
    });

    let data = client.post("/users/me/time_entries", &body)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    if let Some(entry) = data.get("time_entry") {
        let out = CompactOutput::one_liner(
            "created",
            &[
                ("time_entry", format!("{}", entry["id"])),
                ("project", project_id.to_string()),
                (
                    "start",
                    entry["started_at"].as_str().unwrap_or("-").to_string(),
                ),
                (
                    "stop",
                    entry["stopped_at"].as_str().unwrap_or("-").to_string(),
                ),
            ],
        );
        println!("{out}");
    }
    Ok(())
}

fn normalize_timestamp(input: &str) -> String {
    if input.contains('T') {
        input.to_string()
    } else {
        format!("{input}T00:00:00Z")
    }
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
    fn create_includes_project_and_normalized_timestamps_in_body() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/users/me/time_entries")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::Regex(r#""project_id"\s*:\s*123"#.into()),
                mockito::Matcher::Regex(
                    r#""started_at"\s*:\s*"2026-03-10T00:00:00Z""#.into(),
                ),
                mockito::Matcher::Regex(
                    r#""stopped_at"\s*:\s*"2026-03-10T01:00:00Z""#.into(),
                ),
            ]))
            .with_status(201)
            .with_body(
                r#"{"time_entry":{"id":1,"started_at":"2026-03-10T00:00:00Z","stopped_at":"2026-03-10T01:00:00Z"}}"#,
            )
            .create();

        let mut client = test_client(&server.url());
        create(&mut client, 123, "2026-03-10", "2026-03-10T01:00:00Z", true)
            .expect("create should succeed");

        mock.assert();
    }
}
