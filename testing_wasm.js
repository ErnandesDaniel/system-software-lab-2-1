const fs = require('fs');
const path = require('path');

const wasmPath = path.join(__dirname, 'output', 'program.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);

const memory = new WebAssembly.Memory({ initial: 1 });

const imports = {
    env: {
        memory: memory,
        __stack_pointer: 65600,
    }
};

const instance = new WebAssembly.Instance(
    new WebAssembly.Module(wasmBuffer),
    imports
);

const args = process.argv.slice(2);
const func = args[0];
const params = args.slice(1).map(Number);

if (!func) {
    console.log(`Usage: node testing_wasm.js <function> [args...]
    
Available functions:
  factorial n      - факториал n
  power base exp  - base в степени exp
  sum a b         - a + b
  diff a b        - a - b
  product a b     - a * b`);
    process.exit(1);
}

const fn = instance.exports[func];
if (!fn) {
    console.error(`Unknown function: ${func}`);
    process.exit(1);
}

const result = fn(...params);
console.log(result);