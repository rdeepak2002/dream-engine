import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";

// Wrap wasm-bindgen exports (the `generate` function) to add time measurement.
function wrapExports({sum_of_squares}) {
    return ({width, height, maxIterations}) => {
        const start = performance.now();
        const rawImageData = sum_of_squares(new Int32Array([1, 2, 3]));
        const time = performance.now() - start;
        return {
            // Little perf boost to transfer data to the main thread w/o copying.
            rawImageData: Comlink.transfer(rawImageData, [rawImageData.buffer]),
            time
        };
    };
}

async function initHandlers() {
    const multiThread = await import(
        './build/dream_runner.js'
        );
    await multiThread.default();
    await multiThread.initThreadPool(navigator.hardwareConcurrency);
    multiThread.start_thread(BigInt(16));
    console.error(multiThread.sum_of_squares(new Int32Array([1, 2, 3])));
    return Comlink.proxy(wrapExports(multiThread));
}

Comlink.expose({
    handlers: initHandlers()
});