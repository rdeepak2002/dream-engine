export async function readFileFromStorage(file_path) {
    const filePath = file_path;
    let root = await navigator.storage.getDirectory();
    const filepath_arr = filePath.split('/');
    const fileName = filepath_arr[filepath_arr.length - 1];
    // create the necessary directories to place the file into
    let curDir = root;
    for (let i = 0; i < filepath_arr.length - 1; i++) {
        const dirName = filepath_arr[i];
        if (dirName && dirName !== "") {
            curDir = await curDir.getDirectoryHandle(dirName, {create: false});
        }
    }
    let fileHandle = await curDir.getFileHandle(fileName);
    const file = await fileHandle.getFile();
    const buffer = await file.arrayBuffer();
    return new Uint8Array(buffer);
}