export class CircularBuffer<T> {
    private buffer: (T | undefined)[];
    private start = 0; // index of the logical "front"
    private _size = 0;

    constructor(private readonly limit: number) {
        if (limit <= 0) {
            throw new Error("CircularBuffer limit must be > 0");
        }
        this.buffer = new Array(limit);
    }

    /** Current number of stored items */
    get size(): number {
        return this._size;
    }

    /** Add an item to the beginning, overwriting the oldest if full */
    unshift(item: T): void {
        this.start = (this.start - 1 + this.limit) % this.limit;
        this.buffer[this.start] = item;

        if (this._size < this.limit) {
            this._size++;
        }
    }

    /** Get item at index (0 = newest/front) */
    get(index: number): T | undefined {
        if (index < 0 || index >= this._size) return undefined;
        return this.buffer[(this.start + index) % this.limit];
    }

    /** Convert buffer to an ordered array */
    toArray(): T[] {
        const out: T[] = [];
        for (let i = 0; i < this._size; i++) {
            out.push(this.get(i)!);
        }
        return out;
    }

    /** Check if buffer is at capacity */
    isFull(): boolean {
        return this._size === this.limit;
    }
}
