DEFINE TABLE OVERWRITE chat SCHEMAFULL;

DEFINE FIELD OVERWRITE id ON chat TYPE string DEFAULT rand::ulid();
DEFINE FIELD OVERWRITE timestamp ON chat TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE last_interaction ON chat TYPE option<datetime>;

DEFINE INDEX OVERWRITE chat_timestamp ON chat FIELDS timestamp;
DEFINE INDEX OVERWRITE chat_last_interaction ON chat FIELDS last_interaction;
