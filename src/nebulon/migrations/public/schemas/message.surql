DEFINE TABLE OVERWRITE message SCHEMAFULL;

DEFINE FIELD OVERWRITE id ON message TYPE string DEFAULT rand::ulid();
DEFINE FIELD OVERWRITE timestamp ON message TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE content ON message TYPE string;
DEFINE FIELD OVERWRITE sender ON message TYPE string;

DEFINE ANALYZER OVERWRITE message_content_analyzer TOKENIZERS class FILTERS snowball(english);

DEFINE INDEX OVERWRITE message_timestamp ON message FIELDS timestamp;
DEFINE INDEX OVERWRITE message_content ON message FIELDS content SEARCH ANALYZER message_content_analyzer BM25 HIGHLIGHTS;
