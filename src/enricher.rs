use std::collections::HashMap;

use crate::{progressbar, run::Product, run::Variation};
use scraper::{Html, Selector};

pub fn enrich_products(
    base_url: &str,
    products: &mut Vec<Product>,
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
        product.description = get_description(&fragment, &product.id)?;
        let (category, subcategory) = get_category(&fragment, &product.id)?;
        product.category = category;
        product.subcategory = subcategory;
        product.pictures = get_pictures(&fragment, &product.id)?;
        product.variations = get_variations(&fragment, &product.id)?
            .into_iter()
            .map(|v| Variation {
                type1: v.type1,
                type2: v.type2,
                type3: v.type3,
                name1: v.name1,
                name2: v.name2,
                name3: v.name3,
                picture: v.picture,
                price: v.price,
            })
            .collect();
        printlnv!("Enriched product: {:?}", product);
    }
    Ok(())
}

fn get_variations(fragment: &Html, product_id: &str) -> Result<Vec<VariationWithId>, String> {
    let variations_selector = Selector::parse("input.allVariations").map_err(|e| {
        format!(
            "Could not get variations for product {}: {:?}",
            product_id, e
        )
    })?;
    let variations_select = fragment.select(&variations_selector);
    let mut variations = variations_select
        .map(|e| e.value())
        .map(|e| {
            printlnv!("Found variation input: {:?}", e);
            VariationWithId {
                type1: "".to_owned(),
                type2: None,
                type3: None,
                name1: "".to_owned(),
                name2: None,
                name3: None,
                id1: if let Some(v) = e.attr("value1") {
                    v.trim()
                } else {
                    ""
                }
                .to_owned(),
                id2: if let Some(v) = e.attr("value2") {
                    let trimmed = v.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_owned())
                    }
                } else {
                    None
                },
                id3: if let Some(v) = e.attr("value3") {
                    let trimmed = v.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_owned())
                    }
                } else {
                    None
                },
                price: e
                    .attr("convertedprice")
                    .unwrap_or("R$ 0,00")
                    .replace("R$ ", "")
                    .replace(".", "")
                    .replace(",", ".")
                    .parse::<f64>()
                    .unwrap(),
                picture: e.attr("mainpictureurl550").map(|v| get_picture_url(v)),
            }
        })
        .collect::<Vec<_>>();

    let mut variation_ids_1 = HashMap::new();
    for id in variations
        .iter()
        .map(|v| &v.id1)
        .filter(|id| !id.is_empty())
    {
        variation_ids_1.insert(id.clone(), "".to_owned());
    }
    let mut variation_ids_2 = HashMap::new();
    let mut variation_ids_3 = HashMap::new();
    for id in variations.iter().map(|v| v.id2.as_ref()).filter_map(|v| v) {
        variation_ids_2.insert(id.clone(), "".to_owned());
    }
    for id in variations.iter().map(|v| v.id3.as_ref()).filter_map(|v| v) {
        variation_ids_3.insert(id.clone(), "".to_owned());
    }

    let variation_type_1 =
        get_text_from_selector(fragment, "#iluria-product-variation1 > option[value='0']")?;
    let variation_type_2 =
        get_text_from_selector(fragment, "#iluria-product-variation2 > option[value='0']")?;
    let variation_type_3 =
        get_text_from_selector(fragment, "#iluria-product-variation3 > option[value='0']")?;

    for (id, value) in variation_ids_1.iter_mut() {
        *value = if let Some(value) = get_text_from_selector(
            fragment,
            &format!("#iluria-product-variation1 > option[value='{}']", id),
        )? {
            value
        } else {
            return Err(format!("Could not find variation value for id '{}'.", id));
        }
    }
    for (id, value) in variation_ids_2.iter_mut() {
        *value = if let Some(value) = get_text_from_selector(
            fragment,
            &format!("#iluria-product-variation2 > option[value='{}']", id),
        )? {
            value
        } else {
            return Err(format!("Could not find variation value for id '{}'.", id));
        }
    }
    for (id, value) in variation_ids_3.iter_mut() {
        *value = if let Some(value) = get_text_from_selector(
            fragment,
            &format!("#iluria-product-variation3 > option[value='{}']", id),
        )? {
            value
        } else {
            return Err(format!("Could not find variation value for id '{}'.", id));
        }
    }
    for variation in variations.iter_mut() {
        variation.type1 = variation_type_1.clone().unwrap();
        variation.type2 = variation_type_2.clone();
        variation.type3 = variation_type_3.clone();
        variation.name1 = variation_ids_1[&variation.id1].clone();
        if let Some(v) = &variation.id2 {
            variation.name2 = Some(variation_ids_2[v].clone());
        }
        if let Some(v) = &variation.id3 {
            variation.name3 = Some(variation_ids_3[v].clone());
        }
    }
    Ok(variations)
}

fn get_text_from_selector(fragment: &Html, selector: &str) -> Result<Option<String>, String> {
    let variation1_selector = Selector::parse(selector)
        .map_err(|e| format!("Could not get value for selector {}: {:?}", selector, e))?;
    let mut select = fragment.select(&variation1_selector);
    Ok(if let Some(d) = select.next() {
        Some(d.text().collect::<String>().trim().to_owned())
    } else {
        None
    })
}

fn get_description(fragment: &Html, product_id: &str) -> Result<String, String> {
    let description_selector =
        Selector::parse("div:not([id]).product-description").map_err(|e| {
            format!(
                "Could not get description for product {}: {:?}",
                product_id, e
            )
        })?;
    Ok(
        if let Some(d) = fragment.select(&description_selector).next() {
            d.inner_html()
        } else {
            "".to_owned()
        },
    )
}

fn get_category(fragment: &Html, product_id: &str) -> Result<(String, String), String> {
    let category_selector = Selector::parse(".breadcrumb a")
        .map_err(|e| format!("Could not get category for product {}: {:?}", product_id, e))?;
    let category_and_subcategory: Vec<_> = fragment.select(&category_selector).skip(2).collect();
    let category = if !category_and_subcategory.is_empty() {
        category_and_subcategory[0].text().collect()
    } else {
        "".to_owned()
    };
    let subcategory = if category_and_subcategory.len() == 2 {
        category_and_subcategory[1].text().collect()
    } else {
        "".to_owned()
    };
    Ok((category, subcategory))
}

fn get_pictures(fragment: &Html, product_id: &str) -> Result<Vec<String>, String> {
    let images_selector = Selector::parse("#thumbsContainer img")
        .map_err(|e| format!("Could not get images for product {}: {:?}", product_id, e))?;
    let pictures = fragment
        .select(&images_selector)
        .filter_map(|i| i.value().attr("mainpictureurl"))
        .map(|s| get_picture_url(s))
        .collect();
    Ok(pictures)
}

fn get_picture_url(url: &str) -> String {
    if url.starts_with('/') {
        format!("http:{}", url)
    } else {
        url.to_owned()
    }
}

struct VariationWithId {
    id1: String,
    id2: Option<String>,
    id3: Option<String>,
    type1: String,
    type2: Option<String>,
    type3: Option<String>,
    name1: String,
    name2: Option<String>,
    name3: Option<String>,
    price: f64,
    picture: Option<String>,
}
