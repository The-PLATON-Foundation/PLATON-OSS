import asyncio
import asyncpg
import os

async def init():
    # GitHub Actions usa estos datos por defecto según tu ci.yml
    dsn = os.getenv("DB_URL", "postgresql://postgres:password@localhost:5432/platon_db")
    conn = await asyncpg.connect(dsn)
    
    # Crear tabla de comandos
    await conn.execute('''
        CREATE TABLE IF NOT EXISTS platon_commands (
            internal_id INTEGER PRIMARY KEY,
            name VARCHAR(50) UNIQUE NOT NULL,
            description TEXT
        );
    ''')
    
    # Insertar comandos básicos para que los tests no fallen
    await conn.execute('''
        INSERT INTO platon_commands (internal_id, name, description) 
        VALUES (0, 'addResult', 'Export result to context')
        ON CONFLICT DO NOTHING;
    ''')
    await conn.close()
    print("✅ Database initialized for CI")

if __name__ == "__main__":
    asyncio.run(init())