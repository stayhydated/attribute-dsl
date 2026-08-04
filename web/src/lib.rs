use dioxus::prelude::*;
use stayhydated_dioxus::{Project, ProjectSite, StayhydatedSinglePageProjectApp};

const PROJECT: Project = Project::new(
    "attribute-dsl",
    "Shared parser helpers for Rust proc-macro attribute DSLs.",
)
.with_skill_command("npx skills add stayhydated/attribute-dsl");
const SITE_URL: &str = "https://stayhydated.github.io/attribute-dsl/";
const RUSTDOC_URL: &str = "https://docs.rs/attribute-dsl/";
const SOURCE_URL: &str = "https://github.com/stayhydated/attribute-dsl";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn site() -> ProjectSite {
    ProjectSite::builder()
        .project(PROJECT)
        .site_url(SITE_URL)
        .rustdoc_url(RUSTDOC_URL)
        .source_url(SOURCE_URL)
        .version(VERSION)
        .build()
}

#[component]
pub fn App() -> Element {
    rsx! { StayhydatedSinglePageProjectApp { site: site() } }
}

pub fn route_manifest() -> stayhydated_site::SiteRouteManifest {
    site().single_page_route_manifest()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_page_site_uses_project_owned_destinations_without_demos() {
        let site = site();

        assert_eq!(site.rustdoc_url(), RUSTDOC_URL);
        assert_eq!(site.source_url(), SOURCE_URL);
        assert_eq!(site.demo_path(), None);
        assert_eq!(
            site.project().skill_command(),
            Some("npx skills add stayhydated/attribute-dsl")
        );
        assert_eq!(route_manifest().application_paths()[0].as_str(), "/");
    }
}
