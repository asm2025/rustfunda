CREATE TABLE IF NOT EXISTS timeseries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collector_id TEXT,
    received BIGINT,
    total_memory BIGINT,
    used_memory BIGINT,
    cpus INTEGER,
    cpu_usage REAL,
    avg_cpu_usage REAL
);