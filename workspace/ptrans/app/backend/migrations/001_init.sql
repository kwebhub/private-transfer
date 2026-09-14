-- Таблица commitments: все листья Merkle Tree
CREATE TABLE IF NOT EXISTS commitments (
    id              BIGSERIAL PRIMARY KEY,
    leaf_index      BIGINT NOT NULL UNIQUE,
    commitment      BYTEA NOT NULL CHECK (octet_length(commitment) = 32),
    pool_address    TEXT NOT NULL,
    tx_signature    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_commitments_leaf_index ON commitments(leaf_index);
CREATE INDEX IF NOT EXISTS idx_commitments_pool ON commitments(pool_address);

-- Таблица roots: история корней Merkle Tree
CREATE TABLE IF NOT EXISTS roots (
    id              BIGSERIAL PRIMARY KEY,
    root            BYTEA NOT NULL CHECK (octet_length(root) = 32),
    leaf_index      BIGINT NOT NULL,
    pool_address    TEXT NOT NULL,
    tx_signature    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_roots_root ON roots(root);
CREATE INDEX IF NOT EXISTS idx_roots_pool ON roots(pool_address);

-- Таблица nullifiers: использованные нуллификаторы
CREATE TABLE IF NOT EXISTS nullifiers (
    nullifier_hash  BYTEA PRIMARY KEY CHECK (octet_length(nullifier_hash) = 32),
    pool_address    TEXT NOT NULL,
    recipient       TEXT,
    amount          BIGINT,
    tx_signature    TEXT,
    used_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nullifiers_pool ON nullifiers(pool_address);
