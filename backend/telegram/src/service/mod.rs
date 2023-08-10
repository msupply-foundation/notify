/*
   The telegram serviçe polls the telegram /getUpdates api and handle the following cases.

   A new chat_id is seen
       - Create a new recipient for the chat id
       - Send a welcome message to the chat (including the chat id for reference)

   An existing chat_id is seen (check if we need to update the chat name in the recipient)
       - No message is required

   A direct message is seen
       - Send the message with the chat id
*/

/*
Example API Polling responses

{
    "ok": true,
    "result": [
        {
            "update_id": 794348048,
            "message": {
                "message_id": 30,
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "chat": {
                    "id": -903279238,
                    "title": "User1 & bot-name",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "date": 1691531796,
                "group_chat_created": true
            }
        },
        {
            "update_id": 794348049,
            "my_chat_member": {
                "chat": {
                    "id": -903279238,
                    "title": "User1 & bot-name",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "date": 1691531796,
                "old_chat_member": {
                    "user": {
                        "id": 6544022299,
                        "is_bot": true,
                        "first_name": "bot-name",
                        "username": "bot-name"
                    },
                    "status": "left"
                },
                "new_chat_member": {
                    "user": {
                        "id": 6544022299,
                        "is_bot": true,
                        "first_name": "bot-name",
                        "username": "bot-name"
                    },
                    "status": "member"
                }
            }
        },
        {
            "update_id": 794348050,
            "message": {
                "message_id": 31,
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "chat": {
                    "id": -903279238,
                    "title": "User1 & bot-name (West)",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "date": 1691531864,
                "new_chat_title": "User1 & bot-name (West)"
            }
        },
        {
            "update_id": 794348051,
            "message": {
                "message_id": 32,
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "chat": {
                    "id": -914917543,
                    "title": "User1 & bot-name",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "date": 1691536017,
                "text": "@bot-name Can you tell me this chat_id please?",
                "entities": [
                    {
                        "offset": 0,
                        "length": 15,
                        "type": "mention"
                    }
                ]
            }
        },
        {
            "update_id": 794348052,
            "message": {
                "message_id": 33,
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "chat": {
                    "id": -914917543,
                    "title": "User1 & bot-name",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "date": 1691536034,
                "text": "This is a normal non direct message message..."
            }
        },
        {
            "update_id": 794348053,
            "message": {
                "message_id": 34,
                "from": {
                    "id": 5068627745,
                    "is_bot": false,
                    "first_name": "User1",
                    "last_name": "Last1",
                    "language_code": "en"
                },
                "chat": {
                    "id": -931768832,
                    "title": "Testing Notifications :)",
                    "type": "group",
                    "all_members_are_administrators": true
                },
                "date": 1691536201,
                "text": "User2 Just wanting to check how mentions look that aren\u2019t too the bot\u2026",
                "entities": [
                    {
                        "offset": 0,
                        "length": 5,
                        "type": "text_mention",
                        "user": {
                            "id": 1320002934,
                            "is_bot": false,
                            "first_name": "User2",
                            "last_name": "Last2"
                        }
                    }
                ]
            }
        }
    ]
}
*/

use crate::{TelegramClient, TelegramUpdate};
use log;

static TELEGRAM_POLL_TIMEOUT_SECONDS: i64 = 30;
static ERROR_SLEEP_SECONDS: u64 = 10;

pub async fn poll_get_updates(
    telegram_client: &TelegramClient,
    tx_updates: &tokio::sync::mpsc::Sender<TelegramUpdate>,
) {
    // Might be a good idea to persist this to the KV store so we can pickup from where we left off on a restart?
    let mut last_update_id: i64 = -2;

    loop {
        let updates = telegram_client
            .get_updates(last_update_id, TELEGRAM_POLL_TIMEOUT_SECONDS)
            .await;
        match updates {
            Ok(updates) => {
                let num = updates.len();
                log::debug!("Got {} updates", num);
                for update in updates {
                    // First handle the update_id, if something goes wrong with parsing the JSON, we don't want to end up re-processing in an infinite loop
                    let update_id = match update.get("update_id") {
                        Some(update_id) => update_id,
                        None => {
                            log::error!(
                                "Error update doesn't include an update_id !: {:?}",
                                update
                            );
                            // Increment update_id so we hopefully don't get this update again.
                            last_update_id += 1;
                            continue;
                        }
                    };
                    let update_id = match update_id.as_i64() {
                        Some(update_id) => update_id,
                        None => {
                            log::error!("Error parsing update_id as i64: {:?}", update);
                            last_update_id += 1;
                            continue;
                        }
                    };
                    if update_id > last_update_id {
                        last_update_id = update_id;
                    }

                    // Now try to parse the update using serde_json
                    // TODO: extract this into a function
                    let telegram_update: TelegramUpdate =
                        match serde_json::from_value(update.clone()) {
                            Ok(telegram_update) => telegram_update,
                            Err(error) => {
                                log::error!(
                                    "Error parsing update: {:?} update: {:?}",
                                    error,
                                    update
                                );
                                continue;
                            }
                        };

                    // Send the update on the channel so other processors can handle it.
                    let result = tx_updates.send(telegram_update).await;
                    match result {
                        Ok(_) => {
                            log::debug!("Sent update to tx_updates");
                        }
                        Err(error) => {
                            log::error!("Error sending message to tx_updates: {:?}", error);
                        }
                    };
                }
            }

            Err(error) => {
                log::error!("Error getting updates: {:?} \n Sleeping...", error);
                // Sleep for a bit so we don't hammer the CPU or telegram API
                tokio::time::sleep(std::time::Duration::from_secs(ERROR_SLEEP_SECONDS)).await;
            }
        };
    }
}

#[cfg(test)]
mod test {

    // TODO Need tests!

    // #[tokio::test]
    // async fn test_get_poll_updates_single_update() {
    //   let json = r#"
    //   ";

    // }
}
