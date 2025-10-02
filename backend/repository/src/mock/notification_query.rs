use crate::NotificationQueryRow;

pub fn mock_notification_queries() -> Vec<NotificationQueryRow> {
    vec![
        mock_notification_query_with_params(),
        mock_notification_query_with_no_param_2_rows(),
        mock_notification_query_with_no_rows(),
    ]
}

pub fn mock_notification_query_with_params() -> NotificationQueryRow {
    let sql_query = r#"SELECT {{ latest_temperature }} as latest_temperature, {{ sensor_limit }} as sensor_limit, 
CASE WHEN {{ latest_temperature }} > {{ sensor_limit }} THEN TRUE ELSE FALSE END as is_above_limit
"#;
    NotificationQueryRow {
        id: String::from("id_notification_query_with_params"),
        name: String::from("notification_query_with_params"),
        description: String::from("The accepts 2 params, sensor_limit (number) and latest_temperature (number). It returns a single row with temperature (number), sensor_limit (number), and is_above_limit (boolean). is_above_limit is true if latest_temperature is greater than sensor_limit"),
        query: sql_query.to_string(),
        required_parameters: "[\"sensor_limit\", \"latest_temperature\"]".to_string(),
        reference_name: String::from("query1"),
        ..Default::default()
    }
}

pub fn mock_notification_query_with_no_param_2_rows() -> NotificationQueryRow {
    let sql_query = r#"SELECT 1.25 as latest_temperature, 'sensor1' as sensor_name, -10 as sensor_limit UNION
SELECT 1.51 as latest_temperature, 'sensor2' as sensor_name, -10 as sensor_limit"#;
    NotificationQueryRow {
        id: String::from("id_notification_query_no_param"),
        name: String::from("notification_query_no_param"),
        description: String::from("Returns 2 rows, each with temperature (number), sensor_name (string), and sensor_limit (number)"),
        query: sql_query.to_string(),
        required_parameters: "[]".to_string(),
        reference_name: String::from("query2"),
        ..Default::default()
    }
}

pub fn mock_notification_query_with_no_rows() -> NotificationQueryRow {
    let sql_query = r#"SELECT 1 as value WHERE 1 = 0"#;
    NotificationQueryRow {
        id: String::from("id_notification_query_no_rows"),
        name: String::from("notification_query_no_rows"),
        description: String::from("Returns no rows - for testing required query behavior"),
        query: sql_query.to_string(),
        required_parameters: "[]".to_string(),
        reference_name: String::from("query_no_rows"),
        ..Default::default()
    }
}
