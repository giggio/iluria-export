use serde::Serialize;
use std::fs;

pub fn save_enriched_products_to_file(
    products: Vec<crate::run::Product>,
    products_file: Option<String>,
    variations_file: Option<String>,
) -> Result<(), String> {
    let mut i: u32 = 0;
    let (product_export, variation_export) = products
        .into_iter()
        .map(|p| {
            i += 1;
            let len = p.pictures.len();
            let mut pics = p.pictures;
            let picture1 = if len > 0 {
                pics.remove(0)
            } else {
                "".to_owned()
            };
            let picture2 = if len > 0 {
                pics.remove(0)
            } else {
                "".to_owned()
            };
            let picture3 = if len > 0 {
                pics.remove(0)
            } else {
                "".to_owned()
            };
            let picture4 = if len > 0 {
                pics.remove(0)
            } else {
                "".to_owned()
            };
            let picture5 = if len > 0 {
                pics.remove(0)
            } else {
                "".to_owned()
            };
            (
                ProductCsvExport {
                    id: i.to_string(),
                    active: "Sim".to_owned(),
                    name: p.name,
                    stock: p.stock,
                    price: p.price,
                    price_cost: p.price_cost,
                    vendor_name: p.vendor_name,
                    description: p.description.trim().to_owned(),
                    category: p.category,
                    subcategory: p.subcategory,
                    picture1,
                    picture2,
                    picture3,
                    picture4,
                    picture5,
                },
                p.variations
                    .into_iter()
                    .map(|v| VariationCsvExport {
                        id: "".to_owned(),
                        product_id: i.to_string(),
                        type1: v.type1,
                        name1: v.name1,
                        type2: v.type2,
                        name2: v.name2,
                        type3: v.type3,
                        name3: v.name3,
                        price: v.price,
                        picture: v.picture,
                    })
                    .collect::<Vec<VariationCsvExport>>(),
            )
        })
        .fold((vec![], vec![]), |(mut ps, mut vss), (p, mut vs)| {
            ps.push(p);
            vss.append(&mut vs);
            (ps, vss)
        });
    let mut product_wtr = csv::Writer::from_writer(vec![]);
    for p in product_export.into_iter() {
        product_wtr
            .serialize(&p)
            .map_err(|e| format!("Could not serialize product {:?}. Details: {}", &p, e))?;
    }
    let product_text = String::from_utf8(product_wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    match products_file {
        None => printlnpb!("Products:\n{}", product_text),
        Some(file) => fs::write(&file, encode_ansi(product_text))
            .map_err(|e| format!("Error when writing products file '{}': {}", file, e))?,
    }
    let mut variation_wtr = csv::Writer::from_writer(vec![]);
    for v in variation_export.into_iter() {
        variation_wtr
            .serialize(&v)
            .map_err(|e| format!("Could not serialize variation {:?}. Details: {}", &v, e))?;
    }
    let variation_text = String::from_utf8(variation_wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    match variations_file {
        None => printlnpb!("Variations:\n{}", variation_text),
        Some(file) => fs::write(&file, encode_ansi(variation_text))
            .map_err(|e| format!("Error when writing variations file '{}': {}", file, e))?,
    }
    Ok(())
}

fn encode_ansi(str: String) -> Vec<u8> {
    encoding_rs::WINDOWS_1252.encode(&str).0.to_vec()
}

#[derive(Debug, Serialize)]
struct ProductCsvExport {
    id: String,
    active: String,
    name: String,
    stock: Option<u32>,
    price: f64,
    price_cost: Option<f64>,
    vendor_name: String,
    description: String,
    category: String,
    subcategory: String,
    picture1: String,
    picture2: String,
    picture3: String,
    picture4: String,
    picture5: String,
}

#[derive(Debug, Serialize)]
struct VariationCsvExport {
    id: String,
    product_id: String,
    type1: String,
    name1: String,
    type2: Option<String>,
    name2: Option<String>,
    type3: Option<String>,
    name3: Option<String>,
    pub price: f64,
    pub picture: Option<String>,
}
