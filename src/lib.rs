mod pdk;
mod types;

use anyhow::{Result, anyhow};
use extism_pdk::{HttpRequest, http};
use pdk::http::http_request_with_retry;
use pdk::types::*;
use schemars::schema_for;
use serde_json::Value;
use types::*;
use url::Url;

const ARXIV_API_URL: &str = "https://export.arxiv.org/api/query";

pub(crate) fn list_tools(_input: ListToolsRequest) -> Result<ListToolsResult> {
    Ok(ListToolsResult {
        tools: vec![Tool {
            name: "query".to_string(),
            title: Some("Query arXiv".to_string()),
            annotations: Some(ToolAnnotations {
                read_only_hint: Some(true),
                open_world_hint: Some(true),
                ..Default::default()
            }),
            description: Some(
                r#"Search arXiv.org for e-prints via the arXiv API. Provide a `search_query` (e.g. `all:electron`, `ti:"quantum criticality"`, or a boolean expression like `au:del_maestro AND ti:checkerboard`) and/or an `id_list` of comma-delimited arXiv ids. If both are given, results are the articles in `id_list` that also match `search_query`. Use `start` and `max_results` to page through results, and `sortBy`/`sortOrder` to control ordering."#
                    .to_string(),
            ),
            input_schema: schema_for!(QueryArguments),
            output_schema: Some(schema_for!(QueryResponse)),
        }],
    })
}

pub(crate) fn call_tool(input: CallToolRequest) -> Result<CallToolResult> {
    match input.request.name.as_str() {
        "query" => Ok(query(input)),
        other => Ok(CallToolResult::error(format!("Unknown tool: {other}"))),
    }
}

fn query(input: CallToolRequest) -> CallToolResult {
    let args: QueryArguments =
        match serde_json::from_value(Value::Object(input.request.arguments.unwrap_or_default())) {
            Ok(args) => args,
            Err(e) => return CallToolResult::error(format!("Invalid arguments: {e}")),
        };

    let search_query = args
        .search_query
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let id_list = args
        .id_list
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if search_query.is_none() && id_list.is_none() {
        return CallToolResult::error(
            "At least one of `search_query` or `id_list` must be provided.".to_string(),
        );
    }

    let mut url = match Url::parse(ARXIV_API_URL) {
        Ok(url) => url,
        Err(e) => return CallToolResult::error(e.to_string()),
    };
    {
        let mut pairs = url.query_pairs_mut();
        if let Some(q) = search_query {
            pairs.append_pair("search_query", q);
        }
        if let Some(ids) = id_list {
            pairs.append_pair("id_list", ids);
        }
        if let Some(start) = args.start {
            pairs.append_pair("start", &start.to_string());
        }
        if let Some(max_results) = args.max_results {
            pairs.append_pair("max_results", &max_results.to_string());
        }
        if let Some(sort_by) = args.sort_by {
            pairs.append_pair("sortBy", sort_by.as_param());
        }
        if let Some(sort_order) = args.sort_order {
            pairs.append_pair("sortOrder", sort_order.as_param());
        }
    }

    let req = HttpRequest::new(url.as_str()).with_method("GET");
    let res = match http_request_with_retry(&req) {
        Ok(res) => res,
        Err(e) => return CallToolResult::error(format!("arXiv API request failed: {e}")),
    };

    let body = String::from_utf8_lossy(&res.body()).to_string();
    if res.status_code() < 200 || res.status_code() >= 300 {
        return CallToolResult::error(format!(
            "arXiv API request failed with status {}: {}",
            res.status_code(),
            body,
        ));
    }

    // arXiv reports errors (e.g. malformed ids) as a feed with a single error
    // entry rather than via an HTTP error status, so the response parses into
    // one of two distinct types.
    let mut response = match ArxivResponse::from_atom(&body) {
        Ok(ArxivResponse::Results(response)) => response,
        Ok(ArxivResponse::Error(error)) => {
            return CallToolResult::error(format!("arXiv API error: {}", error.message));
        }
        Err(e) => return CallToolResult::error(format!("Failed to parse arXiv response: {e}")),
    };

    // Populate the source (e-print) URL for entries whose e-print bundle
    // actually exists, verified with a HEAD request. This issues one request
    // per entry, so it is skippable (e.g. for large or unbounded queries) via
    // `verify_source_url=false`, in which case `source_url` is left unset.
    if args.verify_source_url.unwrap_or(true) {
        for entry in &mut response.entries {
            if entry.id.is_empty() {
                continue;
            }
            if let Some(candidate) = eprint_url(&entry.id)
                && head_ok(candidate.as_str())
            {
                entry.source_url = Some(candidate);
            }
        }
    }

    let structured_content = match serde_json::to_value(&response) {
        Ok(Value::Object(map)) => Some(map),
        _ => {
            return CallToolResult::error(
                "Failed to convert query response to a JSON object.".to_string(),
            );
        }
    };

    let text = structured_content
        .as_ref()
        .and_then(|sc| serde_json::to_string_pretty(sc).ok())
        .unwrap_or_default();

    CallToolResult {
        content: vec![ContentBlock::Text(TextContent {
            text,
            ..Default::default()
        })],
        structured_content,
        ..Default::default()
    }
}

/// Return `true` if a HEAD request to `url` succeeds with a 2xx status.
fn head_ok(url: &str) -> bool {
    let req = HttpRequest::new(url).with_method("HEAD");
    match http::request::<()>(&req, None) {
        Ok(res) => res.status_code() >= 200 && res.status_code() < 300,
        Err(_) => false,
    }
}

// Provide completion suggestions for a partially-typed input.
pub(crate) fn complete(_input: CompleteRequest) -> Result<CompleteResult> {
    Ok(CompleteResult::default())
}

// Retrieve a specific prompt by name.
pub(crate) fn get_prompt(_input: GetPromptRequest) -> Result<GetPromptResult> {
    Err(anyhow!("Prompts are not supported by this plugin"))
}

// List all available prompts.
pub(crate) fn list_prompts(_input: ListPromptsRequest) -> Result<ListPromptsResult> {
    Ok(ListPromptsResult::default())
}

// List all available resource templates.
pub(crate) fn list_resource_templates(
    _input: ListResourceTemplatesRequest,
) -> Result<ListResourceTemplatesResult> {
    Ok(ListResourceTemplatesResult::default())
}

// List all available resources.
pub(crate) fn list_resources(_input: ListResourcesRequest) -> Result<ListResourcesResult> {
    Ok(ListResourcesResult::default())
}

// Notification that the list of roots has changed.
pub(crate) fn on_roots_list_changed(_input: PluginNotificationContext) -> Result<()> {
    Ok(())
}

// Read the contents of a resource by its URI.
pub(crate) fn read_resource(_input: ReadResourceRequest) -> Result<ReadResourceResult> {
    Err(anyhow!("Resources are not supported by this plugin"))
}
