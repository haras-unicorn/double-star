DEFINE TABLE OVERWRITE memo SCHEMAFULL;

DEFINE FIELD OVERWRITE id ON memo TYPE string DEFAULT rand::ulid();
DEFINE FIELD OVERWRITE timestamp ON memo TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE content ON memo TYPE string;

DEFINE ANALYZER OVERWRITE memo_content_analyzer TOKENIZERS class FILTERS snowball(english);

DEFINE INDEX OVERWRITE memo_content ON memo FIELDS content SEARCH ANALYZER memo_content_analyzer BM25 HIGHLIGHTS;
