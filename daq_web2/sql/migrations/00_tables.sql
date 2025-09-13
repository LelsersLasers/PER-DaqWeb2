
-- Uploads/Signals -------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Uploads (
    id				INTEGER PRIMARY KEY,
	upload_name		TEXT NOT NULL,
	commit_hash		TEXT NOT NULL,
	start_time		TEXT NOT NULL, -- YYYY-MM-DD HH:MM:SS.SSS
	upload_time		TEXT NOT NULL, -- YYYY-MM-DD HH:MM:SS.SSS
	upload_status	TEXT NOT NULL CHECK (type IN ('in_progress', 'completed', 'failed'))
);

CREATE TABLE IF NOT EXISTS Logs (
	id 		 	INTEGER PRIMARY KEY,
	file_name	TEXT NOT NULL,
	upload_id	INTEGER NOT NULL,

	FOREIGN KEY (upload_id) REFERENCES Uploads(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Messages (
	id 				INTEGER PRIMARY KEY,
	log_id			INTEGER NOT NULL,
	msg_id			INTEGER NOT NULL,
	msg_name		TEXT NOT NULL,
	timestamp_raw	INTEGER NOT NULL,
	timestamp_adj   ?? NOT NULL,
	msg_desc		TEXT,

	FOREIGN KEY (log_id) REFERENCES Logs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Signals (
	id 				INTEGER PRIMARY KEY,
	msg_id			INTEGER NOT NULL,
	signal_name		TEXT NOT NULL,
	signal_value	REAL NOT NULL,
	signal_unit		TEXT,
	signal_unit		TEXT,

	FOREIGN KEY (msg_id) REFERENCES Messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS UploadTags (
	id 			INTEGER PRIMARY KEY,
	tag_name	TEXT NOT NULL,
	color		TEXT NOT NULL -- Hex color code (e.g. #rrggbb)
);

CREATE TABLE IF NOT EXISTS UploadTagPairs (
	id				INTEGER PRIMARY KEY,
	upload_id		INTEGER NOT NULL,
	upload_tag_id	INTEGER NOT NULL,

	FOREIGN KEY (upload_id) REFERENCES Uploads(id) ON DELETE CASCADE,
	FOREIGN KEY (upload_tag_id) REFERENCES UploadTags(id) ON DELETE CASCADE
);


-- PRESETS -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Presets (
	id				INTEGER PRIMARY KEY,
	preset_name		TEXT NOT NULL,
	preset_color	TEXT NOT NULL -- Hex color code (e.g. #rrggbb)
);

CREATE TABLE IF NOT EXISTS SignalPresets (
	id				INTEGER PRIMARY KEY,
	preset_id		INTEGER NOT NULL,
	msg_name 		TEXT NOT NULL,
	signal_name		TEXT NOT NULL,

	FOREIGN KEY (preset_id) REFERENCES Presets(id) ON DELETE CASCADE
);


-- POSTS -------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Posts (
	id				INTEGER PRIMARY KEY,
	post_title		TEXT NOT NULL,
	post_desc		TEXT NOT NULL,
	graph_query		?? NOT NULL,
	created_at		TEXT NOT NULL, -- YYYY-MM-DD HH:MM:SS.SSS
	updated_at		TEXT NOT NULL  -- YYYY-MM-DD HH:MM:SS.SSS
);

CREATE TABLE IF NOT EXISTS PostTags (
	id 			INTEGER PRIMARY KEY,
	tag_name	TEXT NOT NULL,
	color		TEXT NOT NULL -- Hex color code (e.g. #rrggbb)
);

CREATE TABLE IF NOT EXISTS PostTagPairs (
	id			INTEGER PRIMARY KEY,
	post_id		INTEGER NOT NULL,
	post_tag_id	INTEGER NOT NULL,

	FOREIGN KEY (post_id) REFERENCES Posts(id) ON DELETE CASCADE,
	FOREIGN KEY (post_tag_id) REFERENCES PostTags(id) ON DELETE CASCADE
);