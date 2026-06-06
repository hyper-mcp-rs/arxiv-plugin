//! Request and response types for the arXiv `query` tool.
//!
//! The public [`QueryArguments`] / [`QueryResponse`] types derive [`JsonSchema`]
//! so the plugin can advertise structured input and output schemas. The `Xml*`
//! types are private intermediate representations used to deserialize the
//! Atom 1.0 feed returned by the arXiv API before it is transformed into the
//! clean, namespace-free [`QueryResponse`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

const ABS_PREFIX_HTTP: &str = "http://arxiv.org/abs/";
const ABS_PREFIX_HTTPS: &str = "https://arxiv.org/abs/";

// ---------------------------------------------------------------------------
// Tool arguments (structured input)
// ---------------------------------------------------------------------------

/// How to sort the result set. Mirrors the arXiv `sortBy` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SortBy {
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "lastUpdatedDate")]
    LastUpdatedDate,
    #[serde(rename = "submittedDate")]
    SubmittedDate,
}

impl SortBy {
    pub fn as_param(&self) -> &'static str {
        match self {
            SortBy::Relevance => "relevance",
            SortBy::LastUpdatedDate => "lastUpdatedDate",
            SortBy::SubmittedDate => "submittedDate",
        }
    }
}

/// The direction of the sort. Mirrors the arXiv `sortOrder` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SortOrder {
    #[serde(rename = "ascending")]
    Ascending,
    #[serde(rename = "descending")]
    Descending,
}

impl SortOrder {
    pub fn as_param(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "ascending",
            SortOrder::Descending => "descending",
        }
    }
}

/// Arguments accepted by the `query` tool. These map directly onto the
/// parameters of the arXiv query interface.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryArguments {
    /// The arXiv search query, e.g. `all:electron`, `ti:"quantum criticality"`,
    /// or a boolean expression such as `au:del_maestro AND ti:checkerboard`.
    /// See the arXiv API user manual for the full query-construction syntax.
    #[serde(
        rename = "search_query",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_query: Option<String>,

    /// A comma-delimited list of arXiv ids to fetch, e.g. `2301.00001,hep-ex/0307015`.
    /// If both `search_query` and `id_list` are provided, results are the
    /// articles in `id_list` that also match `search_query`.
    #[serde(rename = "id_list", default, skip_serializing_if = "Option::is_none")]
    pub id_list: Option<String>,

    /// The 0-based index of the first returned result. Defaults to 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,

    /// The maximum number of results to return. Defaults to 10. arXiv limits
    /// this to at most 2000 per call.
    #[serde(
        rename = "max_results",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_results: Option<u32>,

    /// How to sort results: by relevance, last updated date, or submitted date.
    #[serde(rename = "sortBy", default, skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortBy>,

    /// The sort direction: ascending or descending.
    #[serde(rename = "sortOrder", default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
}

// ---------------------------------------------------------------------------
// Tool response (structured output)
// ---------------------------------------------------------------------------

/// A DOI associated with an article, together with its resolved URL when
/// arXiv provides one.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Doi {
    /// The DOI itself, e.g. `10.1103/PhysRevD.61.084004`.
    pub doi: String,

    /// The resolved DOI URL (from the matching `<link title="doi">`), if
    /// present and well-formed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
}

/// An author of an article.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Author {
    /// The author's name.
    pub name: String,

    /// The author's affiliations, if provided. arXiv allows an author to list
    /// more than one affiliation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affiliations: Vec<String>,
}

/// A single article returned by an arXiv query.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    /// The bare arXiv identifier, parsed from the `<id>` tag (the
    /// `http(s)://arxiv.org/abs/` prefix is stripped). May be a new-style id
    /// (`2301.00001v1`) or an old-style id containing a slash
    /// (`hep-ex/0307015v1`).
    pub id: String,

    /// The article title.
    pub title: String,

    /// The article abstract.
    pub summary: String,

    /// The date version 1 of the article was submitted.
    pub published: String,

    /// The date the retrieved version of the article was submitted.
    pub updated: String,

    /// The article authors, in order of authorship.
    pub authors: Vec<Author>,

    /// The primary arXiv subject classification, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_category: Option<String>,

    /// All arXiv / ACM / MSC subject classifications for the article.
    #[serde(default)]
    pub categories: Vec<String>,

    /// The author's comment, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,

    /// The journal reference, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal_ref: Option<String>,

    /// The article DOIs, if present. A single article may resolve to several
    /// DOIs (e.g. an original plus errata), each with its resolved URL.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dois: Vec<Doi>,

    /// URL of the article's abstract page. Present whenever the feed provides
    /// a well-formed URL (always, in practice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abstract_url: Option<Url>,

    /// URL of the article's PDF, if present and well-formed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_url: Option<Url>,

    /// URL of the article's source (e-print) bundle, populated only when the
    /// e-print URL responds with a successful HEAD request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<Url>,
}

/// The full response of a `query` tool call.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QueryResponse {
    /// The total number of results matching the query (not just this page).
    pub total_results: u64,

    /// The 0-based index of the first returned result within the total set.
    pub start_index: u64,

    /// The number of results returned in this page.
    pub items_per_page: u64,

    /// The returned articles.
    pub entries: Vec<Entry>,
}

/// An error reported by the arXiv API.
///
/// arXiv does not signal query errors (e.g. malformed ids) via the HTTP
/// status; instead it returns an Atom feed containing a single error entry.
/// This struct captures that error entry.
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArxivError {
    /// A short error code parsed from the error id fragment, e.g.
    /// `incorrect_id_format_for_1234.12345`, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The human-readable error message (the entry's `<summary>`).
    pub message: String,

    /// A URL to a more detailed explanation of the error.
    pub link: String,

    /// When the error response was generated.
    pub updated: String,
}

/// The outcome of an arXiv query: either a successful result set or an error
/// reported by the API. The two share the same Atom skeleton but are
/// represented as distinct Rust types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArxivResponse {
    Results(QueryResponse),
    Error(ArxivError),
}

// ---------------------------------------------------------------------------
// Internal Atom/XML deserialization types
// ---------------------------------------------------------------------------
//
// quick-xml's serde support strips namespace prefixes and matches on the
// element's local name, and exposes attributes as `@`-prefixed fields. The
// renames below therefore use the local names (e.g. `totalResults`, not
// `opensearch:totalResults`).

#[derive(Debug, Default, Deserialize)]
pub(crate) struct XmlFeed {
    #[serde(rename = "totalResults", default)]
    pub total_results: Option<String>,
    #[serde(rename = "startIndex", default)]
    pub start_index: Option<String>,
    #[serde(rename = "itemsPerPage", default)]
    pub items_per_page: Option<String>,
    #[serde(rename = "entry", default)]
    pub entries: Vec<XmlEntry>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct XmlEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub published: String,
    #[serde(default)]
    pub updated: String,
    #[serde(rename = "author", default)]
    pub authors: Vec<XmlAuthor>,
    #[serde(rename = "link", default)]
    pub links: Vec<XmlLink>,
    #[serde(rename = "category", default)]
    pub categories: Vec<XmlCategory>,
    #[serde(rename = "primary_category", default)]
    pub primary_category: Option<XmlCategory>,
    #[serde(rename = "comment", default)]
    pub comment: Option<String>,
    #[serde(rename = "journal_ref", default)]
    pub journal_ref: Option<String>,
    // A single article may carry multiple `<arxiv:doi>` elements (original plus
    // errata), so this must be a `Vec` (a scalar would fail with "duplicate
    // field `doi`").
    #[serde(rename = "doi", default)]
    pub doi: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct XmlAuthor {
    #[serde(default)]
    pub name: String,
    // An author may carry multiple `<arxiv:affiliation>` elements, so this must
    // be a `Vec` (a scalar would fail with "duplicate field `affiliation`").
    #[serde(rename = "affiliation", default)]
    pub affiliation: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct XmlLink {
    #[serde(rename = "@href", default)]
    pub href: String,
    #[serde(rename = "@rel", default)]
    pub rel: Option<String>,
    #[serde(rename = "@title", default)]
    pub title: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct XmlCategory {
    #[serde(rename = "@term", default)]
    pub term: String,
}

// ---------------------------------------------------------------------------
// Transformation helpers
// ---------------------------------------------------------------------------

/// Collapse all runs of whitespace into single spaces and trim the result.
/// arXiv wraps titles and abstracts across multiple lines with leading
/// indentation, so this produces a clean single-line value.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn trimmed_opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Parse a string into a [`Url`], returning `None` for empty or malformed
/// input. Used to ensure only well-formed URLs are ever surfaced to callers.
fn parse_url_opt(s: &str) -> Option<Url> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Url::parse(trimmed).ok()
    }
}

/// Parse a bare arXiv id out of the `<id>` URL.
///
/// The `<id>` tag holds the abstract URL, e.g.
/// `http://arxiv.org/abs/hep-ex/0307015v1`. We strip the `…/abs/` prefix to
/// recover the identifier. Old-style ids contain a `/` (`hep-ex/0307015`),
/// so we must strip a known prefix rather than splitting on the last `/`.
pub fn parse_arxiv_id(id_url: &str) -> String {
    let trimmed = id_url.trim();
    trimmed
        .strip_prefix(ABS_PREFIX_HTTPS)
        .or_else(|| trimmed.strip_prefix(ABS_PREFIX_HTTP))
        .unwrap_or(trimmed)
        .to_string()
}

/// Build the candidate e-print (source) URL for a bare arXiv id. Returns
/// `None` if the resulting string is not a valid URL.
pub fn eprint_url(id: &str) -> Option<Url> {
    parse_url_opt(&format!("https://arxiv.org/e-print/{id}"))
}

impl Entry {
    /// Convert a deserialized Atom entry into a clean [`Entry`].
    ///
    /// The `source_url` field is left as `None`; callers are expected to
    /// verify the e-print URL (see [`eprint_url`]) and populate it.
    pub(crate) fn from_xml(xml: XmlEntry) -> Entry {
        let id = parse_arxiv_id(&xml.id);

        // Prefer the `alternate` link (the versioned abstract URL); fall back
        // to the raw `<id>` URL. Only a well-formed URL is surfaced.
        let abstract_url = xml
            .links
            .iter()
            .find(|l| l.rel.as_deref() == Some("alternate"))
            .and_then(|l| parse_url_opt(&l.href))
            .or_else(|| parse_url_opt(&xml.id));

        let pdf_url = xml
            .links
            .iter()
            .find(|l| l.title.as_deref() == Some("pdf"))
            .and_then(|l| parse_url_opt(&l.href));

        // Resolved DOI URLs come from `<link title="doi">` elements (one per
        // DOI). A doi link is the resolved DOI, so its path is exactly
        // `/<doi>`. Pair each `<arxiv:doi>` value with the link whose path tail
        // equals it: this is order-independent and avoids matching a DOI that
        // is merely a substring/prefix of another DOI's URL.
        let doi_links: Vec<Url> = xml
            .links
            .iter()
            .filter(|l| l.title.as_deref() == Some("doi"))
            .filter_map(|l| parse_url_opt(&l.href))
            .collect();
        let dois = xml
            .doi
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|doi| Doi {
                doi: doi.to_string(),
                url: doi_links
                    .iter()
                    .find(|u| u.path().trim_start_matches('/') == doi)
                    .cloned(),
            })
            .collect();

        let authors = xml
            .authors
            .into_iter()
            .map(|a| Author {
                name: normalize_whitespace(&a.name),
                affiliations: a
                    .affiliation
                    .iter()
                    .map(|s| normalize_whitespace(s))
                    .filter(|s| !s.is_empty())
                    .collect(),
            })
            .collect();

        let categories = xml
            .categories
            .into_iter()
            .map(|c| c.term.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();

        Entry {
            id,
            title: normalize_whitespace(&xml.title),
            summary: normalize_whitespace(&xml.summary),
            published: xml.published.trim().to_string(),
            updated: xml.updated.trim().to_string(),
            authors,
            primary_category: xml
                .primary_category
                .map(|c| c.term.trim().to_string())
                .filter(|t| !t.is_empty()),
            categories,
            comment: trimmed_opt(xml.comment),
            journal_ref: trimmed_opt(xml.journal_ref),
            dois,
            abstract_url,
            pdf_url,
            source_url: None,
        }
    }
}

/// Deserialize the shared Atom feed skeleton. Both successful and error
/// responses use this same structure; classification happens afterwards based
/// on the entry contents.
fn parse_feed(xml: &str) -> Result<XmlFeed, quick_xml::DeError> {
    quick_xml::de::from_str(xml)
}

fn parse_count(v: &Option<String>) -> u64 {
    v.as_deref()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

impl QueryResponse {
    fn from_feed(feed: XmlFeed) -> QueryResponse {
        QueryResponse {
            total_results: parse_count(&feed.total_results),
            start_index: parse_count(&feed.start_index),
            items_per_page: parse_count(&feed.items_per_page),
            entries: feed.entries.into_iter().map(Entry::from_xml).collect(),
        }
    }
}

impl ArxivError {
    /// Build an [`ArxivError`] from a feed if it represents an arXiv error
    /// response: a single entry whose id points at `arxiv.org/api/errors`.
    fn from_feed(feed: &XmlFeed) -> Option<ArxivError> {
        if feed.entries.len() != 1 {
            return None;
        }
        let entry = &feed.entries[0];
        let id = entry.id.trim();
        if !id.contains("arxiv.org/api/errors") {
            return None;
        }

        // The error id has the form `…/errors#incorrect_id_format_for_1234.12345`;
        // the fragment after `#` is a machine-readable error code.
        let code = id
            .split_once('#')
            .map(|(_, fragment)| fragment.trim().to_string())
            .filter(|c| !c.is_empty());

        Some(ArxivError {
            code,
            message: normalize_whitespace(&entry.summary),
            link: id.to_string(),
            updated: entry.updated.trim().to_string(),
        })
    }
}

impl ArxivResponse {
    /// Parse an arXiv Atom feed, classifying it as either a successful result
    /// set ([`QueryResponse`]) or an [`ArxivError`].
    ///
    /// `source_url` fields on successful results are left unpopulated; the
    /// caller verifies e-print URLs separately.
    pub fn from_atom(xml: &str) -> Result<ArxivResponse, quick_xml::DeError> {
        let feed = parse_feed(xml)?;
        Ok(match ArxivError::from_feed(&feed) {
            Some(error) => ArxivResponse::Error(error),
            None => ArxivResponse::Results(QueryResponse::from_feed(feed)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ELECTRON_FEED: &str = include_str!("../tests/fixtures/electron.xml");
    const ERROR_FEED: &str = include_str!("../tests/fixtures/error.xml");
    const RUST_FEED: &str = include_str!("../tests/fixtures/rust.xml");
    const RUST_LASTUPDATED_FEED: &str = include_str!("../tests/fixtures/rust_lastupdated.xml");
    const MULTI_DOI_FEED: &str = include_str!("../tests/fixtures/multi_doi.xml");

    /// Parse a feed and unwrap the successful results variant.
    fn parse_results(xml: &str) -> QueryResponse {
        match ArxivResponse::from_atom(xml).expect("feed parses") {
            ArxivResponse::Results(response) => response,
            ArxivResponse::Error(error) => panic!("unexpected error response: {error:?}"),
        }
    }

    #[test]
    fn parses_new_and_old_style_ids() {
        assert_eq!(
            parse_arxiv_id("http://arxiv.org/abs/2301.00001v1"),
            "2301.00001v1"
        );
        assert_eq!(
            parse_arxiv_id("https://arxiv.org/abs/2301.00001"),
            "2301.00001"
        );
        // Old-style ids contain a slash and must not be split on the last `/`.
        assert_eq!(
            parse_arxiv_id("http://arxiv.org/abs/hep-ex/0307015v1"),
            "hep-ex/0307015v1"
        );
        assert_eq!(
            parse_arxiv_id("https://arxiv.org/abs/cond-mat/0011267"),
            "cond-mat/0011267"
        );
        // Surrounding whitespace (as in the manual examples) is trimmed.
        assert_eq!(
            parse_arxiv_id("\n    http://arxiv.org/abs/hep-ex/0307015\n"),
            "hep-ex/0307015"
        );
    }

    #[test]
    fn eprint_url_is_built_from_id() {
        assert_eq!(
            eprint_url("hep-ex/0307015v1").as_ref().map(Url::as_str),
            Some("https://arxiv.org/e-print/hep-ex/0307015v1")
        );
    }

    #[test]
    fn parses_feed_metadata() {
        let response = parse_results(ELECTRON_FEED);
        assert_eq!(response.total_results, 182239);
        assert_eq!(response.start_index, 0);
        assert_eq!(response.items_per_page, 2);
        assert_eq!(response.entries.len(), 2);
    }

    #[test]
    fn parses_first_entry() {
        let response = parse_results(ELECTRON_FEED);
        let entry = &response.entries[0];

        assert_eq!(entry.id, "cond-mat/0011267v1");
        assert_eq!(
            entry.title,
            "The electronic structure of cuprates from high energy spectroscopy"
        );
        assert!(entry.summary.starts_with("We report studies"));
        // Whitespace and newlines in the abstract are collapsed.
        assert!(!entry.summary.contains('\n'));
        assert_eq!(entry.published, "2000-11-15T16:19:15Z");
        assert_eq!(entry.updated, "2000-11-15T16:19:15Z");

        assert_eq!(entry.authors.len(), 8);
        assert_eq!(entry.authors[0].name, "Mark S. Golden");
        assert!(entry.authors[0].affiliations.is_empty());

        assert_eq!(entry.primary_category.as_deref(), Some("cond-mat.supr-con"));
        assert_eq!(
            entry.categories,
            vec!["cond-mat.supr-con", "cond-mat.str-el"]
        );
        assert_eq!(
            entry.journal_ref.as_deref(),
            Some("J. Electron Spectr. Relat. Phenom. 117-118, 203 (2001)")
        );
        assert!(entry.comment.as_deref().unwrap().contains("special issue"));
        assert!(entry.dois.is_empty());

        assert_eq!(
            entry.abstract_url.as_ref().map(Url::as_str),
            Some("https://arxiv.org/abs/cond-mat/0011267v1")
        );
        assert_eq!(
            entry.pdf_url.as_ref().map(Url::as_str),
            Some("https://arxiv.org/pdf/cond-mat/0011267v1")
        );
        // source_url is only populated after a live HEAD check.
        assert!(entry.source_url.is_none());
    }

    #[test]
    fn parses_entry_with_noncontiguous_links() {
        // The last entry in this feed has its `alternate`/`pdf` links early and
        // a third `doi` link later, separated by other elements. quick-xml's
        // `overlapped-lists` feature is required to collect such non-contiguous
        // repeated elements into a single `Vec` instead of erroring with
        // "duplicate field `link`".
        let response = parse_results(RUST_FEED);
        assert_eq!(response.entries.len(), 5);

        let last = response.entries.last().unwrap();
        assert_eq!(last.id, "2411.14174v2");
        assert_eq!(
            last.abstract_url.as_ref().map(Url::as_str),
            Some("https://arxiv.org/abs/2411.14174v2")
        );
        assert_eq!(
            last.pdf_url.as_ref().map(Url::as_str),
            Some("https://arxiv.org/pdf/2411.14174v2")
        );

        // The DOI comes from the non-contiguous `<arxiv:doi>` element and is
        // paired with its resolved URL from the matching `<link title="doi">`.
        assert_eq!(last.dois.len(), 1);
        assert_eq!(last.dois[0].doi, "10.14722/ndss.2025.241407");
        assert_eq!(
            last.dois[0].url.as_ref().map(Url::as_str),
            Some("https://doi.org/10.14722/ndss.2025.241407")
        );
    }

    #[test]
    fn parses_entry_with_multiple_dois() {
        // gr-qc/9910091 carries five `<arxiv:doi>` elements; a scalar field
        // would fail with "duplicate field `doi`".
        let response = parse_results(MULTI_DOI_FEED);
        let entry = &response.entries[0];
        let dois: Vec<&str> = entry.dois.iter().map(|d| d.doi.as_str()).collect();
        assert_eq!(
            dois,
            vec![
                "10.1103/PhysRevD.61.084004",
                "10.1103/PhysRevD.63.049902",
                "10.1103/PhysRevD.65.069902",
                "10.1103/PhysRevD.67.089901",
                "10.1103/PhysRevD.78.109902",
            ]
        );
        // Each DOI is paired with its resolved `https://doi.org/<doi>` URL.
        for d in &entry.dois {
            assert_eq!(
                d.url.as_ref().map(Url::as_str),
                Some(format!("https://doi.org/{}", d.doi).as_str())
            );
        }
    }

    #[test]
    fn doi_url_pairing_is_exact_not_substring() {
        // Two DOIs where one is a prefix of the other, with the links listed in
        // an order that would mis-pair under a substring match. Verifies each
        // DOI resolves to its own URL via exact path-tail matching.
        let xml = XmlEntry {
            id: "http://arxiv.org/abs/1234.5678v1".to_string(),
            title: "Prefix DOI entry".to_string(),
            doi: vec!["10.1234/foo".to_string(), "10.1234/foo.s1".to_string()],
            links: vec![
                // The longer (prefix-containing) URL is listed first on purpose.
                XmlLink {
                    href: "https://doi.org/10.1234/foo.s1".to_string(),
                    rel: Some("related".to_string()),
                    title: Some("doi".to_string()),
                },
                XmlLink {
                    href: "https://doi.org/10.1234/foo".to_string(),
                    rel: Some("related".to_string()),
                    title: Some("doi".to_string()),
                },
            ],
            ..Default::default()
        };

        let entry = Entry::from_xml(xml);
        assert_eq!(entry.dois.len(), 2);
        assert_eq!(entry.dois[0].doi, "10.1234/foo");
        assert_eq!(
            entry.dois[0].url.as_ref().map(Url::as_str),
            Some("https://doi.org/10.1234/foo")
        );
        assert_eq!(entry.dois[1].doi, "10.1234/foo.s1");
        assert_eq!(
            entry.dois[1].url.as_ref().map(Url::as_str),
            Some("https://doi.org/10.1234/foo.s1")
        );
    }

    #[test]
    fn parses_author_with_multiple_affiliations() {
        // The first author of the first entry in this feed carries two
        // `<arxiv:affiliation>` elements. A scalar field would fail to
        // deserialize with "duplicate field `affiliation`".
        let response = parse_results(RUST_LASTUPDATED_FEED);
        let first_author = &response.entries[0].authors[0];
        assert_eq!(first_author.name, "Rohit Goswami");
        assert_eq!(
            first_author.affiliations,
            vec![
                "TurtleTech ehf., Reykjavik, Iceland".to_string(),
                "Institute IMX and Lab-COSMO, EPFL, Lausanne, Switzerland".to_string(),
            ]
        );

        // An author with a single affiliation still yields a one-element vec.
        let second_author = &response.entries[0].authors[1];
        assert_eq!(second_author.name, "Ruhila Goswami");
        assert_eq!(second_author.affiliations.len(), 1);
    }

    #[test]
    fn parse_url_opt_rejects_invalid_and_accepts_valid() {
        // Valid absolute URLs (optionally surrounded by whitespace) are kept.
        assert!(parse_url_opt("https://arxiv.org/abs/2301.00001v1").is_some());
        assert_eq!(
            parse_url_opt("  https://arxiv.org/x  ").map(|u| u.to_string()),
            Some("https://arxiv.org/x".to_string())
        );
        // Empty / whitespace-only input is rejected.
        assert!(parse_url_opt("").is_none());
        assert!(parse_url_opt("   ").is_none());
        // Scheme-less / relative strings are not valid URLs and are rejected.
        assert!(parse_url_opt("not a url").is_none());
        assert!(parse_url_opt("/relative/path").is_none());
        assert!(parse_url_opt("10.14722/ndss.2025").is_none());
    }

    #[test]
    fn malformed_urls_are_dropped_from_entry() {
        // A synthetic entry whose links are all malformed. Verifies that
        // invalid URLs are omitted rather than surfaced to the caller, while
        // the DOI string itself is still retained.
        let xml = XmlEntry {
            id: "not a url".to_string(),
            title: "Bad URL entry".to_string(),
            doi: vec!["10.1234/example".to_string()],
            links: vec![
                // alternate (abstract) link with a malformed href
                XmlLink {
                    href: "not a url".to_string(),
                    rel: Some("alternate".to_string()),
                    title: None,
                },
                // pdf link with a malformed href
                XmlLink {
                    href: "/relative/path".to_string(),
                    rel: Some("related".to_string()),
                    title: Some("pdf".to_string()),
                },
                // doi link with a malformed (scheme-less) href
                XmlLink {
                    href: "10.1234/example".to_string(),
                    rel: Some("related".to_string()),
                    title: Some("doi".to_string()),
                },
            ],
            ..Default::default()
        };

        let entry = Entry::from_xml(xml);

        // Both the malformed alternate link and the non-URL `<id>` fallback
        // fail to parse, so no abstract URL is surfaced.
        assert!(entry.abstract_url.is_none());
        // Malformed pdf link is dropped.
        assert!(entry.pdf_url.is_none());
        // The DOI string is kept, but its malformed link URL is dropped to None.
        assert_eq!(entry.dois.len(), 1);
        assert_eq!(entry.dois[0].doi, "10.1234/example");
        assert!(entry.dois[0].url.is_none());
    }

    #[test]
    fn error_feed_parses_into_error_variant() {
        match ArxivResponse::from_atom(ERROR_FEED).expect("error feed parses") {
            ArxivResponse::Error(error) => {
                assert_eq!(error.message, "incorrect id format for 1234.12345");
                assert_eq!(
                    error.code.as_deref(),
                    Some("incorrect_id_format_for_1234.12345")
                );
                assert!(error.link.contains("arxiv.org/api/errors"));
                assert_eq!(error.updated, "2007-10-12T00:00:00-04:00");
            }
            ArxivResponse::Results(_) => panic!("expected an error response"),
        }
    }

    #[test]
    fn successful_feed_parses_into_results_variant() {
        match ArxivResponse::from_atom(ELECTRON_FEED).expect("feed parses") {
            ArxivResponse::Results(response) => {
                assert_eq!(response.entries.len(), 2);
                assert_eq!(response.total_results, 182239);
            }
            ArxivResponse::Error(_) => panic!("expected a results response"),
        }
    }
}
