-- Number of each type within groups
SELECT categories.name, count(*) as count
FROM products
JOIN subcategories ON products.subcategory_id = subcategories.id
JOIN categories ON subcategories.category_id = categories.id
GROUP BY categories.name
ORDER BY count DESC;

-- Number of each type within subcategories
SELECT subcategories.name, count(*) as count
FROM products
JOIN subcategories ON products.subcategory_id = subcategories.id
GROUP BY subcategories.name
ORDER BY count DESC;

-- Get average and maximum number of attributes per category
DROP VIEW IF EXISTS products_with_attribute_count;

CREATE VIEW products_with_attribute_count AS
SELECT *,
  (SELECT count(*)
   FROM attribute_values
   WHERE attribute_values.product_id = products.id) AS attribute_count
FROM products;

SELECT categories.name,
       avg(products_with_attribute_count.attribute_count) AS avg_attribute_count,
       max(products_with_attribute_count.attribute_count) AS max_attribute_count
FROM products_with_attribute_count
JOIN subcategories ON products_with_attribute_count.subcategory_id = subcategories.id
JOIN categories ON subcategories.category_id = categories.id
GROUP BY categories.name;

DROP VIEW IF EXISTS products_with_attribute_count;