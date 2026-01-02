-- scripts/init_db.sql
CREATE TABLE IF NOT EXISTS platon_commands (
    internal_id INTEGER PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT
);

INSERT INTO platon_commands (internal_id, name, description) VALUES 
(0, 'addResult', 'Exporta un valor al resultado final'),
(1, 'log', 'Registra un mensaje en los logs')
ON CONFLICT DO NOTHING;