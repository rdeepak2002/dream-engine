import init from './build/dream.js';

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

const fetchResourceFile = async (root, resourceFileDescriptor, showDownloadLogs = false) => {
    // url of the file system for debugging purposes
    const filesystemUrl = `filesystem:${window.location.protocol}//${window.location.host}/temporary`
    const filepath_arr = resourceFileDescriptor.filepath.split('/');
    // create the necessary directories to place the file into
    let curDir = root;
    for (let i = 0; i < filepath_arr.length - 1; i++) {
        const dirName = filepath_arr[i];
        if (dirName && dirName !== "") {
            curDir = await curDir.getDirectoryHandle(dirName, {create: true});
            console.log('Created directory ', dirName);
        }
    }
    const fileName = filepath_arr[filepath_arr.length - 1];
    // const filePath = `/${fileName}`;
    const filePath = `/${resourceFileDescriptor.filepath}`;
    const fileUrl = resourceFileDescriptor?.fileUrl || `/res${filePath}`;
    // fetch the file from the URL and get the blob data
    let fetchedFileBlob;
    try {
        console.log(`Downloading ${fileUrl}`);
        if (showDownloadLogs) {
            updateLoaderBarText(`Downloading ${fileUrl}`);
        }
        const fetchedFile = await fetch(fileUrl);
        fetchedFileBlob = await fetchedFile.blob();
    } catch (e) {
        console.error(`Unable to download ${fileUrl}`, e);
        throw new Error(`Unable to download ${fileUrl}`);
    }
    // write the file to webkit persistent storage
    try {
        console.log(`Writing file downloaded from ${fileUrl} to ${filesystemUrl}${filePath}`)
        const fileHandle = await curDir.getFileHandle(fileName, {create: true});
        const writable = await fileHandle.createWritable();
        await writable.write(fetchedFileBlob);
        await writable.close();
    } catch (e) {
        console.error(`Unable to write ${filePath} to file system`, e);
        throw new Error(`Unable to write ${filePath} to file system`)
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

    // get root directory of file system
    let root;
    try {
        root = await navigator.storage.getDirectory();
    } catch (e) {
        console.error(`Unable to get root directory of temporary file system`, e);
        throw new Error(`Unable to get root directory of temporary file system`);
    }
    // clear file system by clearing root directory
    try {
        root.remove();
    } catch (e) {
        // TODO: doesn't work on safari and firefox
        console.error(`Unable to remove root directory`, e);
    }
    root = await navigator.storage.getDirectory();

    // TODO: in long run we want users to toggle between a local and cloud saved project
    // TODO: have JSON file (or db thingy) that specifies what files are a part of the project & urls (so in future we can do google docs approach if user chooses to do a cloud synced project)
    // TODO: stream read json file that describes each resourceFileDescriptor (or read from db for cloud saved projects)
    // TODO: use await navigator.storage.estimate() to ensure we have enough storage space available
    const resources = [
        {
            filepath: "link.glb",
            fileUrl: undefined,
        },
        {
            filepath: "cube.glb",
            fileUrl: undefined,
        },
        {
            filepath: "ice_cube.glb",
            fileUrl: undefined,
        },
        {
            filepath: "robot.glb",
            fileUrl: undefined,
        },
        {
            filepath: "scene.gltf",
            fileUrl: undefined,
        },
        {
            filepath: "scene.bin",
            fileUrl: undefined,
        },
        {
            filepath: "textures/main_mat_baseColor.png",
            fileUrl: undefined,
        },
        {
            filepath: "textures/main_mat_metallicRoughness.png",
            fileUrl: undefined,
        },
        {
            filepath: "textures/main_mat_normal.png",
            fileUrl: undefined,
        },
    ];

    // fetch each resource file
    for (let i = 0; i < resources.length; i++) {
        let resourceFileDescriptor = resources[i];
        await fetchResourceFile(root, resourceFileDescriptor, showDownloadLogs);
        updateLoaderBar((i + 1) / resources.length);
        await sleep(10);
    }

    if (showDownloadLogs) {
        updateLoaderBarText("Done downloading resources");
    }
    hideWindowOverlay();

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
        init().then(() => {
            disableWebKeyboardEvents();
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