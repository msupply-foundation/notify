# Reference Notifications

This document covers reference notification configurations for stock management relating to mSupply.

1. **Critical Stock Weekly** — a weekly digest highlighting items that have fallen below a critical stock level.
2. **Daily Stock Outs** — a daily alert listing items that are currently complets

---

## 1. Critical Stock Weekly

### Purpose

Sent once per week, this notification summarises all stock items whose quantity on hand has dropped below 3 months of stock.

### Step 1 — Create the Notification Queries

Navigate to **Notification Queries** in the admin UI and create the following queries.

#### Data Query — `critical_stock`

```sql
with pos AS(
  SELECT
    po.id,
    po.store_id,
    pol.item_id,
    po.confirm_date,
    po.serial_number,
    n.name AS supplier_name,
    pol.delivery_date_expected,
    RANK() OVER (PARTITION BY po.store_id, pol.item_id ORDER BY po.confirm_date DESC) AS ranking,
    pol.quan_adjusted_order-pol.quan_rec_to_date AS po_outstanding
  FROM purchase_order po
  JOIN purchase_order_line pol ON po.id = pol.purchase_order_id
  JOIN name n ON po.name_id = n.id
  WHERE po.status IN ('cn')
  AND pol.quan_adjusted_order-pol.quan_rec_to_date > 0
),
current_soh AS (
SELECT il.item_id, il.store_id, COALESCE(SUM(il.quantity * il.pack_size), 0) AS soh
FROM item_line il
WHERE (il.expiry_date IS NULL OR il.expiry_date >= current_date)
GROUP BY il.item_id, il.store_id
)
SELECT
s.code,
s.name as store_name,
i.code as item_code,
i.item_name,
u.units,
max(aggamc.value) as amc,
COALESCE(max(current_soh.soh), 0) AS soh,
max(aggmos.value) as mos,
max(pos.confirm_date) as latest_po_date,
max(pos.serial_number) as po_number,
max(pos.po_outstanding) as stock_on_order,
max(pos.delivery_date_expected) as delivery_date,
max(pos.supplier_name) as supplier
FROM
store s
CROSS JOIN item i
LEFT JOIN unit u ON i.unit_ID = u.id
LEFT JOIN current_soh ON current_soh.item_id = i.id AND current_soh.store_id = s.id
LEFT JOIN aggregator aggamc ON i.id = aggamc.itemid AND aggamc.storeid = s.id AND aggamc.dataelement='AMC'
LEFT JOIN aggregator aggmos ON i.id = aggmos.itemid AND aggmos.storeid = s.id AND aggmos.dataelement='currentMOS'
LEFT JOIN pos ON pos.store_id = s.id AND pos.item_id = i.id AND pos.ranking = 1
WHERE aggmos.value IS NOT NULL -- Don't show items that don't have a mos calculated, this might be an issue?
AND aggmos.value < 3.0
AND s.name = '{{store_name}}'
AND i.id in (select item_id from list_master ml JOIN list_master_line mll ON ml.id = mll.item_master_ID WHERE ml.description ='{{master_list_name}}')
GROUP BY 1,2,3,4,5

```

### Step 2 — Add the Template

#### Body Template

```
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Critical Items Report for {{ store_name }}</title>
</head>
<body style="font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5;">
<div style="max-width: 800px; margin: 0 auto; background-color: white; padding: 20px;">
<h2 style="color: #333; margin-top: 0; text-align: center;">Critical Items Availability Report</h2>
{% if critical_stock and critical_stock | length > 0 %}
<table style="width: 100%; border-collapse: collapse; margin: 20px 0;">
<tr style="background-color: #f0f0f0;">
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Store Name</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Item Code</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Item Name</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Units</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">AMC (12 months)</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">SOH</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">MOS Remaining</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">PO Placed</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">PO Number</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Stock on Order</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Delivery Date</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Supplier</th>
</tr>
{% for item in critical_stock %}
<tr>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["store_name"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["item_code"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["item_name"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["units"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["amc"] | default(value=0) | round(precision=2) }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["soh"] | default(value=0) | round }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["mos"] | default(value=0) | round(precision=2) }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["latest_po_date"] | default(value="No") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["po_number"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["stock_on_order"] | default(value=0) | round }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["delivery_date"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["supplier"] | default(value="") }}</td>
</tr>
{% endfor %}
</table>
{% else %}
<p style="text-align: center; color: #666;">No critical items are low on stock.</p>
{% endif %}
</div>
</body>
</html>
```

#### Parameters

| Key                | Requirements                                                                                  |
| ------------------ | --------------------------------------------------------------------------------------------- |
| `master_list_name` | Must match a master list name in mSupply, if it's re-named the notification will stop working |
| `store_name`       | Which store you want to send notifications for, if it's renamed the notification will stop.   |

### Behaviour

- The notification runs once per week at the scheduled time.
- If the data query returns **zero rows**, no notification is sent (assuming the template checks `{% if results %}` — adjust your template accordingly if you want an explicit "all clear" message).
- If the server was offline during the scheduled time, it will run once on next startup.

---

## 2. Daily Stock Outs

### Purpose

Sent every day, this notification lists all stock items that are completely out of stock (quantity on hand = 0). It is intended for operations staff who need to action replenishment urgently.

### Step 1 — Create the Notification Quey

#### Data Query — `stock_out_today`

```sql
 with pos AS(
  SELECT
    po.id,
    po.store_id,
    pol.item_id,
    po.confirm_date,
    po.serial_number,
    n.name AS supplier_name,
    pol.delivery_date_expected,
    RANK() OVER (PARTITION BY po.store_id, pol.item_id ORDER BY po.confirm_date DESC) AS ranking,
    pol.quan_adjusted_order-pol.quan_rec_to_date AS po_outstanding
  FROM purchase_order po
  JOIN purchase_order_line pol ON po.id = pol.purchase_order_id
  JOIN name n ON po.name_id = n.id
  WHERE po.status IN ('cn')
  AND pol.quan_adjusted_order-pol.quan_rec_to_date > 0
),
prev_soh AS (
SELECT agg.storeid, agg.itemid, max(value) AS soh
FROM aggregator agg
WHERE dataElement = 'stockHistory'
AND agg.itemid in (select item_id from list_master ml JOIN list_master_line mll ON ml.id = mll.item_master_ID WHERE ml.description ='{{ master_list_name }}')
AND agg.storeid in (select id from store WHERE name = '{{ store_name }}')
AND
fulldate = (SELECT max(fulldate) FROM aggregator agg1
			WHERE agg1.dataElement = 'stockHistory'
			AND fulldate < CURRENT_DATE
			AND agg1.itemid = agg.itemid
			AND agg1.storeid = agg.storeid)
GROUP BY agg.storeid, agg.itemid
),
current_soh AS (
SELECT il.item_id, il.store_id, COALESCE(SUM(il.quantity * il.pack_size), 0) AS soh
FROM item_line il
WHERE (il.expiry_date IS NULL OR il.expiry_date >= current_date)
GROUP BY il.item_id, il.store_id
)
SELECT
s.code,
s.name as store_name,
i.code as item_code,
i.item_name,
u.units,
max(aggamc.value) as amc,
COALESCE(max(current_soh.soh), 0) AS soh,
max(aggmos.value) as mos,
max(pos.confirm_date) as latest_po_date,
max(pos.serial_number) as po_number,
max(pos.po_outstanding) as stock_on_order,
max(pos.delivery_date_expected) as delivery_date,
max(pos.supplier_name) as supplier,
max(prev_soh.soh) as prev_soh
FROM
store s
CROSS JOIN item i
LEFT JOIN unit u ON i.unit_ID = u.id
LEFT JOIN current_soh ON current_soh.item_id = i.id AND current_soh.store_id = s.id
LEFT JOIN aggregator aggamc ON i.id = aggamc.itemid AND aggamc.storeid = s.id AND aggamc.dataelement='AMC'
LEFT JOIN aggregator aggmos ON i.id = aggmos.itemid AND aggmos.storeid = s.id AND aggmos.dataelement='currentMOS'
LEFT JOIN pos ON pos.store_id = s.id AND pos.item_id = i.id AND pos.ranking = 1
LEFT JOIN prev_soh ON prev_soh.storeid = s.id AND prev_soh.itemid = i.id
WHERE
s.name = '{{ store_name }}'
AND i.id in (select item_id from list_master ml JOIN list_master_line mll ON ml.id = mll.item_master_ID WHERE ml.description ='{{ master_list_name }}')
GROUP BY 1,2,3,4,5
HAVING COALESCE(max(current_soh.soh), 0) = 0

```

### Step 2 — Add the Template

#### Body Template

```
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
</head>
<body style="font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5;">
<div style="max-width: 800px; margin: 0 auto; background-color: white; padding: 20px;">
<h2 style="color: #333; margin-top: 0; text-align: center;">Daily Out of Stock Notifications</h2>
<table style="width: 100%; border-collapse: collapse; margin: 20px 0;">
<tr style="background-color: #f0f0f0;">
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Store Name</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Item Code</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: left;">Item Name</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Units</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">AMC (12 months)</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">SOH</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">MOS Remaining</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">PO Placed</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">PO Number</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Stock on Order</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Delivery Date</th>
<th style="border: 1px solid #ccc; padding: 10px; text-align: center;">Supplier</th>
</tr>
{% for item in stock_out_today %}
<tr>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["store_name"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["item_code"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px;">{{ item["item_name"] }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["units"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["amc"] | default(value=0) | round(precision=2) }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["soh"] | default(value=0) | round }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center; font-weight: bold;">{{ item["mos"] | default(value=0) | round(precision=2) }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["latest_po_date"] | default(value="No") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["po_number"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["stock_on_order"] | default(value=0) | round }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["delivery_date"] | default(value="") }}</td>
<td style="border: 1px solid #ccc; padding: 10px; text-align: center;">{{ item["supplier"] | default(value="") }}</td>
</tr>
{% endfor %}
</table>
</div>
</body>
</html>
```

> NOTE: MAKE SURE YOU MARK THIS QUERY AS `required` So you don't get notfications where there are no out of stock items.

#### Parameters

| Key                | Requirements                                                                                  |
| ------------------ | --------------------------------------------------------------------------------------------- |
| `master_list_name` | Must match a master list name in mSupply, if it's re-named the notification will stop working |
| `store_name`       | Which store you want to send notifications for, if it's renamed the notification will stop.   |

### Behaviour

- The notification runs once per day at the scheduled time.
- If there are no stock-outs, no notification is sent
- If the server was offline, it runs once on next startup — it does **not** catch up on every missed day.
- Not this relies on aggregator data from OG mSupply - if that isn't be generated this won't work.

---

## Related Documentation

- [Notification Setup](notification_setup.md) — datasource connection and query syntax
- [Schema](schema.md) — database entity reference
- [FAQ](faq.md) — scheduling edge cases (end-of-month, missed runs)
