import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";

const workerInstance = {
    async initSharedMem(sharedMemory, importPath) {
        const multiThread = await import(importPath);
        await multiThread.default(undefined, sharedMemory);
        await multiThread.initThreadPool(navigator.hardwareConcurrency);
        multiThread.start_worker_thread();
        // await multiThread.start_thread(BigInt(500));
    }
};

Comlink.expose(workerInstance);