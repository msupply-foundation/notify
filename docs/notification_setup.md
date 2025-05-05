## Datasource Configuration

Notify assumes you have a postresql database to connect too.
This is known as the `datasource` in the `backend` configuration.

To configure your datasource connection, update the `local.yaml` file in the configuration directory with connection details to your database.
```
datasource:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "password"
  database_name: "dashboard"
```
We recommend using readonly credentials for production environments.

## Creating SQL Queries

Notify uses SQL queries to retrieve data from your datasource.
There are two types of queries
1. Recipient Queries
Allows you to customise who receives a notification based on an SQL Query against your datasource.
2. Data Queries
Allows you to customise the data that is sent in a notification based on an SQL Query against your datasource.

Sql Queries are parameterised using [tera](https://keats.github.io/tera/docs/).

For example, if you wanted to send a notification to all users who's accounts are part of a group you could use the following query:
```
SELECT * FROM users WHERE group = {{ group }}
```

By specifiying `{{ group }}` in the query, you can pass in a value for `group` when you send a notification.
Any sets of double curly braces `{{ }}` will be replaced with the value you pass in, and by adding double curly braces to your query, notify will assume that this parameter is required.

Note: Parameters are always assumed to be strings, if you want to use a parameter as a number, you'll need to cast it to a number in your query.

Something like this...
```sql
SELECT * FROM users WHERE group = {{ group }} LIMIT CAST ('{{ limit }}' AS INTEGER)
```

[https://keats.github.io/tera/docs/](https://keats.github.io/tera/docs/)

## Telegram Bot
To configure telegram, you need to create a bot and get a token.

To create a bot open a telegram chat to @BotFather
Create a new bot with the command `/newbot`

BotFather will ask you for a username for the bot, call it something like `notify_YOURNAME_bot`

BotFather will then give you a token, copy this token and add it to your local.yaml configuration file.

```
telegram:
  token: "ABIGLONGSTRINGOFRANDOMCHARACTERSHERE"
```

To use your new bot within telegram groups, you may need to disable privacy mode using the command `/setprivacy` in a chat with @BotFather.
See: https://core.telegram.org/bots#6-botfather and https://core.telegram.org/bots/features#privacy-mode

You'll also need to click on the link to the new bot chat provided by BotFather and click on Start to initialise it, as Notify may not recognise it otherwise.

## Setting up telegram recipients
To add a telegram chat to your notify server, all you need to do is chat with the bot you created earlier, or add it to a group chat.

> Note: You can also ask the bot for the chatid from telegram by sending the command `/chatid` to the bot. Which can be useful when configuring other tools.
