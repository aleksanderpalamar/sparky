use fluent_bundle::FluentArgs;

use crate::app::search::Search;
use crate::app::state::Files;
use crate::domain::search::Finish;
use crate::i18n;

pub fn title(files: &Files, search: &Search) -> String {
    let mut args = FluentArgs::new();
    args.set("query", search.query.clone());
    args.set("place", files.shown(&files.location));
    i18n::message("search-title", Some(&args))
}

pub fn summary(files: &Files, search: &Search) -> String {
    let count = files.listing.total_len();
    match search.finish {
        None => i18n::counted("search-running", count),
        Some(Finish::Limited) => i18n::counted("search-limited", count),
        Some(Finish::Complete | Finish::Cancelled) => i18n::counted("search-complete", count),
    }
}
