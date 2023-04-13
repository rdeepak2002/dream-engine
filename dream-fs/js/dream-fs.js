export async function readFileFromStorage(file_path) {
    const filePath = file_path;
    let root = await navigator.storage.getDirectory();
    let fileHandle = await root.getFileHandle(filePath);
    const file = await fileHandle.getFile();
    const buffer = await file.arrayBuffer();
    return new Uint8Array(buffer);
}