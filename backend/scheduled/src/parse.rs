use chrono::{DateTime, Days, Months, Utc};
use serde::{Deserialize, Serialize};

use crate::NotificationError;

/* Example config:
{
    "bodyTemplate": "Some Template",
    "configurationData": "{}",
    "id": "fc2a0e0c-440a-4abc-a13b-6b1270a59ff0",
    "kind": "SCHEDULED",
    "notificationQueryIds": [
        "72e5342f-08b7-499b-8a51-339ae142f68a",
        "98533e08-99bb-4b18-a045-db67e1852d73",
        "aca03f15-96a1-4f36-bd80-1fd34331a5f1"
    ],
    "parameters": "{\"email_address\":\"test@example.com\",\"project\":\"prj1\",\"province\":\"prov1\"}",
    "parsedParameters": {
        "email_address": "test@example.com",
        "project": "prj1",
        "province": "prov1"
    },
    "recipientIds": [],
    "recipientListIds": [],
    "requiredParameters": [
        "email_address",
        "project",
        "province"
    ],
    "scheduleFrequency": "daily",
    "scheduleStartTime": "2023-10-11T02:09:31.221Z",
    "sqlRecipientListIds": [
        "3f6194ad-1fbb-494b-8ffb-c0f2e1b455d0"
    ],
    "status": "DISABLED",
    "subjectTemplate": "Title Template",
    "title": "Some Notification Name"
}
*/

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledNotificationPluginConfig {
    pub id: String,
    pub title: String,
    pub body_template: String,
    pub subject_template: String,
    pub schedule_frequency: String,
    pub schedule_start_time: DateTime<Utc>,
    #[serde(default)]
    pub notification_query_ids: Vec<String>,
}

impl ScheduledNotificationPluginConfig {
    pub fn from_string(json_string: &str) -> Result<Self, NotificationError> {
        let config: ScheduledNotificationPluginConfig = serde_json::from_str(json_string)
            .map_err(|e| NotificationError::UnableToParseConfig(format!("{:?}", e)))?;

        Ok(config)
    }

    pub fn next_due_date(
        &self,
        now_utc: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, NotificationError> {
        // Take the schedule_start_time and add the schedule_frequency to it
        // First map the schedule_frequency to a chrono::Duration if the schedule_frequency is 'Days' or 'Weeks' or 'Months'
        // Then add the duration to the schedule_start_time until we're past now

        match self.schedule_frequency.as_str() {
            "weekly" => {
                let mut next_due_date = self.schedule_start_time;
                while next_due_date < now_utc {
                    let option = next_due_date.checked_add_days(Days::new(7));
                    next_due_date = match option {
                        Some(d) => d,
                        None => return Err(NotificationError::InvalidNextDueDate),
                    };
                }
                return Ok(next_due_date);
            }
            "daily" => {
                let mut next_due_date = self.schedule_start_time;
                while next_due_date < now_utc {
                    let option = next_due_date.checked_add_days(Days::new(1));
                    next_due_date = match option {
                        Some(d) => d,
                        None => return Err(NotificationError::InvalidNextDueDate),
                    };
                }
                return Ok(next_due_date);
            }
            "monthly" => {
                let mut look_forward = 1;
                let mut next_due_date = self.schedule_start_time;
                while next_due_date < now_utc {
                    // Note: chrono automatically handles leap years and returns last day of the month if day isn't valid for that month
                    // https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDateTime.html#method.checked_add_months
                    let option = self.schedule_start_time.checked_add_months(Months::new(look_forward));
                    next_due_date = match option {
                        Some(d) => d,
                        None => return Err(NotificationError::InvalidNextDueDate),
                    };
                    look_forward += 1;
                }
                return Ok(next_due_date);
            }
            _ => {
                return Err(NotificationError::InvalidNextDueDate);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_broken_parse_config() {
        let result = ScheduledNotificationPluginConfig::from_string("{sdf}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_config() {
        let example1 = r#"{
    "bodyTemplate": "Project {{ parameters.project }}\nProvince: {{ parameters.province }} \n\nOf the {{store_query_result.count}} stores configured:\n29 (100%) have synced at least once\n29 (100%) have conducted their first stocktake\n4 (13.8%) have synced within the last 30 days\n1 (3.4%) have synced within the last 7 days\n\nOf the 4 stores that have synced within the last 30 days:\n7 (175%) have conducted a stocktake in the last 60 days\n7 (175%) have placed an internal order in the last 60 days\n0 (0%) have created a CI in the last 7 days\n3 (75%) have SIs with age < 14 days old\n\nFor details: https://png.msupply.org:3000/d/NStwzwqMk/item-stock-and-sync-stats?orgId=1&var-masterList=Without%20master%20lists&var-itemCategoryLevel2=All&var-itemCategory=All&var-multiItem=All&var-nameCategory1Level2={{parameters.province}}&var-nameCategory2=All&var-nameCategory3={{parameters.project}}&var-multiFacility=All&var-sites=All&var-itemCategory3=All&var-venCategory=All&var-stockType=All&var-singleItem=--%20Select%20a%20single%20item%20--&var-nullString=NONE&var-storeTableName=store_categories&var-itemTableName=item_categories&var-facilityCount=29&var-itemCount=6204&var-customer=All&var-user_name=Sussoldash&var-LoggedUser=Sussoldash&var-store_configured=29&var-store_synced=4",
    "id": "fcb1ead6-f58b-4dd2-985d-7058e831d360",
    "kind": "SCHEDULED",
    "parameters": "{}",
    "parsedParameters": {},
    "recipientIds": [
        "d9589fb1-7a29-4d56-b55b-4eb7e870064a"
    ],
    "recipientListIds": [],
    "scheduleFrequency": "weekly",
    "scheduleStartTime": "2023-08-29T12:00:00.000Z",
    "sqlQueries": [],
    "sqlRecipientListIds": [
        "765eafdd-62cc-4366-97f8-91146797cddd"
    ],
    "subjectTemplate": "Project {{ parameters.project }}, Province: {{ parameters.province }} Store Status Report",
    "title": "Test Report"
}"#;

        let result = ScheduledNotificationPluginConfig::from_string(example1);
        assert!(result.is_ok());

        let config = result.unwrap();

        assert_eq!("weekly", config.schedule_frequency);
        assert_eq!(
            Utc.with_ymd_and_hms(2023, 08, 29, 12, 0, 0).unwrap(),
            config.schedule_start_time
        );
        assert!(config.body_template.contains("{{ parameters.project }}"));
        assert!(config.subject_template.contains("{{ parameters.project }}"));

        // TODO: add tests for sqlQueries, recipientIds, recipientListIds, sqlRecipientListIds?
    }

    #[test]
    fn test_parse_config_daily() {
        let config = ScheduledNotificationPluginConfig {
            body_template: "".to_string(),
            subject_template: "".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap(),
            ..Default::default()
        };

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 0, 0, 0).unwrap();

        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap()
        );

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap()
        );

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 1).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 30, 7, 0, 0).unwrap()
        );

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 31, 16, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 09, 1, 7, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_parse_config_weekly() {
        let config = ScheduledNotificationPluginConfig {
            body_template: "".to_string(),
            subject_template: "".to_string(),
            schedule_frequency: "weekly".to_string(),
            schedule_start_time: Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap(),
            ..Default::default()
        };

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap()
        );

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 1).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 09, 05, 7, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_parse_config_monthly() {
        let config = ScheduledNotificationPluginConfig {
            body_template: "".to_string(),
            subject_template: "".to_string(),
            schedule_frequency: "monthly".to_string(),
            schedule_start_time: Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap(),
            ..Default::default()
        };

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap()
        );

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 1).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 09, 29, 7, 0, 0).unwrap()
        );

        // last day of the month Leap year, but hasn't reached first iteration yet.
        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 02, 27, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2023, 08, 29, 7, 0, 0).unwrap()
        );

        // last day of the month Leap year
        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 02, 27, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2024, 02, 29, 7, 0, 0).unwrap()
        );

        // last day of the month no Leap year
        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 02, 27, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2025, 02, 28, 7, 0, 0).unwrap()
        );

        // Running through a short month
        let config = ScheduledNotificationPluginConfig {
            body_template: "".to_string(),
            subject_template: "".to_string(),
            schedule_frequency: "monthly".to_string(),
            schedule_start_time: Utc.with_ymd_and_hms(2024, 01, 31, 7, 0, 0).unwrap(),
            ..Default::default()
        };

        let now_utc: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 03, 10, 0, 0, 0).unwrap();
        let next_due_date = config.next_due_date(now_utc);
        assert!(next_due_date.is_ok());
        assert_eq!(
            next_due_date.unwrap(),
            Utc.with_ymd_and_hms(2024, 03, 31, 7, 0, 0).unwrap()
        );
    }
}
