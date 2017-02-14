CREATE VIEW subset_attributes AS
SELECT attributes.id, attributes.name, attributes.description, attributes.value, attributes.compare_type, attributes.up, attributes.down
FROM attributes
JOIN attribute_subcategories
ON attributes.id = attribute_subcategories.attribute_id
JOIN subcategories
ON attribute_subcategories.subcategory_id = subcategories.id
WHERE subcategories.name = 'Diskmaskin'
ORDER BY attributes.id;

CREATE VIEW subset_with_attributes AS
SELECT products.id,
       products.name,
       array_to_json(array(SELECT v.value
                           FROM subset_attributes
                           LEFT JOIN (SELECT *
                                      FROM attribute_values
                                      WHERE attribute_values.product_id = products.id) AS v 
                           ON subset_attributes.id = v.attribute_id
                           ORDER BY subset_attributes.id))
FROM products
JOIN subcategories
ON products.subcategory_id = subcategories.id
WHERE subcategories.name = 'Diskmaskin'
ORDER BY products.id;

\copy (SELECT name, value, compare_type, up, down FROM subset_attributes) TO '../data/subset_attributes.csv' DELIMITER ',' CSV
\copy (SELECT * FROM subset_with_attributes) TO '../data/subset.csv' DELIMITER ',' CSV

DROP VIEW subset_with_attributes;
DROP VIEW subset_attributes;
