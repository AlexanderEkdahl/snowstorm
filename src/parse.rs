extern crate csv;
extern crate rustc_serialize;

use models::*;
use std::collections::HashMap;

pub fn attributes() -> Result<Vec<Attribute>, csv::Error> {
    let mut rdr = csv::Reader::from_file("./data/attributes.csv")?.has_headers(false);
    let mut attributes = Vec::new();

    for record in rdr.decode() {
        let (name, values, compare_type, parameter_1, parameter_2): (String, String, u16, u16, u16) =
        record?;
        let values: Vec<String> = values.split(',').map(|s| s.to_string()).collect();
        let compare_type = match compare_type {
            1 => CompareType::Equal,
            2 => CompareType::GreaterThanOrEqual,
            3 => CompareType::LessThanOrEqual,
            6 => CompareType::IntervalMatch(parameter_1, parameter_2),
            50 => CompareType::LinearInterval(parameter_2, parameter_1),
            _ => panic!("Unknown compare type: {}", compare_type),
        };
        attributes.push(Attribute {
            name: name,
            values: values,
            compare_type: compare_type,
        });
    }

    Ok(attributes)
}

pub fn products(attributes: &Vec<Attribute>) -> Result<HashMap<ProductId, Product>, csv::Error> {
    let mut rdr = csv::Reader::from_file("./data/products.csv")?.has_headers(false);
    let mut products = HashMap::new();

    for record in rdr.decode() {
        let (id, name, values): (ProductId, String, String) = record?;
        let values: Vec<Option<String>> = rustc_serialize::json::decode(&values).unwrap();
        let values: Vec<_> = values.into_iter()
            .zip(attributes)
            .map(|(x, attribute)| attribute.map_value(x))
            .collect();

        products.insert(id,
                        Product {
                            id: id,
                            name: name,
                            values: values,
                        });
    }

    Ok(products)
}

pub fn matches(products: &HashMap<ProductId, Product>) -> Result<Vec<Match>, csv::Error> {
    let mut rdr = csv::Reader::from_file("./data/matches.csv")?.has_headers(false);
    let mut matches = Vec::new();

    for record in rdr.decode() {
        let (original, new): (ProductId, ProductId) = record?;

        if products.contains_key(&original) && products.contains_key(&new) {
            matches.push(Match(original, new));
        }
    }

    Ok(matches)
}
