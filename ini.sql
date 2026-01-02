CREATE TABLE IF NOT EXISTS platon_commands (
    internal_id INTEGER PRIMARY KEY, -- El ID que usará el Bytecode (0x60, 0x61...)
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT
);

-- Insertamos los comandos iniciales
INSERT INTO platon_commands (internal_id, name, description) VALUES 
(0, 'addResult', 'Exporta un valor al resultado final del script'),
(1, 'log', 'Registra un evento en los logs de ejecución');
-- Nota: addVar no necesita estar aquí porque es una instrucción nativa (Opcode)