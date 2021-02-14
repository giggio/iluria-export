use crate::VERBOSE;
use serde::{de, Deserialize, Deserializer};
use std::io::Read;
use std::{fs::File, num::ParseFloatError};

pub fn get_products_with_variations(file: &str) -> Result<Vec<ProductWithVariation>, String> {
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

fn convert_number(str: &str) -> Result<f64, ParseFloatError> {
    let x = str.replace(".", "").replace(",", ".");
    let number = x.parse::<f64>()?;
    Ok(number)
}

#[derive(Debug, Deserialize)]
pub struct ProductWithVariation {
    #[serde(rename = "Produto")]
    pub produto: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "Estoque")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub estoque: Option<u32>,
    #[serde(rename = "Preço")]
    #[serde(deserialize_with = "number_with_comma")]
    pub preco: f64,
    #[serde(rename = "Preço de custo")]
    #[serde(deserialize_with = "optional_number_with_comma")]
    pub preco_de_custo: Option<f64>,
    #[serde(rename = "Nome do fornecedor")]
    pub nome_do_fornecedor: String,
}
