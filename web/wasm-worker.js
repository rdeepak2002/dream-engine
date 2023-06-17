import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";

function wrapExports({start_thread, sum_of_squares}) {
    return async (action) => {
        if (action === "start-thread") {
            await start_thread(BigInt(16));
        } else if (action === "sum-of-squares") {
            return await sum_of_squares(new Int32Array([1, 2, 3, 4]));
        } else {
            // TODO: comlink has example for transferring raw image data, maybe try to transfer
        }
    };
}

async function createWasmRuntime() {
    const multiThread = await import('./build/dream_runner.js');
    await multiThread.default();
    await multiThread.initThreadPool(navigator.hardwareConcurrency);
    return Comlink.proxy(wrapExports(multiThread));
}

Comlink.expose({
    wasmRuntime: createWasmRuntime()
});