//! URL classification: which links stay inside the app and how meeting links are
//! normalised before navigating the main window.

use url::Url;

const TEAMS_HOSTS: &[&str] = &["teams.microsoft.com", "teams.cloud.microsoft", "teams.live.com"];

/// Hosts that must open inside the app because they are part of sign-in or
/// Teams itself. Anything else requested via `window.open` goes to the browser.
const IN_APP_SUFFIXES: &[&str] = &[
    "teams.microsoft.com",
    "teams.cloud.microsoft",
    "teams.live.com",
    "login.microsoftonline.com",
    "login.microsoft.com",
    "login.live.com",
    "account.live.com",
    "microsoftonline.com",
    "outlook.office.com",
    "outlook.office365.com",
    "outlook.cloud.microsoft",
    "outlook.live.com",
];

fn host_matches(host: &str, suffix: &str) -> bool {
    host == suffix || host.ends_with(&format!(".{suffix}"))
}

pub fn is_teams(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str().is_some_and(|h| TEAMS_HOSTS.iter().any(|t| host_matches(h, t)))
}

pub fn opens_in_app(url: &Url) -> bool {
    url.scheme() == "about"
        || (url.scheme() == "https"
            && url.host_str().is_some_and(|h| IN_APP_SUFFIXES.iter().any(|s| host_matches(h, s))))
}

/// Only these schemes may be handed to the desktop's URL opener.
pub fn is_safe_external(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http" | "mailto" | "tel")
}

/// Accepts `https://teams.microsoft.com/l/meetup-join/...`, `msteams:/l/...`
/// and links pasted with surrounding whitespace or text.
pub fn meeting_url(input: &str) -> Option<Url> {
    let input = input.trim();
    let candidate = input
        .split_whitespace()
        .map(|w| w.trim_matches(|c| matches!(c, '<' | '>' | '"' | '\'' | '(' | ')')))
        .find(|w| w.starts_with("https://") || w.starts_with("msteams:"))
        .unwrap_or(input);

    let url = if let Some(rest) = candidate.strip_prefix("msteams:") {
        Url::parse(&format!("https://teams.microsoft.com/{}", rest.trim_start_matches('/'))).ok()?
    } else {
        Url::parse(candidate).ok()?
    };
    is_teams(&url).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeting_links() {
        let u = meeting_url("  https://teams.microsoft.com/l/meetup-join/19%3ameeting_x/0?context=%7b%7d ").unwrap();
        assert_eq!(u.host_str(), Some("teams.microsoft.com"));
        let u = meeting_url("msteams:/l/meetup-join/abc").unwrap();
        assert_eq!(u.as_str(), "https://teams.microsoft.com/l/meetup-join/abc");
        assert!(meeting_url("Join: <https://teams.live.com/meet/123>").is_some());
        assert!(meeting_url("https://evil.example/teams.microsoft.com").is_none());
        assert!(meeting_url("https://teams.microsoft.com.evil.example/").is_none());
        assert!(meeting_url("http://teams.microsoft.com/").is_none());
    }

    #[test]
    fn in_app_routing() {
        assert!(opens_in_app(&Url::parse("https://login.microsoftonline.com/common").unwrap()));
        assert!(!opens_in_app(&Url::parse("https://github.com/").unwrap()));
        assert!(!is_safe_external(&Url::parse("file:///etc/passwd").unwrap()));
    }
}
