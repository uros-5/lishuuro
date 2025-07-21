import JSZip from 'jszip';
export type DBRecord = File | Blob | ArrayBuffer;

export class IndexedDBStorage {
  private dbName: string;
  private dbVersion: number;
  private storeName: string;
  private db: IDBDatabase | null = null;

  constructor(dbName: string, storeName: string, dbVersion = 1) {
    this.dbName = dbName;
    this.storeName = storeName;
    this.dbVersion = dbVersion;
  }

  private async openDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.dbVersion);

      request.onupgradeneeded = (event: IDBVersionChangeEvent) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName);
        }
      };

      request.onsuccess = (event: Event) => {
        this.db = (event.target as IDBOpenDBRequest).result;
        resolve(this.db);
      };

      request.onerror = (event: Event) => {
        reject((event.target as IDBOpenDBRequest).error);
      };
    });
  }

  public async setItem(key: string, value: DBRecord): Promise<void> {
    const db = this.db || await this.openDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(this.storeName, 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.put(value, key);

      request.onsuccess = () => resolve();
      request.onerror = (event: Event) => {
        reject((event.target as IDBRequest).error);
      };
    });
  }

  public async getItem(key: string): Promise<DBRecord | undefined> {
    const db = this.db || await this.openDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(this.storeName, 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.get(key);

      request.onsuccess = (event: Event) => {
        const result = (event.target as IDBRequest).result;
        resolve(result || undefined);
      };

      request.onerror = (event: Event) => {
        reject((event.target as IDBRequest).error);
      };
    });
  }

  public async getAllZips(): Promise<({name: string, blob: DBRecord}) [] | undefined> {
    const db = this.db || await this.openDB();

    return new Promise((resolve, reject) => {
      const transaction = db.transaction(this.storeName, 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.getAllKeys();

      request.onsuccess = async (event: Event) => {
        const result = (event.target as IDBRequest).result;
        const items = [];
        for (let i = 0; i < result.length; i++) {
          const request = await this.getItem(result[i]);
          if (request) {
            items.push({name: result[i], blob: request})
          }
        }
        resolve(items);
      };

      request.onerror = (event: Event) => {
        reject((event.target as IDBRequest).error);
      };
    });

  }

  public async removeItem(key: string): Promise<void> {
    const db = this.db || await this.openDB();
    return new Promise((resolve, reject) => {
      const transaction = db.transaction(this.storeName, 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.delete(key);

      request.onsuccess = () => resolve();
      request.onerror = (event: Event) => {
        reject((event.target as IDBRequest).error);
      };
    });
  }
}


export async function extractZipFile(zipFile: File): Promise<Record<string, Blob>> {
  const zip = new JSZip();
  const content = await zip.loadAsync(zipFile);
  const files = content.files;
  const extractedFiles: Record<string, Blob> = {};

  for (const [relativePath, file] of Object.entries(files)) {
    if (!file.dir) { // Skip directories
      const fileData = await file.async('blob');
      extractedFiles[relativePath] = fileData;
    }
  }

  return extractedFiles;
}

export async function createZipFile(files: Record<string, Blob>): Promise<Blob> {
  const zip = new JSZip();

  for (const [path, blob] of Object.entries(files)) {
    zip.file(path, blob);
  }

  return await zip.generateAsync({ type: 'blob' });

}
