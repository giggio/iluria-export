use crate::args::Args;
use serde::{de, Deserialize, Deserializer};
use std::io::Read;
use std::{fs::File, num::ParseFloatError};

pub fn run(args: Args) -> Result<(), Option<String>> {
    let products_with_variation = get_products_with_variations(&args.file)?;
    for product_with_variation in products_with_variation.iter() {
        println!("{:?}", product_with_variation);
    }
    let products = get_products_from_variations(products_with_variation);
    enrich_products(&products)?;
    save_enriched_products_to_file(products)?;
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
        convert_enconding(file).map_err(|e| format!("Could not read text file: {}", e))?;
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

fn convert_enconding(file: File) -> std::io::Result<String> {
    let mut decoder = encoding_rs_io::DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding_rs::WINDOWS_1252))
        .build(&file);
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}

fn get_products_from_variations(
    products_with_variation: Vec<ProductWithVariation>,
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
            } else {
                ps.push(Product {
                    id: product_id,
                    name: product_with_variation.nome,
                    variations: vec![variation],
                    stock: product_with_variation.estoque,
                    price: product_with_variation.preco,
                    price_cost: product_with_variation.preco_de_custo,
                    vendor_name: product_with_variation.nome_do_fornecedor,
                });
            }
            ps
        })
}

fn enrich_products(products: &Vec<Product>) -> Result<(), String> {
    todo!()
}

fn save_enriched_products_to_file(products: Vec<Product>) -> Result<(), String> {
    for product in products.into_iter() {
        println!("{:?}", product);
    }
    todo!()
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
}

#[derive(Debug)]
struct Variation {
    one: String,
    two: String,
    three: String,
}
