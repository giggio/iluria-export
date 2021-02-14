use crate::{args::Args, enricher, exporter, importer, progressbar};

pub fn run(args: Args) -> Result<(), Option<String>> {
    printlnv!("Starting...");
    progressbar::start_progress_bar(100);
    let products_with_variation = importer::get_products_with_variations(&args.file)?;
    progressbar::inc_progress_bar(10);
    let mut products = get_products_from_variations(products_with_variation, args.limit);
    progressbar::inc_progress_bar(10);
    progressbar::set_progress_bar_len((products.len() as f64 / 0.8).round() as u64);
    enricher::enrich_products(&args.url, &mut products, args.simulate)?;
    let (products_file, variations_file) = args.get_output_files();
    exporter::save_enriched_products_to_file(products, products_file, variations_file)?;
    progressbar::finish_progress_bar();
    printlnv!("Done!");
    Ok(())
}

fn get_products_from_variations(
    products_with_variation: Vec<importer::ProductWithVariation>,
    limit: u32,
) -> Vec<Product> {
    products_with_variation
        .into_iter()
        .fold(vec![], |mut ps, product_with_variation| {
            let product_id = product_with_variation.produto;
            if ps.iter().find(|p| p.id == product_id).is_none()
                && (limit == 0 || (ps.len() as u32) < limit)
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
