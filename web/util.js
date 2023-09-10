import init, {run_main} from './build/dream_runner.js';
import * as Comlink from "./unpkg.com_comlink@4.4.1_dist_esm_comlink.mjs";
import {fs} from 'https://cdn.jsdelivr.net/npm/memfs@4.2.0/+esm';

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function updateLoaderBarText(text) {
    const resourceLoaderTextTag = "dream-resource-loader-text";
    const resourceLoaderText = document.getElementById(resourceLoaderTextTag);

    if (!resourceLoaderText) {
        console.warn(`Unable to find loading text element with ID ${resourceLoaderTextTag}`);
        return;
    }

    resourceLoaderText.innerText = text;
}

function updateLoaderBar(percentLoaded) {
    const resourceLoaderBarTag = "bar";
    const resourceLoaderBarBgTag = "bg";

    const resourceLoaderBar = document.getElementById(resourceLoaderBarTag);
    const resourceLoaderBarBg = document.getElementById(resourceLoaderBarBgTag);

    if (!resourceLoaderBar) {
        console.warn(`Unable to find loading bar element with ID ${resourceLoaderBarTag}`);
        return;
    }

    if (!resourceLoaderBarBg) {
        console.warn(`Unable to find loading bar background element with ID ${resourceLoaderBarBgTag}`);
        return;
    }

    const maxResourceLoaderBarWidth = resourceLoaderBarBg?.getBoundingClientRect()?.width;
    if (maxResourceLoaderBarWidth) {
        resourceLoaderBar.style.width = `${Math.round(maxResourceLoaderBarWidth * percentLoaded)}px`;
    } else {
        console.warn('Unable to retrieve width of resource loading bar background');
    }
}

function showWindowOverlay() {
    const windowOverlayTag = "dream-window-overlay";
    const windowOverlay = document.getElementById(windowOverlayTag);
    windowOverlay.style.display = "flex";
    windowOverlay.classList.add("fadeIn");
    windowOverlay.classList.remove("fadeOut");
}

function hideWindowOverlay() {
    const windowOverlayTag = "dream-window-overlay";
    const windowOverlay = document.getElementById(windowOverlayTag);
    windowOverlay.classList.add("fadeOut");
    windowOverlay.classList.remove("fadeIn");
    sleep(1000).then(() => {
        windowOverlay.style.display = "none";
    });
}

function disableWebKeyboardEvents() {
    const canvasElement = document?.getElementsByTagName('canvas')[0];
    if (canvasElement) {
        // remove context menu pop-up when right-clicking on canvas
        canvasElement.addEventListener('contextmenu', function (e) {
            if (e.button === 2) {
                e.preventDefault();
                return false;
            }
        }, false);
        // handle command s
        canvasElement.addEventListener("keydown", function (e) {
            if (e.key === 's' && (navigator.platform.match("Mac") ? e.metaKey : e.ctrlKey)) {
                e.preventDefault();
            }
        }, false);
        // handle command o
        canvasElement.addEventListener("keydown", function (e) {
            if (e.key === 'o' && (navigator.platform.match("Mac") ? e.metaKey : e.ctrlKey)) {
                e.preventDefault();
            }
        }, false);
        // handle command r
        canvasElement.addEventListener("keydown", function (e) {
            if (e.key === 'r' && (navigator.platform.match("Mac") ? e.metaKey : e.ctrlKey)) {
                e.preventDefault();
            }
        }, false);
        // handle command p
        canvasElement.addEventListener("keydown", function (e) {
            if (e.key === 'p' && (navigator.platform.match("Mac") ? e.metaKey : e.ctrlKey)) {
                e.preventDefault();
            }
        }, false);
    } else {
        console.error('Unable to find canvas to disable keyboard events');
    }
}

const fetchResourceFile = async (paths, resourceFileDescriptor, showDownloadLogs = false) => {
    let joinedPaths = paths.join("/");
    // url of the file system for debugging purposes
    const full_file_path = `${joinedPaths ? `${joinedPaths}/` : joinedPaths}${resourceFileDescriptor.filepath}`;
    const filepath_arr = full_file_path.split('/');
    // create the necessary directories to place the file into
    let runningMemFsDir = "";
    for (let i = 0; i < filepath_arr.length - 1; i++) {
        const dirName = filepath_arr[i];
        runningMemFsDir += `/${filepath_arr[i]}`;
        if (dirName && dirName !== "") {
            if (!fs.existsSync(runningMemFsDir)) {
                fs.mkdirSync(runningMemFsDir);
            }
        }
    }
    const filePath = `/${full_file_path}`;
    const fileUrl = resourceFileDescriptor?.fileUrl || `${filePath}`;
    // fetch the file from the URL and get the blob data
    let fetchedFileBlob;
    try {
        // console.log(`Downloading ${fileUrl}`);
        if (showDownloadLogs) {
            updateLoaderBarText(`Downloading ${fileUrl}`);
        }
        const fetchedFile = await fetch(fileUrl);
        fetchedFileBlob = await fetchedFile.blob();
    } catch (e) {
        console.error(`Unable to download ${fileUrl}`, e);
        throw new Error(`Unable to download ${fileUrl}`);
    }
    // write the file to memfs
    try {
        const arr = new Uint8Array(await fetchedFileBlob.arrayBuffer());
        fs.writeFileSync(filePath, arr);
        // TODO: also persist in idb
    } catch (e) {
        console.error(`Unable to write ${filePath} to file system`, e);
        throw new Error(`Unable to write ${filePath} to file system`);
    }
}

// TODO: in long run we want to move all this logic to be called directly by Rust like what we are doing in dream-fs/js/dream-fs.js
// ^ so whenever project starts up in general we want to read through the JSON file or query db to get all the file resource descriptors
// then for each one 'download' it to our project (if its locally stored, dont do anything on desktop build when downloading a file)
// but ofc for web build we want to run above fetchResource() method when downloading a file
const fetchResourceFiles = async (showDownloadLogs = false) => {
    showWindowOverlay();
    if (showDownloadLogs) {
        updateLoaderBarText("Retrieving filesystem root");
    }
    updateLoaderBar(0.0 / 9);

    // TODO: in long run we want users to toggle between a local and cloud saved project
    // TODO: have JSON file (or db thingy) that specifies what files are a part of the project & urls (so in future we can do google docs approach if user chooses to do a cloud synced project)
    let paths = ["examples", "blank"];
    const projectUrl = `${window.location.protocol}//${window.location.host}${paths.length === 0 ? "" : "/" + paths.join("/")}`;
    let response, resources;
    try {
        response = await fetch(`${projectUrl}/files.json`);
        resources = await response.json();
    } catch (e) {
        console.error('Unable to download and parse files.json', e);
        alert('Unable to download and parse files.json');
        return;
    }

    // fetch each resource file
    for (let i = 0; i < resources.length; i++) {
        let resourceFileDescriptor = resources[i];
        await fetchResourceFile(paths, resourceFileDescriptor, showDownloadLogs);
        updateLoaderBar((i + 1) / resources.length);
        await sleep(10);
    }

    if (showDownloadLogs) {
        updateLoaderBarText("Done downloading resources");
    }

    // TODO: when deciding which things to fetch and which things to use from index db:
    // TODO STEP 1: check local storage 'lastSynced' variable indicating when we pulled down data
    // TODO STEP 2: send query to server for all project resources with updatedAt timestamp greater than lastSynced
    // TODO STEP 3: server will only send back things we need, so only pull those things and overwrite indexed db thing

    // TODO (keep below code): below is an example of fetching file from url (useful when we do cloud syncing like google docs, where each file will be stored in storage bucket)
    // and the filepath + url can be stored in a db collection as a single db entry
    // await fetchResourceFile(root, {
    //     filepath: "foo/bar/Box.glb",
    //     fileUrl: "http://127.0.0.1:8080/res/Box.glb",
    // });
}

const startApplication = (showDownloadLogs = false) => {
    fetchResourceFiles(showDownloadLogs).then(() => {
        // initialize web assembly application and disable possible keyboard input events
        init().then(async (wasmRuntime) => {
            // TODO: enable multi threading when headers are correct and navigator.hardware supports it
            // problem where rayon spawn sometimes blocks main thread which causes program to fail
            const mem = wasmRuntime.memory;
            const enableMultiThreading = false;
            if (enableMultiThreading) {
                let workerInstance = await Comlink.wrap(
                    new Worker(new URL('./wasm-worker.js', import.meta.url), {
                        type: 'module'
                    })
                );
                await workerInstance.initSharedMem(mem);
            } else {
                const backgroundAsyncInstance = await import('./build/dream_runner.js');
                await backgroundAsyncInstance.default(undefined, mem);
                await backgroundAsyncInstance.set_multithreading_enabled(false);
                const asyncTask = setInterval(() => {
                    backgroundAsyncInstance.complete_task();
                }, 100);
            }

            hideWindowOverlay();
            disableWebKeyboardEvents();

            await run_main();
        }).catch((err) => {
            alert('Unable to initialize application. Please try again later.');
            console.error('Unable to initialize application', err);
        });
    }).catch((error) => {
        console.error('Unable to fetch resource files', error);
        alert('Unable to fetch resource files, please try again later');
    });
}

export {disableWebKeyboardEvents, fetchResourceFiles, startApplication};