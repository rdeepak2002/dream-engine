/**
 * Get bytes for a file
 * @param file_path
 * @returns {Promise<Uint8Array>}
 */
export async function readBinary(file_path) {
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

/**
 * Get files in directory. Returns in format Array<[string, bool]> where the string is the file name and the bool
 * is whether the file is a directory.
 * @param file_path
 * @returns {Promise<*[]>}
 */
export async function readDir(file_path) {
    const filePath = file_path;
    // throw new Error("test 1");
    let root = await navigator.storage.getDirectory();
    const filepath_arr = filePath.split('/');
    let curDir = root;
    for (let i = 0; i < filepath_arr.length; i++) {
        const dirName = filepath_arr[i];
        if (dirName && dirName !== "") {
            curDir = await curDir.getDirectoryHandle(dirName, {create: false});
        }
    }
    let names = [];
    for await(const [key, value] of curDir.entries()) {
        // value.name is the filename (same as key)
        // value.kind is either 'file' or 'directory'
        let is_dir = value.kind === 'directory';
        let entry = [value?.name || key, is_dir];
        names.push(entry);
    }
    return names;
}
