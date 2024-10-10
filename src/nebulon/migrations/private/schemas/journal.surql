DEFINE TABLE OVERWRITE journal SCHEMAFULL;

DEFINE FIELD OVERWRITE id ON journal TYPE string DEFAULT rand::ulid();
DEFINE FIELD OVERWRITE timestamp ON journal TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE content ON journal TYPE string;

DEFINE ANALYZER OVERWRITE journal_content_analyzer TOKENIZERS class FILTERS snowball(english);

DEFINE INDEX OVERWRITE journal_content ON journal FIELDS content SEARCH ANALYZER journal_content_analyzer BM25 HIGHLIGHTS;
