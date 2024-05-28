pub fn capture_page(url: &str) -> Option<String> {
    reqwest::blocking::get(url).ok()?.text().ok()
}
