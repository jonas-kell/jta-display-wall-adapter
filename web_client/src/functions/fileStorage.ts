import { openDB, IDBPDatabase } from "idb";

interface KVSchema {
    kv: {
        key: string;
        value: string;
    };
}

let dbPromise: Promise<IDBPDatabase<KVSchema>> | null = null;

function getDB(): Promise<IDBPDatabase<KVSchema>> {
    if (!dbPromise) {
        dbPromise = openDB<KVSchema>("kv-store", 1, {
            upgrade(db) {
                db.createObjectStore("kv");
            },
        });
    }
    return dbPromise;
}

export async function storeString(key: string, value: string): Promise<void> {
    const db = await getDB();
    await db.put("kv", value, key);
}

export async function loadString(key: string): Promise<string | undefined> {
    const db = await getDB();
    return db.get("kv", key);
}

export async function deleteString(key: string): Promise<void> {
    const db = await getDB();
    await db.delete("kv", key);
}
