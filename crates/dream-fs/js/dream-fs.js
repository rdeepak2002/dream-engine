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
 * Get string text from a file.
 * @param file_path
 * @returns {Promise<string>}
 */
export async function readString(file_path) {
    // TODO: untested
    // TODO: make sync
    console.error("TODO in dream-fs!");
    let buf = fs.readFileSync(file_path);
    return String.fromCharCode.apply(null, new Uint16Array(buf));
}

/**
 * Get files in directory. Returns in format Array<[string, bool]> where the string is the file name and the bool
 * is whether the file is a directory.
 * @param file_path
 * @returns {Promise<*[]>}
 */
export async function readDir(file_path) {
    // TODO: start using fs sync
    // TODO: make sync
    console.error("TODO in dream-fs!");
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
        // console.error('entry thingy ', entry);
        names.push(entry);
    }
    return names;
}

/**
 * Return true if file exists.
 * @param file_path
 * @returns {Promise<boolean>}
 */
export async function fileExists(file_path) {
    // TODO: use fs sync
    // TODO: make sync
    console.error("TODO in dream-fs!");
    const filePath = file_path;
    let root = await navigator.storage.getDirectory();
    const filepath_arr = filePath.split('/');
    const fileName = filepath_arr[filepath_arr.length - 1];
    // create the necessary directories to place the file into
    let curDir = root;
    for (let i = 0; i < filepath_arr.length - 1; i++) {
        const dirName = filepath_arr[i];
        if (dirName && dirName !== "") {
            try {
                curDir = await curDir.getDirectoryHandle(dirName, {create: false});
            } catch (e) {
                return false;
            }
        }
    }
    try {
        await curDir.getFileHandle(fileName);
        return true;
    } catch (e1) {
        try {
            await curDir.getDirectoryHandle(fileName);
            return true;
        } catch (e2) {
            return false;
        }
    }
}

/**
 * Dump Uint8Array content to a specific file
 * @param file_path
 * @param content
 * @returns {Promise<void>}
 */
export async function writeAll(file_path, content) {
    // TODO: make method sync
    // TODO: make directory if necessary
    console.error("TODO in dream-fs!");
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