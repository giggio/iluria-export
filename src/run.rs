use crate::args::Args;
use scraper::{Html, Selector};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::{
    fs::{self, File},
    num::ParseFloatError,
};

pub fn run(args: Args) -> Result<(), Option<String>> {
    let products_with_variation = get_products_with_variations(&args.file)?;
    for product_with_variation in products_with_variation.iter() {
        printlnv!("{:?}", product_with_variation);
    }
    let mut products = get_products_from_variations(products_with_variation, args.limit);
    enrich_products(&args.url, &mut products)?;
    let (products_file, variations_file) = args.get_output_files();
    save_enriched_products_to_file(products, products_file, variations_file)?;
    Ok(())
}

fn convert_number(str: &str) -> Result<f64, ParseFloatError> {
    let x = str.replace(".", "").replace(",", ".");
    let number = x.parse::<f64>()?;
    Ok(number)
}

fn get_products_with_variations(file: &str) -> Result<Vec<ProductWithVariation>, String> {
    let file = File::open(&std::path::Path::new(file))
        .map_err(|err| format!("Error when opening summary file: {}", err))?;
    let file_contents =
        open_file_and_decode(file).map_err(|e| format!("Could not read text file: {}", e))?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file_contents.as_bytes());
    let mut products = vec![];
    for result in rdr.deserialize() {
        let product: ProductWithVariation =
            result.map_err(|e| format!("Could not map row: {}", e))?;
        products.push(product);
    }
    Ok(products)
}

fn open_file_and_decode(file: File) -> std::io::Result<String> {
    let mut decoder = encoding_rs_io::DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding_rs::WINDOWS_1252))
        .build(&file);
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}

fn encode_ansi(str: String) -> Vec<u8> {
    encoding_rs::WINDOWS_1252.encode(&str).0.to_vec()
}

fn get_products_from_variations(
    products_with_variation: Vec<ProductWithVariation>,
    limit: u32,
) -> Vec<Product> {
    products_with_variation
        .into_iter()
        .fold(vec![], |mut ps, product_with_variation| {
            let variation = Variation {
                one: product_with_variation.variacao_1,
                two: product_with_variation.variacao_2,
                three: product_with_variation.variacao_3,
            };
            let product_id = product_with_variation.produto;
            if let Some(product) = ps.iter_mut().find(|p2| p2.id == product_id) {
                product.variations.push(variation);
            } else if limit == 0 || (ps.len() as u32) < limit {
                // todo: work around usize limit in products, see ps.len above
                ps.push(Product {
                    id: product_id,
                    name: product_with_variation.nome,
                    variations: vec![variation],
                    stock: product_with_variation.estoque,
                    price: product_with_variation.preco,
                    price_cost: product_with_variation.preco_de_custo,
                    vendor_name: product_with_variation.nome_do_fornecedor,
                    description: "".to_owned(),
                    category: "".to_owned(),
                    subcategory: "".to_owned(),
                    pictures: vec![],
                });
            }
            ps
        })
}

fn enrich_products(base_url: &str, products: &mut Vec<Product>) -> Result<(), String> {
    for product in products.iter_mut() {
        let url = format!("{}/pd-{}", base_url, product.id);
        printlnv!("Making web request at: {}", url);
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
            // d.text().collect()
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

fn save_enriched_products_to_file(
    products: Vec<Product>,
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
                    .filter_map(|v| {
                        if v.one.is_empty() {
                            None
                        } else {
                            Some(VariationCsvExport {
                                product_id: i.to_string(),
                                type1: "var1".to_owned(),
                                name1: v.one,
                                type2: if v.two.is_empty() {
                                    None
                                } else {
                                    Some("var2".to_owned())
                                },
                                name2: if v.two.is_empty() { None } else { Some(v.two) },
                            })
                        }
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
        None => println!("Products:\n{}", product_text),
        Some(file) => fs::write(file, encode_ansi(product_text))
            .map_err(|e| format!("Error products writing file: {}", e))?,
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
        None => println!("Variations:\n{}", variation_text),
        Some(file) => fs::write(file, encode_ansi(variation_text))
            .map_err(|e| format!("Error variations writing file: {}", e))?,
    }
    Ok(())
}

fn number_with_comma<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    Ok(convert_number(&str).map_err(de::Error::custom)?)
}

fn optional_number_with_comma<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    if str.is_empty() {
        return Ok(None);
    }
    Ok(Some(convert_number(&str).map_err(de::Error::custom)?))
}

#[derive(Debug, Deserialize)]
struct ProductWithVariation {
    #[serde(rename = "Produto")]
    produto: String,
    #[serde(rename = "Nome")]
    nome: String,
    #[serde(rename = "Variação 1")]
    variacao_1: String,
    #[serde(rename = "Variação 2")]
    variacao_2: String,
    #[serde(rename = "Variação 3")]
    variacao_3: String,
    #[serde(rename = "Estoque")]
    #[serde(deserialize_with = "csv::invalid_option")]
    estoque: Option<u32>,
    #[serde(rename = "Preço")]
    #[serde(deserialize_with = "number_with_comma")]
    preco: f64,
    #[serde(rename = "Preço de custo")]
    #[serde(deserialize_with = "optional_number_with_comma")]
    preco_de_custo: Option<f64>,
    #[serde(rename = "Nome do fornecedor")]
    nome_do_fornecedor: String,
}

#[derive(Debug)]
struct Product {
    id: String,
    name: String,
    variations: Vec<Variation>,
    stock: Option<u32>,
    price: f64,
    price_cost: Option<f64>,
    vendor_name: String,
    description: String,
    category: String,
    subcategory: String,
    pictures: Vec<String>,
}

#[derive(Debug)]
struct Variation {
    one: String,
    two: String,
    three: String,
}

#[derive(Debug, Serialize)]
struct ProductCsvExport {
    id: String,
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
    product_id: String,
    type1: String,
    name1: String,
    type2: Option<String>,
    name2: Option<String>,
}
