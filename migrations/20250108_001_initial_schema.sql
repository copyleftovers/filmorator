-- Initial schema for filmorator

CREATE TABLE photos (
    id UUID PRIMARY KEY,
    filename TEXT NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,
    file_hash TEXT NOT NULL UNIQUE,
    position INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE matchups (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    photo_indices INT[] NOT NULL,
    is_seed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE comparison_results (
    id UUID PRIMARY KEY,
    matchup_id UUID NOT NULL REFERENCES matchups(id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    ranked_photo_indices INT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE photo_ratings (
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    photo_idx INT NOT NULL,
    strength FLOAT8 NOT NULL DEFAULT 0.0,
    uncertainty FLOAT8 NOT NULL DEFAULT 1.0,
    PRIMARY KEY (session_id, photo_idx)
);

CREATE INDEX idx_matchups_session ON matchups(session_id);
CREATE INDEX idx_comparison_results_session ON comparison_results(session_id);
CREATE INDEX idx_photo_ratings_session ON photo_ratings(session_id);
