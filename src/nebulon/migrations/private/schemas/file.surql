DEFINE TABLE OVERWRITE file SCHEMAFULL;

DEFINE FIELD OVERWRITE id ON file TYPE string DEFAULT rand::ulid();
DEFINE FIELD OVERWRITE timestamp ON file TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE data ON file TYPE bytes;
DEFINE FIELD OVERWRITE title ON file TYPE string;
DEFINE FIELD OVERWRITE extension ON file TYPE string;
DEFINE FIELD OVERWRITE description ON file TYPE string;
DEFINE FIELD OVERWRITE embedding ON file TYPE array<float>;

DEFINE ANALYZER OVERWRITE file_title_description_analyzer TOKENIZERS class FILTERS snowball(english);

DEFINE INDEX OVERWRITE file_extension_timestamp ON file FIELDS extension, timestamp;
DEFINE INDEX OVERWRITE file_timestamp ON file FIELDS timestamp;
DEFINE INDEX OVERWRITE file_title_description ON file FIELDS title, description SEARCH ANALYZER file_title_description_analyzer BM25 HIGHLIGHTS;
DEFINE INDEX OVERWRITE file_embedding ON file FIELDS embedding HNSW DIMENSION 4 DIST EUCLIDEAN EFC 150 M 12;

DEFINE TABLE OVERWRITE attached_to TYPE RELATION FROM file TO memo;
