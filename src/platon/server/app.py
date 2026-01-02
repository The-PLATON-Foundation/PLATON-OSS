import tornado.ioloop
import tornado.web
import json
import asyncpg
import os
from platon.vm import VM
from platon.compiler import Compiler
from platon.context import ExecutionContext
from avap.registry import AvapRegistry

class ExecuteHandler(tornado.web.RequestHandler):
    def initialize(self, registry):
        self.registry = registry
        self.compiler = Compiler()

    async def post(self):
        try:
            data = json.loads(self.request.body)
            ast = data.get("ast", [])
            initial_vars = data.get("variables", {})

            context = ExecutionContext(initial_vars=initial_vars)

            bytecode = self.compiler.compile_ast(ast)

            vm = VM(timeout=2.0, max_instr=50000)
            vm.load(bytecode)
            
            vm.execute(registry=self.registry, context=context)

            self.write(context.to_dict())

        except Exception as e:
            self.set_status(500)
            self.write({
                "success": False, 
                "error": f"Execution failed: {str(e)}"
            })

async def make_app():
    db_url = os.getenv("DATABASE_URL", "postgresql://postgres:password@localhost:5432/platon_db")
    pool = await asyncpg.create_pool(db_url)
    
    registry = AvapRegistry(pool)
    await registry.load_commands()
    
    return tornado.web.Application([
        (r"/execute", ExecuteHandler, dict(registry=registry)),
    ])

if __name__ == "__main__":
    app = tornado.ioloop.IOLoop.current().run_sync(make_app)
    app.listen(8888)
    print("AVAP Language Server running on http://localhost:8888")
    tornado.ioloop.IOLoop.current().start()