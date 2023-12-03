CREATE TABLE pages(
page_id uuid NOT NULL,
PRIMARY KEY (page_id),
hits integer NOT NULL DEFAULT 0,
owner uuid NOT NULL
);
