# arXiv API Plugin

A [hyper-mcp](https://github.com/hyper-mcp-rs/hyper-mcp) WebAssembly plugin that
exposes the [arXiv API](https://info.arxiv.org/help/api/user-manual.html) as a
single MCP tool with structured input and output.

## Tool

### `query`

Searches arXiv.org for e-prints via the arXiv query interface and returns
structured metadata for each matching article.

**Input Schema** (all parameters are optional, but at least one of
`search_query` or `id_list` must be provided):

| Parameter      | Type   | Description |
| -------------- | ------ | ----------- |
| `search_query` | string | An arXiv search query, e.g. `all:electron`, `ti:"quantum criticality"`, or a boolean expression like `au:del_maestro AND ti:checkerboard`. See the [query construction appendix](https://info.arxiv.org/help/api/user-manual.html#query_details). |
| `id_list`      | string | A comma-delimited list of arXiv ids, e.g. `2301.00001,hep-ex/0307015`. |
| `start`        | int    | The 0-based index of the first returned result (default `0`). |
| `max_results`  | int    | The maximum number of results to return (default `10`, max `2000`). |
| `sortBy`       | enum   | One of `relevance`, `lastUpdatedDate`, `submittedDate`. |
| `sortOrder`    | enum   | One of `ascending`, `descending`. |

If both `search_query` and `id_list` are provided, the results are the articles
in `id_list` that also match `search_query` (i.e. `id_list` acts as a filter).

**Output:** The arXiv API returns an Atom 1.0 XML feed, which the plugin
deserializes into a structured response:

```jsonc
{
  "total_results": 182239,   // total matches for the query (not just this page)
  "start_index": 0,          // 0-based index of the first returned result
  "items_per_page": 2,       // number of results in this page
  "entries": [
    {
      "id": "cond-mat/0011267v1",       // parsed from the <id> tag's …/abs/ URL
      "title": "...",
      "summary": "...",                  // the abstract
      "published": "2000-11-15T16:19:15Z",
      "updated": "2000-11-15T16:19:15Z",
      "authors": [{ "name": "...", "affiliation": "..." }],
      "primary_category": "cond-mat.supr-con",
      "categories": ["cond-mat.supr-con", "cond-mat.str-el"],
      "comment": "...",                  // optional
      "journal_ref": "...",              // optional
      "doi": "...",                      // optional
      "abstract_url": "https://arxiv.org/abs/cond-mat/0011267v1",
      "pdf_url": "https://arxiv.org/pdf/cond-mat/0011267v1",
      "source_url": "https://arxiv.org/e-print/cond-mat/0011267v1",  // optional
      "related_links": {                 // optional, keyed by link title
        "pdf": "https://arxiv.org/pdf/cond-mat/0011267v1",
        "doi": "https://doi.org/10.1234/example"
      }
    }
  ]
}
```

Notes on the parsing:

- **`id`** is parsed from the entry's `<id>` tag by stripping the
  `http(s)://arxiv.org/abs/` prefix. arXiv ids may be new-style (`2301.00001`)
  or old-style containing a slash (`hep-ex/0307015`), so the prefix is stripped
  rather than splitting on the last `/`.
- **`primary_category`** / **`categories`** come from the `term` attribute of
  the `<arxiv:primary_category>` and `<category>` elements.
- **`pdf_url`** is taken from the entry's `<link title="pdf">` element.
- **`source_url`** is the article's e-print (source) bundle at
  `https://arxiv.org/e-print/<id>`. It is included **only** when a `HEAD`
  request to that URL returns a 2xx status, so it is omitted for articles
  without a published source bundle.
- **`related_links`** is a map (title → url) of every entry `<link>` that
  carries both a `rel` and a `title` attribute, e.g. `pdf` and `doi`. The
  abstract page link has no title and is exposed separately as `abstract_url`.
  The `pdf` link is intentionally duplicated here even though it is also
  available discretely as `pdf_url`.

Every URL field (`abstract_url`, `pdf_url`, `source_url`, and the `related_links`
values) is parsed and validated as a well-formed URL before being returned; any
malformed URL is omitted rather than passed through. The values are still
serialized as JSON strings, and the output schema marks them with
`"format": "uri"`.

arXiv reports query errors (such as malformed ids) as an Atom feed containing a
single error entry; the plugin detects these and returns them as a tool error.

## Configuration

The plugin needs network access to the arXiv API host and the arXiv site (for
e-print `HEAD` checks):

```json
{
  "plugins": {
    "arxiv": {
      "url": "oci://your-registry/arxiv-plugin:latest",
      "runtime_config": {
        "allowed_hosts": ["export.arxiv.org", "arxiv.org"]
      }
    }
  }
}
```

For local development, point the plugin at the built WASM file:

```json
{
  "plugins": {
    "arxiv": {
      "url": "file:///path/to/target/wasm32-wasip1/release/plugin.wasm",
      "runtime_config": {
        "allowed_hosts": ["export.arxiv.org", "arxiv.org"]
      }
    }
  }
}
```

## Development

### Building

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
# Output: target/wasm32-wasip1/release/plugin.wasm
```

### Testing

Tests run against the native host target and exercise the XML deserialization
and transformation logic using captured fixtures in `tests/fixtures/`:

```sh
cargo test
```

### Code Quality

```sh
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
```

## License

Apache 2.0. See [LICENSE](LICENSE).
