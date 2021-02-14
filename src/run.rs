use crate::{args::Args, enricher, progressbar, VERBOSE};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::{
    fs::{self, File},
    num::ParseFloatError,
};

pub fn run(args: Args) -> Result<(), Option<String>> {
    printlnv!("Starting...");
    progressbar::start_progress_bar(100);
    let products_with_variation = get_products_with_variations(&args.file)?;
    progressbar::inc_progress_bar(10);
    let mut products = get_products_from_variations(products_with_variation, args.limit);
    progressbar::inc_progress_bar(10);
    progressbar::set_progress_bar_len((products.len() as f64 / 0.8).round() as u64);
    enricher::enrich_products(&args.url, &mut products, args.simulate)?;
    let (products_file, variations_file) = args.get_output_files();
    save_enriched_products_to_file(products, products_file, variations_file)?;
    progressbar::finish_progress_bar();
    printlnv!("Done!");
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
    unsafe {
        if VERBOSE {
            printlnpb!("Products read from csv file:");
            for product_with_variation in products.iter() {
                printlnpb!("{:?}", product_with_variation);
            }
            printlnpb!("");
        }
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
            let product_id = product_with_variation.produto;
            if ps.iter_mut().find(|p2| p2.id == product_id).is_none() && limit == 0
                || (ps.len() as u32) < limit
            {
                // todo: work around usize limit in products, see ps.len above
                ps.push(Product {
                    id: product_id,
                    name: product_with_variation.nome,
                    variations: vec![],
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
pub struct Product {
    pub id: String,
    pub name: String,
    pub variations: Vec<Variation>,
    pub stock: Option<u32>,
    pub price: f64,
    pub price_cost: Option<f64>,
    pub vendor_name: String,
    pub description: String,
    pub category: String,
    pub subcategory: String,
    pub pictures: Vec<String>,
}

#[derive(Debug)]
pub struct Variation {
    pub type1: String,
    pub type2: Option<String>,
    pub type3: Option<String>,
    pub name1: String,
    pub name2: Option<String>,
    pub name3: Option<String>,
    pub price: f64,
    pub picture: Option<String>,
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
