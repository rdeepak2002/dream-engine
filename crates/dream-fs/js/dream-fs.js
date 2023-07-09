import {fs} from 'https://cdn.jsdelivr.net/npm/memfs@4.2.0/+esm';

/**
 * Get bytes for a file.
 * @param file_path
 * @returns {Promise<Uint8Array>}
 */
export function readBinary(file_path) {
    return fs.readFileSync(file_path);
}

/**
 * Get files in directory. Returns in format Array<[string, bool]> where the string is the file name and the bool
 * is whether the file is a directory.
 * @param file_path
 * @returns {[]}
 */
export function readDir(file_path) {
    let readdirResult = fs.readdirSync(file_path, {withFileTypes: true});
    let result = [];
    for (let i = 0; i < readdirResult.length; i++) {
        const readdirRes = readdirResult[i];
        result.push([readdirRes.name, readdirRes.isDirectory()]);
    }
    return result;
}

/**
 * Return true if file exists.
 * @param file_path
 * @returns {Promise<boolean>}
 */
export function fileExists(file_path) {
    return fs.existsSync(file_path);
}

/**
 * Dump Uint8Array content to a specific file
 * @param file_path
 * @param content
 * @returns {Promise<void>}
 */
export function writeAll(file_path, content) {
    // TODO: make method sync
    // TODO: make directory if necessary
    console.error("TODO in dream-fs!");
    console.warn("writing file", file_path);
    fs.writeFileSync(file_path, content);
    // const filePath = file_path;
    // let root = await navigator.storage.getDirectory();
    // const filepath_arr = filePath.split('/');
    // const fileName = filepath_arr[filepath_arr.length - 1];
    // // create the necessary directories to place the file into
    // let curDir = root;
    // for (let i = 0; i < filepath_arr.length - 1; i++) {
    //     const dirName = filepath_arr[i];
    //     if (dirName && dirName !== "") {
    //         curDir = await curDir.getDirectoryHandle(dirName, {create: true});
    //     }
    // }
    // let fileHandle = await curDir.getFileHandle(fileName, {create: true});
    // const writable = await fileHandle.createWritable();
    // let blob = new Blob([content]);
    // await writable.write(blob);
    // await writable.close();
    // return null;
}