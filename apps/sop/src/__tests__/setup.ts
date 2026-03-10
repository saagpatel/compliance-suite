import "@testing-library/jest-dom/vitest";

function createStorage() {
  let store = new Map<string, string>();

  return {
    get length() {
      return store.size;
    },
    clear() {
      store = new Map<string, string>();
    },
    getItem(key: string) {
      return store.has(key) ? store.get(key)! : null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    }
  };
}

const storage =
  typeof globalThis.localStorage?.setItem === "function" &&
  typeof globalThis.localStorage?.clear === "function"
    ? globalThis.localStorage
    : createStorage();

Object.defineProperty(globalThis, "localStorage", {
  value: storage,
  configurable: true
});

Object.defineProperty(globalThis, "sessionStorage", {
  value: storage,
  configurable: true
});
