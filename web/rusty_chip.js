/* tslint:disable */
import * as wasm from './rusty_chip_bg';

const __wbg_f_random_random_n_target = Math.random;

export function __wbg_f_random_random_n() {
    return __wbg_f_random_random_n_target();
}

let cachedUint8Memory = null;
function getUint8Memory() {
    if (cachedUint8Memory === null ||
        cachedUint8Memory.buffer !== wasm.memory.buffer)
        cachedUint8Memory = new Uint8Array(wasm.memory.buffer);
    return cachedUint8Memory;
}

function passArray8ToWasm(arg) {
    const ptr = wasm.__wbindgen_malloc(arg.length);
    getUint8Memory().set(arg, ptr);
    return [ptr, arg.length];
}

let cachedUint32Memory = null;
function getUint32Memory() {
    if (cachedUint32Memory === null ||
        cachedUint32Memory.buffer !== wasm.memory.buffer)
        cachedUint32Memory = new Uint32Array(wasm.memory.buffer);
    return cachedUint32Memory;
}

let cachedGlobalArgumentPtr = null;
function globalArgumentPtr() {
    if (cachedGlobalArgumentPtr === null)
        cachedGlobalArgumentPtr = wasm.__wbindgen_global_argument_ptr();
    return cachedGlobalArgumentPtr;
}

function setGlobalArgument(arg, i) {
    const idx = globalArgumentPtr() / 4 + i;
    getUint32Memory()[idx] = arg;
}

export class WasmCpu {

                static __construct(ptr) {
                    return new WasmCpu(ptr);
                }

                constructor(ptr) {
                    this.ptr = ptr;
                }

            free() {
                const ptr = this.ptr;
                this.ptr = 0;
                wasm.__wbg_wasmcpu_free(ptr);
            }
        static new() {
    return WasmCpu.__construct(wasm.wasmcpu_new());
}
load_rom(arg0) {
    const [ptr0, len0] = passArray8ToWasm(arg0);
    setGlobalArgument(len0, 0);
    try {
        return wasm.wasmcpu_load_rom(this.ptr, ptr0);
    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);
    }
}
exit() {
    return (wasm.wasmcpu_exit(this.ptr)) !== 0;
}
}

const TextDecoder = typeof self === 'object' && self.TextDecoder
    ? self.TextDecoder
    : require('util').TextDecoder;

let cachedDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedDecoder.decode(getUint8Memory().slice(ptr, ptr + len));
}

export function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

