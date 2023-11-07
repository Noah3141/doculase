CREATE TABLE query_cache (
    rid int NOT NULL AUTO_INCREMENT,
    timestamp varchar(45) NOT NULL,
    model varchar(45) NOT NULL,
    temperature float NOT NULL,
    prompt char(64) NOT NULL,
    query_key longtext NOT NULL,
    query_key_hash char(64) NOT NULL,
    prompt_tokens int NOT NULL,
    completion_tokens int NOT NULL,
    total_tokens int NOT NULL,
    process_time int NOT NULL,
    response json NOT NULL,
    cost float NOT NULL,
    PRIMARY KEY (rid),
    UNIQUE KEY query_key_hash_UNIQUE (query_key_hash)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci


/* 

sea-orm-cli generate entity -o openai_for_rs/src/models/db --with-serde both 

"CLI create files for table at Output=src/models/ and be sure to add serialize-deserialize implementations for it all"


Uses DATABASE_URL by default, use --help flag with any combination of commands to get the next layer's flags
This is Rust: extra effort goes into ENSURING a value is not accidentally null, therefore we can easily say no to the need to skip "not null", 
and we therefore also WANT to include NOT NULL, so that we aren't literally-actually uselessly checking all our data a "options", reduntantly checking
if it's null, when we already did it long long ago.
*/

