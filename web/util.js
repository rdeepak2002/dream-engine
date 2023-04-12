import init from './build/dream.js';

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

const fetchResourceFiles = async () => {
    // url of the file system for debugging purposes
    const filesystemUrl = `filesystem:${window.location.protocol}//${window.location.host}/temporary`
    // get root directory of file system
    let root;
    try {
        root = await navigator.storage.getDirectory();
    } catch (e) {
        console.error(`Unable to get root directory of temporary file system`, e);
        throw new Error(`Unable to get root directory of temporary file system`);
    }
    // get url of file to fetch
    const fileName = `Box.glb`;
    const filePath = `/${fileName}`;
    const fileUrl = `/res${filePath}`;
    // fetch the file from the URL and get the blob data
    let fetchedFileBlob;
    try {
        console.log(`Downloading ${fileUrl}`);
        const fetchedFile = await fetch(fileUrl);
        fetchedFileBlob = await fetchedFile.blob();
    } catch (e) {
        console.error(`Unable to download ${fileUrl}`, e);
        throw new Error(`Unable to download ${fileUrl}`);
    }
    // write the file to webkit persistent storage
    try {
        console.log(`Writing file downloaded from ${fileUrl} to ${filesystemUrl}${filePath}`)
        const fileHandle = await root.getFileHandle(fileName, {create: true});
        const writable = await fileHandle.createWritable();
        await writable.write(fetchedFileBlob);
        await writable.close();
    } catch (e) {
        console.error(`Unable to write ${filePath} to file system`, e);
        throw new Error(`Unable to write ${filePath} to file system`)
    }
}

const startApplication = (numMB = 1024) => {
    navigator.webkitPersistentStorage.requestQuota(numMB * 1024 * 1024, () => {
        window.webkitRequestFileSystem(window.TEMPORARY, numMB * 1024 * 1024, () => {
            fetchResourceFiles().then(() => {
                // initialize web assembly application and disable possible keyboard input events
                init().then(() => {
                    disableWebKeyboardEvents();
                });
            }).catch((error) => {
                console.error('Unable to fetch resource files', error);
                alert('Unable to fetch resource files');
            });
        }, () => {
            alert(`Unable to initialize ${numMB} MB of space for temporary file system`);
        });
    }, () => {
        alert(`Unable to initialize ${numMB} MB of space for temporary file system`);
    });
}

export {disableWebKeyboardEvents, fetchResourceFiles, startApplication};