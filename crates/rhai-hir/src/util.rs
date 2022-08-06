use url::Url;

/// Return the rhai script url from an url.
///
/// This is needed for rhai definitions that typically
/// end with `.d.rhai`.
///
/// If the URL is already a script URL, [`None`] is returned.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub fn script_url(url: &Url) -> Option<Url> {
    let script_name = match url.path_segments().and_then(Iterator::last) {
        Some(name) => name,
        None => {
            tracing::debug!(%url, "could not figure out script url");
            return None;
        }
    };

    if let Some(script_name_base) = script_name.strip_suffix(".d.rhai") {
        Some(url.join(&format!("{script_name_base}.rhai")).unwrap())
    } else if url.as_str().ends_with(".rhai") {
        None
    } else {
        tracing::debug!(%url, "could not figure out script url");
        None
    }
}
