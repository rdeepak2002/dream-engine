import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";

const workerInstance = {
    async initSharedMem(sharedMemory) {
        const multiThread = await import('./build/dream_runner.js');
        await multiThread.default(undefined, sharedMemory);
        await multiThread.initThreadPool(navigator.hardwareConcurrency);
        await multiThread.start_thread(BigInt(16));
    }
};

Comlink.expose(workerInstance);