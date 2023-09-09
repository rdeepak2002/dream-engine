import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";

const workerInstance = {
    async initSharedMem(sharedMemory) {
        const multiThread = await import('./build/dream_runner.js');
        await multiThread.default(undefined, sharedMemory);
        console.debug("Initializing thread pool");
        multiThread.initThreadPool(navigator.hardwareConcurrency).then((res) => {
            console.debug("Done initializing thread pool", res);
        });
        // await multiThread.start_thread(BigInt(500));
    }
};

Comlink.expose(workerInstance);