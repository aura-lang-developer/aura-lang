// TypeScript Definitions for Aura Module
/* eslint-disable */

export declare const None: { readonly $: "None" };
export declare function Some<T>(val: T): { readonly $: "Some"; readonly _0: T };
export declare function Ok<T>(val: T): { readonly $: "Ok"; readonly value: T };
export declare function Err<E>(err: E): { readonly $: "Err"; readonly error: E };
export declare interface Iterator<T> {
  next(): ({ readonly $: "Some"; readonly _0: T } | { readonly $: "None" });
  map<U>(fn: (item: T) => U): Iterator<U>;
  filter(pred: (item: T) => boolean): Iterator<T>;
  sort(cmp?: (a: T, b: T) => number): Iterator<T>;
  toList(): T[];
  toArray(): T[];
  [Symbol.iterator](): IterableIterator<T>;
}
export declare type Interator<T> = Iterator<T>;
export declare class AuraIterator<T> implements Iterator<T> {
  constructor(iter: any);
  next(): ({ readonly $: "Some"; readonly _0: T } | { readonly $: "None" });
  map<U>(fn: (item: T) => U): Iterator<U>;
  filter(pred: (item: T) => boolean): Iterator<T>;
  sort(cmp?: (a: T, b: T) => number): Iterator<T>;
  toList(): T[];
  toArray(): T[];
  [Symbol.iterator](): IterableIterator<T>;
}
export declare function iter<T>(coll: T[] | Iterator<T> | Iterable<T>): Iterator<T>;
export declare function map<T, U>(coll: T[], fn: (item: T) => U): U[];
export declare function map<T, U>(coll: Iterator<T>, fn: (item: T) => U): Iterator<U>;
export declare function map<T, U>(coll: any, fn: (item: T) => U): any;
export declare function filter<T>(coll: T[], pred: (item: T) => boolean): T[];
export declare function filter<T>(coll: Iterator<T>, pred: (item: T) => boolean): Iterator<T>;
export declare function filter<T>(coll: any, pred: (item: T) => boolean): any;
export declare function sort<T>(coll: T[], cmp?: (a: T, b: T) => number): T[];
export declare function sort<T>(coll: Iterator<T>, cmp?: (a: T, b: T) => number): Iterator<T>;
export declare function sort<T>(coll: any, cmp?: (a: T, b: T) => number): any;
export declare function toList<T>(coll: T[] | Iterator<T> | any): T[];
export declare function toArray<T>(coll: T[] | Iterator<T> | any): T[];
export declare class Channel<T> {
  constructor(capacity?: number);
  static make<T>(capacity?: number): Channel<T>;
  send(value: T): Promise<boolean>;
  recv(): Promise<{ readonly $: "Some"; readonly _0: T } | { readonly $: "None" }>;
  close(): void;
  [Symbol.asyncIterator](): AsyncIterableIterator<T>;
}
export declare type SendChannel<T> = {
  send(value: T): Promise<boolean>;
  close(): void;
};
export declare type RecvChannel<T> = {
  recv(): Promise<{ readonly $: "Some"; readonly _0: T } | { readonly $: "None" }>;
  close(): void;
  [Symbol.asyncIterator](): AsyncIterableIterator<T>;
};
export declare class WaitGroup {
  constructor();
  static new(): WaitGroup;
  add(n?: number): void;
  done(): void;
  wait(): Promise<void>;
}
export declare class Context {
  static background(): Context;
  static todo(): Context;
  static withCancel(parent: Context): [Context, () => void];
  static withTimeout(parent: Context, ms: number): [Context, () => void];
  static withValue(parent: Context, key: string, value: any): Context;
  done(): RecvChannel<void>;
  err(): ({ readonly $: "Some"; readonly _0: string } | { readonly $: "None" });
  isDone(): boolean;
  value(key: string): ({ readonly $: "Some"; readonly _0: any } | { readonly $: "None" });
}
export declare class Pointer<T> {
  value: T;
  val: T;
  constructor(val: T);
}
export declare function ptr<T>(val: T): Pointer<T>;
export declare class AuraPanic extends Error {
  value: any;
  isPanic: boolean;
  constructor(value: any);
}
export declare function panic(value: any): never;
export declare function recover(): ({ readonly $: "Some"; readonly _0: any } | { readonly $: "None" });

export declare class Mutex {
  constructor();
  static new(): Mutex;
  lock(): Promise<void>;
  unlock(): void;
  tryLock(): boolean;
}

export declare class RWMutex {
  constructor();
  static new(): RWMutex;
  lock(): Promise<void>;
  unlock(): void;
  rLock(): Promise<void>;
  rUnlock(): void;
}

export declare class Once {
  constructor();
  static new(): Once;
  do(fn: () => void): void;
}

export declare class Pool<T = any> {
  constructor(newFn?: (() => T) | null);
  static new<T = any>(newFn?: (() => T) | null): Pool<T>;
  get(): T | null;
  put(item: T): void;
}

export declare const Sync: {
  Mutex: typeof Mutex;
  RWMutex: typeof RWMutex;
  Once: typeof Once;
  Pool: typeof Pool;
  WaitGroup: typeof WaitGroup;
};

export declare function close(ch: Channel<any> | SendChannel<any> | RecvChannel<any>): void;
export declare const StructTag: {
  get(target: any, field: string, tagKey: string): ({ readonly $: "Some"; readonly _0: string } | { readonly $: "None" });
};

export declare function spawn(task: (() => any) | Promise<any>): void;
export declare function timeout(ms: number): Promise<void>;
export declare function sleep(ms: number): Promise<void>;
export declare function print(...args: any[]): void;
export declare function println(...args: any[]): void;
export declare function assert(condition: boolean, message?: any): void;
export declare function assertTrue(condition: boolean, message?: any): void;
export declare function assertFalse(condition: boolean, message?: any): void;
export declare function assertEqual<T>(actual: T, expected: T, message?: any): void;
export declare function assertNotEqual<T>(actual: T, expected: T, message?: any): void;
export declare function assertDeepEqual<T>(actual: T, expected: T, message?: any): void;
export declare function assertThrows(fn: () => any, expected?: any): void;
export declare function fetch(url: string, options?: any): Promise<any>;
export declare function __aura_select(cases: any[], defaultFn?: (() => any) | null): Promise<any>;

export interface ServeMux {
  use(middleware: (req: any, res: any, next?: () => any) => any): this;
  handle(pattern: string, handler: any): this;
  handleFunc(pattern: string, handler: (req: any, res: any) => any): this;
  get(pattern: string, handler: (req: any, res: any) => any): this;
  post(pattern: string, handler: (req: any, res: any) => any): this;
  put(pattern: string, handler: (req: any, res: any) => any): this;
  delete(pattern: string, handler: (req: any, res: any) => any): this;
  patch(pattern: string, handler: (req: any, res: any) => any): this;
  serveHTTP(req: any, res: any): Promise<void>;
  listenAndServe(addr: string): Promise<any>;
  close(): Promise<void>;
}
export declare const http: {
  StatusOK: number;
  StatusCreated: number;
  StatusAccepted: number;
  StatusNoContent: number;
  StatusMovedPermanently: number;
  StatusFound: number;
  StatusBadRequest: number;
  StatusUnauthorized: number;
  StatusForbidden: number;
  StatusNotFound: number;
  StatusMethodNotAllowed: number;
  StatusConflict: number;
  StatusUnprocessableEntity: number;
  StatusInternalServerError: number;
  StatusBadGateway: number;
  StatusServiceUnavailable: number;
  newServeMux(): ServeMux;
  NewServeMux(): ServeMux;
  handleFunc(path: string, handler: (req: any, res: any) => any): void;
  handle(path: string, handler: any): void;
  get(path: string, handler: (req: any, res: any) => any): void;
  post(path: string, handler: (req: any, res: any) => any): void;
  put(path: string, handler: (req: any, res: any) => any): void;
  delete(path: string, handler: (req: any, res: any) => any): void;
  patch(path: string, handler: (req: any, res: any) => any): void;
  use(middleware: (req: any, res: any, next?: () => any) => any): void;
  listenAndServe(addr: string): Promise<any>;
  serveHTTP(req: any, res: any): Promise<void>;
  close(): Promise<void>;
  json(res: any, statusCode: number, data: any): void;
  text(res: any, statusCode: number, message: string): void;
  html(res: any, statusCode: number, htmlContent: string): void;
  error(res: any, errorMsg: string, statusCode?: number): void;
  redirect(res: any, url: string, statusCode?: number): void;
  pathValue(req: any, paramName: string): string;
  query(req: any, queryName: string): string;
  parseJson(req: any): Promise<{ readonly $: "Ok"; readonly value: any } | { readonly $: "Err"; readonly error: string }>;
  serveFile(res: any, req: any, filePath: string): void;
  fileServer(rootDir: string): (req: any, res: any) => void;
};

export declare const os: {
  args: string[];
  env(key: string): string;
  getEnv(key: string): string;
  setEnv(key: string, val: string): void;
  exit(code: number): void;
  readFile(filePath: string): string;
  writeFile(filePath: string, data: string): void;
  hostname(): string;
};

export declare const time: {
  now(): number;
  sleep(ms: number): Promise<void>;
};

export interface MysqlQueryResult {
  affectedRows: number;
  insertId: number;
  changedRows: number;
  warningCount: number;
}
export interface MysqlTransaction {
  query(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  execute(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: MysqlQueryResult } | { readonly $: "Err"; readonly error: string }>;
  queryRow(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: any } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  commit(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  rollback(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
}
export interface MysqlPool {
  query(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  execute(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: MysqlQueryResult } | { readonly $: "Err"; readonly error: string }>;
  queryRow(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: any } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  transaction<T>(callback: (tx: MysqlTransaction) => Promise<T> | T): Promise<{ readonly $: "Ok"; readonly value: T } | { readonly $: "Err"; readonly error: string }>;
  ping(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  close(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  end(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  escape(val: any): string;
  format(sql: string, params?: any[]): string;
}
export declare const mysql: {
  createPool(config: any): MysqlPool;
  createConnection(config: any): MysqlPool;
  open(uri: string): MysqlPool;
  escape(val: any): string;
  format(sql: string, params?: any[]): string;
};

export interface PgQueryResult {
  rowCount: number;
  command: string;
  insertId: number;
  affectedRows: number;
}
export type PostgresQueryResult = PgQueryResult;
export interface PgTransaction {
  query(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  execute(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: PgQueryResult } | { readonly $: "Err"; readonly error: string }>;
  queryRow(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: any } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  commit(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  rollback(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
}
export type PostgresTransaction = PgTransaction;
export interface PgPool {
  query(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  execute(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: PgQueryResult } | { readonly $: "Err"; readonly error: string }>;
  queryRow(sql: string, params?: any[]): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: any } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  transaction<T>(callback: (tx: PgTransaction) => Promise<T> | T): Promise<{ readonly $: "Ok"; readonly value: T } | { readonly $: "Err"; readonly error: string }>;
  ping(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  close(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  end(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  escapeIdentifier(id: any): string;
  escapeLiteral(val: any): string;
  escape(val: any): string;
  format(sql: string, params?: any[]): string;
}
export type PostgresPool = PgPool;
export type PgClient = PgPool;
export type PostgresClient = PgPool;
export declare const postgres: {
  createPool(config: any): PgPool;
  createClient(config: any): PgPool;
  open(uri: string): PgPool;
  escapeIdentifier(id: any): string;
  escapeLiteral(val: any): string;
  escape(val: any): string;
  format(sql: string, params?: any[]): string;
};
export declare const pg: typeof postgres;

export interface MongoInsertResult {
  insertedId: any;
  acknowledged: boolean;
}
export interface MongoInsertManyResult {
  insertedIds: any[];
  insertedCount: number;
  acknowledged: boolean;
}
export interface MongoUpdateResult {
  matchedCount: number;
  modifiedCount: number;
  upsertedId: any;
  acknowledged: boolean;
}
export interface MongoDeleteResult {
  deletedCount: number;
  acknowledged: boolean;
}
export interface MongoCollection {
  find(filter?: any, options?: any): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  findOne(filter?: any, options?: any): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: any } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  insertOne(doc: any): Promise<{ readonly $: "Ok"; readonly value: MongoInsertResult } | { readonly $: "Err"; readonly error: string }>;
  insertMany(docs: any[]): Promise<{ readonly $: "Ok"; readonly value: MongoInsertManyResult } | { readonly $: "Err"; readonly error: string }>;
  updateOne(filter: any, update: any, options?: any): Promise<{ readonly $: "Ok"; readonly value: MongoUpdateResult } | { readonly $: "Err"; readonly error: string }>;
  updateMany(filter: any, update: any, options?: any): Promise<{ readonly $: "Ok"; readonly value: MongoUpdateResult } | { readonly $: "Err"; readonly error: string }>;
  deleteOne(filter?: any): Promise<{ readonly $: "Ok"; readonly value: MongoDeleteResult } | { readonly $: "Err"; readonly error: string }>;
  deleteMany(filter?: any): Promise<{ readonly $: "Ok"; readonly value: MongoDeleteResult } | { readonly $: "Err"; readonly error: string }>;
  countDocuments(filter?: any): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  aggregate(pipeline?: any[]): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  drop(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
}
export interface MongoDatabase {
  collection(name: string): MongoCollection;
  listCollections(): Promise<{ readonly $: "Ok"; readonly value: any[] } | { readonly $: "Err"; readonly error: string }>;
  dropDatabase(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
}
export interface MongoClient {
  db(name?: string): MongoDatabase;
  collection(name: string): MongoCollection;
  ping(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  close(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
}
export declare const mongodb: {
  connect(uriOrConfig?: any): Promise<{ readonly $: "Ok"; readonly value: MongoClient } | { readonly $: "Err"; readonly error: string }>;
  createClient(config?: any): MongoClient;
  open(uri: string): MongoClient;
  db(name?: string): MongoDatabase;
};
export declare const mongo: typeof mongodb;

export interface RedisClient {
  get(key: string): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: string } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  set(key: string, value: any, ttlSeconds?: number): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  del(keyOrKeys: any): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  exists(key: string): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  incr(key: string): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  decr(key: string): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  expire(key: string, seconds: number): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  ttl(key: string): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  keys(pattern?: string): Promise<{ readonly $: "Ok"; readonly value: string[] } | { readonly $: "Err"; readonly error: string }>;
  hget(key: string, field: string): Promise<{ readonly $: "Ok"; readonly value: { readonly $: "Some"; readonly _0: string } | { readonly $: "None" } } | { readonly $: "Err"; readonly error: string }>;
  hset(key: string, field: string, value: any): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  hgetall(key: string): Promise<{ readonly $: "Ok"; readonly value: any } | { readonly $: "Err"; readonly error: string }>;
  hdel(key: string, field: string): Promise<{ readonly $: "Ok"; readonly value: number } | { readonly $: "Err"; readonly error: string }>;
  ping(): Promise<{ readonly $: "Ok"; readonly value: string } | { readonly $: "Err"; readonly error: string }>;
  flushall(): Promise<{ readonly $: "Ok"; readonly value: boolean } | { readonly $: "Err"; readonly error: string }>;
  close(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
  quit(): Promise<{ readonly $: "Ok"; readonly value: void } | { readonly $: "Err"; readonly error: string }>;
}
export declare const redis: {
  createClient(config?: any): RedisClient;
  open(uri: string): RedisClient;
  connect(uriOrConfig?: any): Promise<{ readonly $: "Ok"; readonly value: RedisClient } | { readonly $: "Err"; readonly error: string }>;
};

