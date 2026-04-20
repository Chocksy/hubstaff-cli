use crate::client::HubstaffClient;
use crate::error::CliError;
use crate::output::CompactOutput;
use std::collections::HashMap;

pub fn list(
    client: &mut HubstaffClient,
    org_id: u64,
    start: &str,
    stop: Option<&str>,
    json: bool,
    page_start: Option<u64>,
    page_limit: Option<u64>,
) -> Result<(), CliError> {
    let start_ts = normalize_timestamp(start);
    let stop_ts = stop.map_or_else(|| chrono::Utc::now().to_rfc3339(), normalize_timestamp);

    let mut params = HashMap::new();
    params.insert("time_slot[start]".to_string(), start_ts);
    params.insert("time_slot[stop]".to_string(), stop_ts);
    if let Some(ps) = page_start {
        params.insert("page_start_id".to_string(), ps.to_string());
    }
    if let Some(pl) = page_limit {
        params.insert("page_limit".to_string(), pl.to_string());
    }

    let data = client.get(&format!("/organizations/{org_id}/activities"), &params)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let out = CompactOutput::table(
        &data,
        "activities",
        &[
            ("DATE", "time_slot"),
            ("USER_ID", "user_id"),
            ("PROJECT_ID", "project_id"),
            ("TRACKED", "tracked"),
            ("KEYBOARD", "keyboard"),
            ("MOUSE", "mouse"),
        ],
        "activities",
        &format!("org:{org_id}"),
    );
    println!("{out}");
    Ok(())
}

/// Convert a bare date like "2026-03-26" to "2026-03-26T00:00:00Z"
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
    fn list_normalizes_date_inputs_to_midnight_utc() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/organizations/5/activities")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded(
                    "time_slot[start]".into(),
                    "2026-03-01T00:00:00Z".into(),
                ),
                mockito::Matcher::UrlEncoded(
                    "time_slot[stop]".into(),
                    "2026-03-02T00:00:00Z".into(),
                ),
            ]))
            .with_status(200)
            .with_body(r#"{"activities":[]}"#)
            .create();

        let mut client = test_client(&server.url());
        list(
            &mut client,
            5,
            "2026-03-01",
            Some("2026-03-02"),
            true,
            None,
            None,
        )
        .expect("list should succeed");

        mock.assert();
    }
}
