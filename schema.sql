CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    limit_percentage REAL NOT NULL,
    overflow_to_id INTEGER,
    FOREIGN KEY(overflow_to_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS funds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    amount TEXT NOT NULL, -- Stored as string for decimal precision/money
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    remaining_balance_rolled TEXT NOT NULL DEFAULT '0'
);

CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    amount TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    overflow_from_id INTEGER,
    FOREIGN KEY(category_id) REFERENCES categories(id),
    FOREIGN KEY(overflow_from_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS category_balances (
    category_id INTEGER PRIMARY KEY,
    available TEXT NOT NULL DEFAULT '0',
    allocated TEXT NOT NULL DEFAULT '0',
    spent TEXT NOT NULL DEFAULT '0',
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(category_id) REFERENCES categories(id)
);

-- Initial seed data will be handled in Rust code to ensure ID consistency
