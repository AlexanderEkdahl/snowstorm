CREATE VIEW products_with_attributes AS
SELECT products.id,
       products.name,
       array_to_json(array(SELECT v.value
                           FROM attributes
                           LEFT JOIN (SELECT *
                                      FROM attribute_values
                                      WHERE attribute_values.product_id = products.id) AS v 
                           ON attributes.id = v.attribute_id
                           ORDER BY attributes.id))
FROM products
ORDER BY products.id;

\copy (SELECT * FROM products_with_attributes) TO '../data/products.csv' DELIMITER ',' CSV
\copy (SELECT name, description, value, compare_type, up, down FROM attributes ORDER BY attributes.id) TO '../data/attributes.csv' DELIMITER ',' CSV

DROP VIEW products_with_attributes;