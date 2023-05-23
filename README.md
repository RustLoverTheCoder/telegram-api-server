# TelegramApiServer

## Features

* Fast async rust Http Server
* Websocket endpoints for events and logs
* https://core.telegram.org/methods

#### Init data
- init database

	```
    sea-orm-cli generate entity --with-serde both -o entity/src/entities
	```