# Persistent UltraGraph store, stage 1: comparison with embedded graph stores

* Date: 2026-10-16
* Status: draft plan. Facts were checked on 2026-10-16. Items still marked *pending* are re-checked
  before any number is published.
* Parent: `openspec/notes/ultragraph/PersistentStoreStage1.md` (S1-MEAS-1 to S1-MEAS-3 measure stage 1
  alone; this plan compares it with other stores) and `openspec/notes/ultragraph/PersistentStore.md`.
* Scope: embedded graph stores, meaning in-process stores backed by a file or a directory with no
  server, compared with stage 1 on the same machine, at the same durability, with the isolation level
  of each stated. Archived projects are excluded; an active fork of an archived project is eligible
  on its own merits.

## Summary

1. **Seven systems form the comparison set:**
   - LadybugDB, Grafeo, SQLite, SurrealDB in embedded mode, Neo4j embedded and ArcadeDB embedded;
   - DuckDB with DuckPGQ, for analytics only.

   agdb is optional (§2).
2. **No candidate's licence restricts publishing benchmark results** (§2).
3. **Durability settings decide the numbers more than engine design.**
   - On the benchmark machine, an append made durable with `F_FULLFSYNC` takes 5.5 ms, and one with
     `fsync` takes 25 µs (`PersistentStore.md`, Appendix A). On macOS, `fsync` leaves the data in the
     drive's cache.
   - What each system issues on macOS:
     - `F_FULLFSYNC`: Rust's `sync_all` and `sync_data`, LMDB, the JDK's `FileChannel.force`,
       LadybugDB, and SurrealDB's SurrealKV backend.
     - Plain `fsync`: SQLite at its defaults, DuckDB 1.5.6, and every RocksDB built by a Rust crate.
   - A durable-commit comparison on macOS therefore pins and reports each system's barrier. The
     comparison worth publishing runs at matched durability on Linux.
4. **Isolation differs and is stated next to every number:**
   - strictly serializable with one writer: stage 1;
   - serializable with one writer: SQLite, and LadybugDB by its README;
   - snapshot isolation by default: Grafeo, SurrealDB, DuckDB;
   - read committed: Neo4j, ArcadeDB.
5. **Stage 1 has no query language.** The comparison runs operation by operation through each
   system's most direct API, and counts query processing as part of that system's cost.
6. **Correctness is checked with speed.** After every workload, each system's graph is compared with
   the shadow model (S1-SIM-8). Systems known to acknowledge commits before they are durable serve as
   controls that show the harness detects lost writes (§4).

## 1. Rules for publishing a comparison

These follow the attribution rules of the stage-1 draft (§2.1 there): state no more than was
measured, and state it truthfully.

- **Every number names its full context:**
  - the system, its version and its configuration (durability setting, isolation level, cache
    sizes);
  - the API used;
  - the machine, operating system and filesystem;
  - the dataset;
  - the number of runs and their spread.
- **Two configurations per system:** its default, and matched durability (every acknowledged commit
  survives power loss).
- **No ratio appears without the table it comes from.**
- **Configurations, scripts and generator seeds are published,** and each system's maintainers are
  invited to review its configuration before results appear.
- **Licences are checked before publishing.** None of the candidates contains a clause restricting
  benchmark publication (§2). Neo4j's only such clause, §4(v) of its Online Trial Agreement, covers
  its hosted trial and not embedded Community; order-form terms were not checked.
- **LDBC's name stays off our results.** LDBC operates as the Graph Data Council since 2025. Its
  fair-use policy (based on its Byelaws v1.5, 2024-01-12) reserves "LDBC Benchmark Result" for runs
  audited by an LDBC-approved auditor. It treats as infringing any report of results "which are not
  part of a set of LDBC Benchmark Results, or in any other way states or implies that the results are
  endorsed by or originates from LDBC".
  - A workload derived from an LDBC benchmark carries the policy's suggested statement, "This is not
    an LDBC Benchmark" or "These are not LDBC Benchmark Results".
  - It credits LDBC under CC-BY 4.0 and states how it differs from the standard.
  - Byelaws §9.2(c) forbids using "the LDBC name, logo, or LDBC metrics … in reference to
    qualitative statements in reference to a product", unless the statement mentions a currently
    listed official result. So our numbers never sit beside audited results and never use LDBC's
    metric names.
  - Audits are open to members only, for a fee of 3,000 GBP.

## 2. The comparison set

### 2.1 In the set

| System | Licence | Rust access | Storage | Durability default, and barrier on macOS | Isolation and writers | Release and status | Graph interface |
|---|---|---|---|---|---|---|---|
| UltraGraph stage 1 | MIT | Native | Directory: WAL segments, CSR segments, manifests | Group commit; `sync_data`, which issues `F_FULLFSYNC` | Strictly serializable; one writer | Being specified | Native API |
| LadybugDB (a fork of Kuzu) | MIT | Official crate `lbug` 0.21.2, linking a prebuilt static library | Database file, WAL and shadow file; columnar, with CSR adjacency | Syncs every commit, with leader/follower group commit and no switch to turn it off; `F_FULLFSYNC` on the file and its directory when its build detects `HAS_FULLFSYNC`. Whether the prebuilt macOS library has it is *pending* | One writer by default; "Serializable ACID" is a README claim inherited from Kuzu | 0.21.2 (2026-10-01); active | Cypher |
| Grafeo | Apache-2.0 | Native crate `grafeo` 0.5.44 (stable, 2026-10-04); 0.6.0-beta.1 (2026-10-11) | One `.grafeo` file holding full checkpoint images, plus a WAL; whole graph in memory | Default `Batch` (100 ms or 1,000 records, decided at commit); `DurabilityMode::Sync` syncs every commit through `sync_all`, which issues `F_FULLFSYNC` | Snapshot isolation by default; read committed or serializable snapshot isolation per transaction; concurrent writers with write-write aborts | Active | GQL, Cypher, Gremlin, GraphQL, SPARQL, SQL/PGQ; CRUD API |
| SQLite | Public domain | `rusqlite` 0.40.2 (bundles SQLite 3.53.2) | One B-tree file, plus `-wal` and `-shm` in WAL mode | `synchronous=FULL` with a rollback journal by default; in WAL mode, `FULL` syncs the WAL after each commit, while `NORMAL` can roll back committed transactions after power loss. Plain `fsync` unless `PRAGMA fullfsync=ON` | Serializable; one writer | 3.54.0 (2026-10-09) | SQL with recursive CTEs; optionally the GraphQLite extension for Cypher (0.10.0, 2026-10-13, MIT, crate `graphqlite`) |
| SurrealDB, embedded | BSL-1.1 (Apache-2.0 from 2030-01-01) | Crate `surrealdb` 3.3.4 with the `kv-surrealkv` backend | SurrealKV directory: LSM plus WAL | `SyncMode::Every` by default, with group commit through a commit coordinator (a 5 ms window by default for SurrealKV); SurrealKV's `sync_all` issues `F_FULLFSYNC`. Its RocksDB backend issues plain `fsync` | Snapshot isolation (MVCC) | 3.3.4 (2026-10-13); active | SurrealQL `RELATE` and edge traversal |
| DuckDB with DuckPGQ | MIT | Official crate `duckdb` | One file plus a WAL; columnar row groups | Syncs the WAL on every write commit; plain `fsync` on macOS in 1.5.6. An `fsync_mode` setting is merged for an unreleased version | Snapshot isolation (optimistic MVCC); one read-write process | 1.5.6 (2026-09-28); DuckPGQ updated 2026-10-14 and described as a research project | SQL/PGQ; CSR rebuilt per query |
| Neo4j, embedded (Community) | GPL-3.0 | JVM only; needs a Java harness | Record store directory plus transaction logs | Forces every commit, with group commit; the JDK issues `F_FULLFSYNC` | Read committed; concurrent writers with locks | 2026.09.0; active | Java core API and Cypher |
| ArcadeDB, embedded | Apache-2.0 | JVM only; needs a Java harness | Paged files in a directory | Default `txWalFlush=0` does not flush at commit, so it runs with `txWalFlush=1`, which forces the log; the JDK issues `F_FULLFSYNC`. No group commit was found | Page-level optimistic MVCC; read committed by default, repeatable read available; concurrent writers with retry | 26.10.2 (2026-10-15); active | SQL, OpenCypher, Gremlin, GraphQL and a Java API; a CSR "Graph Analytical View" per edge type |

**Why each is in:**
- **LadybugDB** is the closest peer: embedded, CSR adjacency, one writer (serializable by its
  README), a barrier every commit with group commit, and an official Rust crate.
- **Grafeo** has the same shape in Rust: a WAL, full-image checkpoints, and the whole graph in memory.
  It runs with `DurabilityMode::Sync` and serializable transactions. Its known 0.6.0-beta.1 gap is
  stated with its results: a failed WAL write is logged and the commit still succeeds (CHANGELOG,
  #498).
- **SQLite** is the durable embedded baseline, as an edge table with indexes and recursive CTEs. It
  runs in WAL mode with `synchronous=FULL`, and with `fullfsync=ON` on macOS.
- **SurrealDB embedded** is a Rust store with established group commit and `F_FULLFSYNC`. It tests
  stage 1's group-commit throughput against an existing implementation.
- **DuckDB with DuckPGQ** is the analytics comparator (PageRank, WCC, shortest paths), with its CSR
  built per query. It stays out of the durable-commit workloads, because 1.5.6 issues plain `fsync`
  on macOS and syncs each commit separately.
- **Neo4j embedded** is the industry reference: it forces every commit with group commit.
- **ArcadeDB embedded** is the JVM property-graph peer, with a CSR analytical view.

### 2.2 Optional

- **agdb** (Apache-2.0, Rust, 0.15.0 of 2026-10-04) with `SyncMode::Commit`, which calls `sync_data`.
  It keeps the whole graph in memory and allows one writer, but it writes in place with an undo log.
  Its default, `SyncMode::None`, does not sync.

### 2.3 Out of the set

| System | Reason |
|---|---|
| RyuGraph, Bighorn (forks of Kuzu) | Dormant |
| CozoDB | No push since 2024-12-04; never sets a sync on RocksDB |
| IndraDB | Inactive since 2025-08; no per-commit sync |
| HelixDB | Its local store never fsyncs (object_store's `LocalFileSystem` with `fsync` off); embedded only from a source build |
| Jena TDB2 | Its semantics match (one writer, serializable, journal forced every commit), but its data model is RDF |
| GraphLite | Version 0.0.1; no push since 2026-02 |
| Raphtory | No WAL in the open-source tree |
| Oxigraph | RDF, and synchronous writes cannot be enabled |
| TerminusDB and terminus-store | TerminusDB is a server; terminus-store is a triple-layer store without a WAL |
| FalkorDB and FalkorDBLite | Out of process (FalkorDBLite starts `redis-server`); SSPL-1.0 |
| OrientDB | No per-commit fsync; the WAL syncs about once a second |
| JanusGraph on BerkeleyDB JE | Its storage engine has been frozen since 2018 |
| OverGraph, SparrowDB, uni-db, sqlitegraph | Too young. SparrowDB (WAL plus CSR, one writer, `sync_all` on every commit by default) is the one to re-check |

**Servers** (Memgraph, Neo4j server, Dgraph, NebulaGraph, TigerGraph) belong to the stage-2
comparison. **The research stores** (LiveGraph, Teseo, Sortledton, RapidStore, RadixGraph) have no
working recovery (`PersistentStore.md` §3.4).

### 2.4 Published benchmarks of the set

| System | What exists | Durability in those numbers |
|---|---|---|
| SurrealDB | surrealdb.com/benchmarks; Threadripper 9970X, 128 GiB, Lexar EQ790, Ubuntu 24.04, 2026-05-28; harness crud-bench (Apache-2.0) with a `--sync` flag. The embedded category compares against SQLite only; graph traversal is "coming soon" | States "each committed transaction is flushed and fsync'd" |
| Grafeo | Performance page (Ryzen 9 5900X, 64 GB, 980 Pro NVMe, Ubuntu 22.04); comparisons from GrafeoDB/graph-bench, labelled "Not Official LDBC Results" | graph-bench opens Grafeo and LadybugDB in memory by default, so those numbers exclude durability; the graph-bench run itself reports "Platform: Windows" |
| DuckDB with DuckPGQ | CIDR 2023 paper: path-finding on SNB SF10 to SF3000 against Neo4j 4.4.2; 48-vCPU Xeon 8375C, 248 GB | Analytics only; no ingest or point-traversal numbers |
| ArcadeDB | Vendor page (2026-07-30); hardware described only as "datacenter-class" | Not stated; not audited |
| LadybugDB, SQLite as a graph store, Neo4j embedded | None found with hardware | Not applicable |

## 3. Workloads

| ID | Workload | What it measures |
|---|---|---|
| B-1 | Bulk load into an empty store, made durable at the end | Edges per second; time to durable |
| B-2 | One mutation per commit at matched durability, with 1, 4 and 16 concurrent writers | Commit latency (p50, p99, p999) and throughput |
| B-3 | Batched commits of 1, 16, 256 and 4,096 mutations | Throughput against batch size |
| B-4 | LinkBench's default operation mix (`config/FBWorkload.properties`): getlinklist 50.71%, getnode 12.93%, addlink 8.99%, updatelink 8.01%, updatenode 7.37%, countlink 4.89%, deletelink 2.99%, addnode 2.57%, deletenode 1.01%, getlink 0.53%. About 69% reads and 31% writes, with Zipf-distributed access | Throughput and latency per operation |
| B-5 | Out-neighbours of a vertex, and two-hop expansion, from random start vertices, separately for hubs and low-degree vertices | Read latency |
| B-6 | Analytics on a consistent snapshot, using the algorithms of LDBC Graphalytics: BFS, PageRank, weakly connected components (WCC), community detection by label propagation (CDLP), local clustering coefficient (LCC), single-source shortest paths (SSSP). Stage 1 adds SCC | Time per algorithm, for systems that offer each natively; results checked against Graphalytics' reference outputs |
| B-7 | Recovery: kill the process during B-2, then reopen | Time to reopen and replay; the recovered state must equal a prefix of the shadow model holding every acknowledged commit |
| B-8 | Footprint after load and after checkpoint or compaction | Bytes on disk per edge; resident memory; cold open time |
| B-9 | Write latency while a checkpoint runs | Checkpoint interference |

**Why B-2 matters most.** Group commit (S1-LOG-4) should show its advantage under concurrent writers.
A store that syncs every commit separately waits one barrier per commit. Among the set, LadybugDB,
SurrealDB and Neo4j group their commits, while SQLite forces each commit separately, and so does
ArcadeDB, as far as was found.

**The workloads' origins:**
- **LinkBench** (`facebookarchive/linkbench`, Apache-2.0) is archived; its last commit was on
  2015-12-11, and no maintained fork exists. It is Java with no pre-generated data: a load phase
  builds the graph, then a request phase runs. B-4 reimplements its operation mix in the Rust harness
  and credits LinkBench as the source of the mix.
- **LDBC SNB Interactive.** Only v1 can be audited; the v2 chapter says "As of January 2024,
  commissioning audits for this workload is not yet possible." Pre-generated datasets from SF0.1 to
  SF1000 come with update streams at ldbcouncil.org.
- **LDBC Graphalytics.** The driver is v1.10.0 (2023-04-12) and the specification v1.0.5. Datasets run
  from XS to 2XL+ and include reference outputs.

**Datasets:**
- **Kronecker (R-MAT) graphs** from the Graph500 generator, at scales 20, 23 and 26. GraphOne uses
  this generator for its synthetic graphs.
- **LDBC SNB datasets** at SF1 and SF10, and **Graphalytics datasets** in sizes XS to M, used under
  the fair-use rules of §1.
- **A DeepBrain-shaped synthetic graph** from an in-house generator, with keyed parallel edges, ULID
  identities and provenance payloads. No standard dataset has keyed parallel edges with validity
  windows.

## 4. Durability parity and crash checks

**Matched durability** means every acknowledged commit survives power loss.

- **macOS** needs `F_FULLFSYNC`. What each storage engine issues there:

  | Engine | Barrier on macOS | Evidence |
  |---|---|---|
  | Rust std (`File::sync_all`, `sync_data`) | `F_FULLFSYNC` | `library/std/src/sys/fs/unix.rs`, `fn fsync` |
  | LMDB | `F_FULLFSYNC` (`MDB_FDATASYNC` under `__APPLE__`) | `libraries/liblmdb/mdb.c` on `mdb.master` (commit `2a3df93002`) |
  | JDK (`FileChannel.force`) | `F_FULLFSYNC`, falling back to `fsync` only on non-local mounts | JDK-8080589, fixed in JDK 9; `FileDispatcherImpl.c` |
  | LadybugDB | `F_FULLFSYNC` on the file and its directory, when built with `HAS_FULLFSYNC` | `local_file_system.cpp`, `CMakeLists.txt` at commit `f003884` |
  | SurrealKV | `F_FULLFSYNC` (through Rust's `sync_all`) | `surrealdb/kvs/src/config.rs` at v3.3.0 |
  | SQLite | `fsync` unless `PRAGMA fullfsync=ON` | sqlite.org PRAGMA documentation; `os_unix.c` |
  | DuckDB 1.5.6 | `fsync`; `HAVE_FULLFSYNC` is defined nowhere in its tree | `src/common/local_file_system.cpp` at v1.5.6 |
  | RocksDB built by Rust crates | `fsync`, even with synchronous writes. RocksDB uses `F_FULLFSYNC` only with `HAVE_FULLFSYNC`, which only its own CMake and Makefile define | `env/io_posix.cc` at `a215c38a8e`; `rust-rocksdb` at `46d7e4f10d`; Oxigraph's `oxrocksdb-sys` at `2283f42a7e`; SurrealDB's RocksDB fork at `6f0541d` |

  - SQLite reaches matched durability on macOS with `PRAGMA fullfsync=ON`, plus `synchronous=FULL` in
    WAL mode.
  - A system that cannot issue `F_FULLFSYNC` on macOS gets matched-durability numbers from Linux
    only. Today that covers DuckDB 1.5.6 and every RocksDB backend built by a Rust crate.
- **Linux NVMe.** Report the drive model and whether it has power-loss protection; with it, syncs
  become cheap.
- **Crash classes and tools.** Killing the process tests only durability across a process crash.
  Power-loss behaviour needs a layer that discards unsynced writes:
  - **LazyFS** (`dsrhaslab/lazyfs`, release 0.3.1 of 2026-05-07, maintained) is a FUSE filesystem
    with its own page cache.
    - It can drop unsynced writes on command and inject torn writes.
    - Its paper appeared at PVLDB 17 (2024), and Jepsen has used it in several analyses.
    - It runs on Linux.
  - **dm-log-writes and dm-flakey** are Linux device-mapper targets. dm-log-writes records writes up
    to each flush, so a replay shows what reached the disk.
  - ALICE (last commit 2018) and CrashMonkey (last commit 2022, and it tests filesystems) are
    dormant. Pathfinder (PACMPL 2025) is a newer application-level tool.
  - Every result states which crash classes were tested for each system.
- **Controls.** Configurations that acknowledge commits before they are durable should lose
  acknowledged commits under LazyFS. Losing them shows the harness detects loss:
  - ArcadeDB with `txWalFlush=0`;
  - Grafeo in `Batch` mode;
  - SQLite in WAL mode with `synchronous=NORMAL`.

## 5. Measurement protocol

- **Runs.** Three runs per row, reporting mean and spread, and the median when the spread exceeds
  15%. This is the rule of UltraGraph's own benchmark records
  (`openspec/notes/ultragraph/BenchBefore/BenchBefore.md`).
- **Machines.**
  - The benchmark machine (Apple M3 Max, 16 cores, 128 GB, APFS) for development runs.
  - A Linux host with NVMe for published comparisons, naming the instance type and the drive.
- **Harness.**
  - One Rust harness drives every Rust and C system through its native or C API, and a Java harness
    of the same design drives Neo4j and ArcadeDB.
  - A no-op backend measures each harness's own overhead.
  - Versions are pinned and configurations are kept with the results.
- **Correctness.** After each workload, the harness compares each system's graph against the shadow
  model's digest (S1-APP-5, S1-SIM-8).

## 6. Open items

1. Whether LadybugDB's prebuilt macOS library behind `lbug` is built with `HAS_FULLFSYNC`.
2. Whether SurrealDB's embedded SDK reads `SURREAL_DATASTORE_SYNC`, or needs the setting in its
   connection string.
3. Neo4j's order-form terms, if a commercial edition is ever compared.
4. The Linux host for published runs.

## Sources

All read 2026-10-16. Commit identifiers give the revision read.

- SQLite: [PRAGMA](https://www.sqlite.org/pragma.html), [isolation](https://www.sqlite.org/isolation.html),
  [WAL](https://www.sqlite.org/wal.html); `os_unix.c`.
- LadybugDB: [LadybugDB/ladybug](https://github.com/LadybugDB/ladybug) at `f003884`, identical in tag v0.21.2:
  - `src/storage/wal/wal.cpp`, the commit path and leader/follower group commit;
  - `src/common/file_system/local_file_system.cpp`, the barrier;
  - `CMakeLists.txt`, the `HAS_FULLFSYNC` detection;
  - `transaction_manager.cpp` and `database.h`, the single writer.
- Grafeo: [GrafeoDB/grafeo](https://github.com/GrafeoDB/grafeo) at `a966850`:
  - `crates/grafeo-engine/src/config.rs`, the durability default;
  - `crates/grafeo-storage/src/wal/log.rs`, the WAL sync;
  - `CHANGELOG`, #498;
  - `docs/ecosystem/performance.md`;
  - [GrafeoDB/graph-bench](https://github.com/GrafeoDB/graph-bench).
- SurrealDB: `surrealdb/kvs/src/config.rs` at v3.3.0 (`238bfeb`);
  [benchmarks](https://surrealdb.com/benchmarks).
- DuckDB: `src/common/local_file_system.cpp` at v1.5.6;
  [PR #25784](https://github.com/duckdb/duckdb/pull/25784);
  [DuckPGQ](https://github.com/cwida/duckpgq-extension);
  [CIDR 2023 paper](https://www.cidrdb.org/cidr2023/papers/p66-wolde.pdf).
- Neo4j: tag 2026.09.0, `BatchingTransactionAppender` and `TransactionLogFile`;
  [Online Trial Agreement](https://neo4j.com/terms/online-trial-agreement/).
- JDK: [JDK-8080589](https://bugs.openjdk.org/browse/JDK-8080589).
- ArcadeDB: `GlobalConfiguration.java` at `e944f1bc`; [benchmarks](https://arcadedb.com/benchmarks.html).
- HelixDB: [HelixDB/helix-db](https://github.com/HelixDB/helix-db) at `22ed0fef`.
- macOS barriers of shared engines:
  - [rust-lang/rust](https://github.com/rust-lang/rust), `library/std/src/sys/fs/unix.rs`;
  - [LMDB](https://github.com/LMDB/lmdb/blob/mdb.master/libraries/liblmdb/mdb.c);
  - [facebook/rocksdb](https://github.com/facebook/rocksdb) at `a215c38a8e`, `env/io_posix.cc`,
    `CMakeLists.txt`, `build_tools/build_detect_platform`;
  - [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb) at `46d7e4f10d`;
  - [oxigraph](https://github.com/oxigraph/oxigraph) at `2283f42a7e`.
- Repository status: GitHub and crates.io, for every system in §2.
- The durability barrier on the benchmark machine: `PersistentStore.md`, Appendix A.
- GraphOne's use of the Graph500 generator: Kumar and Huang, ACM TOS 15(4), 2019, §7.
- LinkBench: [facebookarchive/linkbench](https://github.com/facebookarchive/linkbench),
  `config/FBWorkload.properties`.
- LDBC:
  - [fair-use policies](https://ldbcouncil.org/benchmarks/fair-use-policies);
  - [SNB Interactive](https://ldbcouncil.org/benchmarks/snb/interactive/);
  - [Graphalytics algorithms](https://ldbcouncil.org/benchmarks/graphalytics/algorithms/);
  - [FAQ](https://ldbcouncil.org/resources/faq).
- Crash tools:
  - [LazyFS](https://github.com/dsrhaslab/lazyfs);
  - Linux kernel documentation for
    [dm-flakey](https://docs.kernel.org/admin-guide/device-mapper/dm-flakey.html) and
    [dm-log-writes](https://docs.kernel.org/admin-guide/device-mapper/log-writes.html);
  - [Pathfinder](https://github.com/efeslab/Pathfinder).
