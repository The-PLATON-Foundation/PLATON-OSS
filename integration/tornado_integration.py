import tornado.web
import json
from avap_core import VM, Compiler, ExecutionContext

class ExecuteHandler(tornado.web.RequestHandler):
    async def post(self):
        data = json.loads(self.request.body)
        
        # Compilar script
        compiler = Compiler()
        # ... lógica de compilación ...
        bytecode = compiler.finalize()
        
        # Ejecutar
        vm = VM()
        vm.load(bytecode)
        
        # Variables iniciales
        initial_vars = data.get('variables', {})
        result = vm.execute(initial_vars)
        
        # Responder
        self.write({
            'success': True,
            'result': result.to_python(),
            'variables': {k: v.to_python() for k, v in vm.globals.items()}
        })