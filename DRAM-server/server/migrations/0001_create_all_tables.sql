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
    user_key     VARCHAR(255) NOT NULL REFERENCES users(user_key),
    session_key  VARCHAR(255) NOT NULL REFERENCES sessions(session_key),
    user_role       VARCHAR(30) CHECK (user_role IN ('owner', 'member')),
    PRIMARY KEY (user_key, session_key)
);

CREATE UNIQUE INDEX one_owner_per_session 
ON user_session (session_key) 
WHERE user_role = 'owner';

CREATE TABLE IF NOT EXISTS blacklist (
    user_key     VARCHAR(255) NOT NULL REFERENCES users(user_key),
    session_key  VARCHAR(255) NOT NULL REFERENCES sessions(session_key),
    PRIMARY KEY (user_key, session_key)
);