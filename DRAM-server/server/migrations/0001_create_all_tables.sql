CREATE TABLE IF NOT EXISTS users (
    user_key       VARCHAR(255) NOT NULL,
    nickname       VARCHAR(100),
    last_time_seen TIMESTAMP,
    PRIMARY KEY (user_key)
);

CREATE TABLE IF NOT EXISTS sessions (
    session_key  VARCHAR(255) NOT NULL,
    session_name VARCHAR(100),
    PRIMARY KEY (session_key)
);

CREATE TABLE IF NOT EXISTS user_session (
    id_user_session SERIAL PRIMARY KEY,
    fk_user_key     VARCHAR(255) NOT NULL,
    fk_session_key  VARCHAR(255) NOT NULL,
    user_role       VARCHAR(30) CHECK (user_role IN ('owner', 'member')),
    CONSTRAINT contains FOREIGN KEY (fk_user_key)    REFERENCES users (user_key),
    CONSTRAINT relates  FOREIGN KEY (fk_session_key) REFERENCES sessions (session_key)
);

CREATE TABLE IF NOT EXISTS blacklist (
    id_blacklist   SERIAL PRIMARY KEY,
    fk_user_key    VARCHAR(255) NOT NULL,
    fk_session_key VARCHAR(255) NOT NULL,
    CONSTRAINT is_banned FOREIGN KEY (fk_user_key)    REFERENCES users (user_key),
    CONSTRAINT banned_in FOREIGN KEY (fk_session_key) REFERENCES sessions (session_key)
);