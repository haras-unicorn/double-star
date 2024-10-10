DEFINE TABLE OVERWRITE posted_in SCHEMAFULL TYPE RELATION FROM message TO chat;

DEFINE EVENT OVERWRITE posted_in ON TABLE posted_in WHEN $event == "CREATE" THEN (
  UPDATE $this.out SET last_interaction = time::now()
);
