use crate::progressbar;
use scraper::{Html, Selector};

pub fn enrich_products(
    base_url: &str,
    products: &mut Vec<crate::run::Product>,
    simulate: bool,
) -> Result<(), String> {
    for product in products.iter_mut() {
        let url = format!("{}/pd-{}", base_url, product.id);
        progressbar::inc_progress_bar(1);
        if simulate {
            printlnv!("Simulating web request at: {}", url);
            std::thread::sleep(std::time::Duration::from_millis(300));
            continue;
        } else {
            printlnv!("Making web request at: {}", url);
        }
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&url)
            .header("user-agent", "Mozilla/5.0")
            .send()
            .map_err(|e| format!("Could not get at {}. Details: {}", url, e))?;
        if !resp.status().is_success() {
            return Err(format!(
                "Request for product {} failed with status code {}",
                product.id,
                resp.status()
            ));
        }
        let body = resp
            .text()
            .map_err(|e| format!("Could not get body: {}", e))?;
        let fragment = Html::parse_document(&body);
        let description_selector =
            Selector::parse("div:not([id]).product-description").map_err(|e| {
                format!(
                    "Could not get description for product {}: {:?}",
                    product.id, e
                )
            })?;
        let description: String = if let Some(d) = fragment.select(&description_selector).next() {
            d.inner_html()
        } else {
            "".to_owned()
        };
        product.description = description;
        let category_selector = Selector::parse(".breadcrumb a")
            .map_err(|e| format!("Could not get category for product {}: {:?}", product.id, e))?;
        let category_and_subcategory: Vec<_> =
            fragment.select(&category_selector).skip(2).collect();
        product.category = if !category_and_subcategory.is_empty() {
            category_and_subcategory[0].text().collect()
        } else {
            "".to_owned()
        };
        product.subcategory = if category_and_subcategory.len() == 2 {
            category_and_subcategory[1].text().collect()
        } else {
            "".to_owned()
        };
        let images_selector = Selector::parse("#thumbsContainer img")
            .map_err(|e| format!("Could not get images for product {}: {:?}", product.id, e))?;
        product.pictures = fragment
            .select(&images_selector)
            .filter_map(|i| i.value().attr("mainpictureurl"))
            .map(|s| {
                if s.starts_with('/') {
                    format!("http:{}", s)
                } else {
                    s.to_owned()
                }
            })
            .collect();
        printlnv!("Enriched product: {:?}", product);
    }
    Ok(())
}
