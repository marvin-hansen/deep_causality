# A persistent UltraGraph store: one ordered log of mutations, applied deterministically

* Date: 2026-10-16
* Scope: a persistent hypergraph store built on UltraGraph, developed in its own repository outside
  `deep_causality`, from an embedded store on one machine to a cluster holding a 100 TB graph.
* Grounded in: UltraGraph's source and benchmark records in this repository (paths below are relative
  to the repository root); Apache Iggy's source at commit `fe3b4269` (2026-10-15); the GraphOne and
  Calvin papers, read in full; the documentation, source code and issue trackers listed under
  Sources, read on 2026-10-16; two measurements on the benchmark machine (Appendices A and B).
* Decided by Marvin on 2026-10-16: the store lives in a dedicated repository; one ordered log of
  graph mutations, applied deterministically, is the source of truth; one shard follows TigerBeetle's
  design; a commit targets single-digit milliseconds, and microsecond latency is outside the goal; the
  end goal is a cluster holding a 100 TB graph; compute and storage separate at that scale; upgrades
  and backups run while the cluster serves; the stage-1 store is a directory.

## Summary

1. **The commit protocol sets consistency and speed.** One totally ordered log of graph mutations,
   applied by a single deterministic writer, makes every write strictly serializable with no
   conflict checker. Group commit makes it fast: one durability barrier covers many commits.
2. **The writer is cheap; the barrier is expensive.** UltraGraph applies a keyed update in 2.6 ns and
   an in-place Live insert in 25 ns. One append made durable with `F_FULLFSYNC` takes 5.5 ms on the
   benchmark machine at any size from 64 B to 1 MB, so a store that syncs each commit on its own stops
   near 180 commits per second.
3. **Lance serves export, the cold tier and large values.** Its transactions hold one operation on one
   table, its local commits are never fsynced (issue #9608), and each commit writes a data file, a
   transaction file and the whole manifest.
4. **GraphOne contributes the read view**: an adjacency snapshot plus a log offset. Its log reaches
   disk asynchronously without an fsync guarantee, and its authors write that it "cannot compete
   against workloads that require transaction support".
5. **TuringDB stores immutable versioned parts under snapshot isolation, with no log.** Its
   first-committer-wins check allows write skew, a commit is visible before it is durable, and its
   licence is the Business Source License 1.1.
6. **Iggy has the right shape and is too young to depend on.** It runs one VSR group per partition on
   single-threaded shards, covers up to 256 prepares with one durability barrier, and ships a
   deterministic simulator. Its clustering shipped in 0.9.0 on 2026-09-18; a log-divergence bug is
   open (#4284), every partition is replicated on every node, and one primary takes all writes.
7. **TigerBeetle is the specification for the write path.** It reaches its throughput with one core,
   one consensus group and large batches, and it does not shard. For a trillion transactions, about
   128 TiB of primary data by its own estimate, it moved its LSM's lower levels to object storage and
   kept the hot working set in one unsharded replicated state machine.
8. **At 100 TB, tiering holds the data.** One write group keeps the log and the upper levels on NVMe
   and the lower levels in object storage, and analytics scale out on compute nodes that read the
   same immutable segments. Size alone does not require sharding writes.
9. **The 2D edge grid is reserved for write throughput.** Once one core's write rate is exceeded,
   placing each edge on exactly one shard keeps the common write single-shard, and a vertex spreads
   over at most 2·√S shards. Only a transaction that spans shards then pays a protocol: Calvin's
   deterministic one, with no two-phase commit.
10. **Compute and storage separate.** The log stays on a replicated NVMe tier, every other byte is an
   immutable object in object storage, and the log names each manifest. A live upgrade and a live
   backup each become one sequenced operation.
11. **Stage 1 is a directory** whose immutable files map one to one onto object-storage keys.

## 1. The store serves UltraGraph's states, the context contract and DeepBrain

UltraGraph is an in-memory graph with three states (`ultragraph/src/types/storage/graph_state/mod.rs`):
Dynamic for mutation, Live for mutation with algorithms on current data, and Frozen, a CSR for batch
analytics. It is single-owner, deterministic and `no_std`-capable, and its only dependency is
`ultragraph_traits`. Versioned reads, concurrent readers and persistence belong to a storage layer
built on it (`openspec/notes/ultragraph/RadixGraphUpdate.md`, Summary item 6). That note's Later
section names the layer's shape: "the replicated log and checkpoints of frozen CSR segments, in the
storage layer". This note designs that layer.

Three sources set its requirements.

- **UltraGraph's own measurements**
  (`openspec/notes/ultragraph/BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`; Apple M3 Max,
  16 cores, 128 GB; three runs each):
  - a keyed update on a low-degree node takes 2.60 ns;
  - an insert into a Live row with spare capacity takes 24.8 ns, and a row rebuild 0.79 µs;
  - SCC on the 1,000-node chain takes 44.2 µs on Live rows and 46.1 µs on the frozen CSR;
  - `freeze` from Live takes 7.0 µs and `freeze_clone` from Live 9.7 µs on that chain;
  - SCC on the 100,000-edge hub takes 1.04 ms in the Live state;
  - the `Vec` lists hold 49.0 bytes per edge on the hub and 176.0 on the chain.
- **The context persistence contract** (`deep_causality_context_store/src/traits/`).
  `ContextStorageStream::subscribe` returns "the context as of a consistent point, and every change
  to it from that point on", resumed from an opaque cursor. `apply_batch` performs a sequence of
  events "as one transaction". A log position is that cursor, a checkpoint plus the log after it is
  that subscription, and one log record is that transaction.
- **DeepBrain**, which UltraGraph is meant to express:
  - edges carry their own identity (a ULID as `u128`), provenance and a validity window;
  - several edges of one relation may join one pair;
  - identities are never reissued;
  - scoped questions (a zone, validity at a time) read projections;
  - capture and analysis run at the same time.

The goal of this study is the strongest consistency available together with the highest read and
write throughput. The latency target is single-digit milliseconds per commit.

## 2. One ordered log with deterministic apply gives strict serializability, and group commit gives speed

**Mechanism.**
- A client's mutations reach one sequencer, which orders them into a log of checksummed batch
  records.
- A replica applies the log in order on one thread.
- Apply is deterministic: the same log prefix yields the same state on every replica. A crashed or
  new replica recovers by loading a checkpoint and replaying the records after it.

Writes are strictly serializable: the log order is the serial order, and a request submitted after
another's acknowledgement lands later in the log. Reads are strictly serializable when they go
through the log, or through the primary at its commit point.

**Invariants become checks.** Under snapshot isolation, an invariant that reads one key and writes
another breaks when two writers run at once and each sees the other's key unchanged (write skew).
Systems that offer snapshot isolation guard such an invariant by forcing a conflict on a chosen key.
Under a total order, the invariant is a plain check inside apply.

**A single writer is fast when its apply is cheap.**
- LMAX's business-logic processor handles "6 million orders per second on a single thread" (a 2011
  benchmark on a dual-socket Nehalem server).
- TigerBeetle "uses a single core by design".
- UltraGraph applies a mutation in 2.6 to 25 ns.

The serial paths that are slow do expensive work per commit:
- **Dgraph.** Its Alpha loop applies Raft entries one at a time: "no two transactions ever execute
  concurrently" (Dgraph PR #9762). Its Zero leader measured about 70,000 timestamps per second
  (Dgraph staff, February 2020). Marvin's own Dgraph benchmark on the benchmark machine topped out
  near 11,000 commits per second; it was not reproduced for this note.
- **Neo4j.** It appends commits under a lock on the log file and forces the log in batches
  (`BatchingTransactionAppender`).

**Durability needs batching.** Appendix A measures one append followed by a durability barrier on the
benchmark machine:

| Barrier | 64 B | 4 KiB | 1 MiB |
|---|---:|---:|---:|
| none | 4.6 µs | 7.9 µs | 73 µs |
| `fsync` | 25 µs | 41 µs | 249 µs |
| `F_FULLFSYNC` | 5.52 ms | 5.55 ms | 6.42 ms |

(Medians.) On macOS, `fsync` hands the data to the drive without flushing the drive's cache;
`F_FULLFSYNC` flushes it. Rust's `File::sync_all` and `File::sync_data` both issue `F_FULLFSYNC` on
Apple platforms (`library/std/src/sys/fs/unix.rs` in the installed stable toolchain).

The barrier costs the same for 64 bytes as for a megabyte, so it has to be shared:
- **One barrier per commit** stops near 180 commits per second.
- **One 1 MiB group** of 48-byte mutation records holds about 21,800 records. At 6.4 ms per barrier
  that is about 3.4 million durable mutations per second at about 6 ms latency. This is computed from
  the measured barrier and an assumed record size; it is not a measurement.

Linux NVMe drives with power-loss protection were not measured.

Published durable-commit numbers show that group commit costs little:
- SiloR (OSDI 2014) commits 8.76 million durable transactions per second with 40 ms epochs, 80% of
  its throughput with logging off.
- GTX (SIGMOD 2025) puts synchronous logging at about 20% (workload not stated).

**Reads come from snapshots, and rows carry no timestamps.**
- GraphOne composes a static view from the latest adjacency snapshot and the log entries after it.
  UltraGraph does the same with a frozen checkpoint plus the log after it, or with an `Arc`-shared
  frozen snapshot published every N operations.
- Keeping versions out of the rows keeps scans fast. LiveGraph checks a timestamp on every edge it
  reads, and the Teseo errata (Table 3, graph500-24) measure its BFS at 19.4× and its PageRank at
  3.7× the time of a static CSR.

## 3. The candidates for the storage format

| Candidate | Consistency | Cost of a small write | Durability | Role |
|---|---|---|---|---|
| Lance 13.0 | Snapshot isolation per table; one operation per transaction; no transaction across tables | A data file, a `.txn` file and a full manifest rewrite per commit | No fsync on local commits | Export, cold tier, large values |
| GraphOne (TOS 2019) | Snapshot isolation for read-only views; each update atomic; no multi-operation transactions | 36–83 M edges/s logged per thread, in memory | Asynchronous log, no fsync guarantee; recovery replays from an upstream source | The read-view model |
| TuringDB 3.0 | Snapshot isolation, first committer wins per node or edge; write skew possible | No write benchmark published | No WAL; visible before durable | Ideas only (BSL 1.1) |
| Mutation log + deterministic apply + checkpoints | Strictly serializable | One record; one shared barrier per group | One barrier per group | The source of truth |

### 3.1 Lance fits export, the cold tier and large values

Facts as of `lance` 13.0.0 (2026-10-07) and the repository's `main` at `d1ba47e1` (2026-10-15):

- **Format.**
  - File formats 2.0, 2.1 and 2.2 are stable.
  - A random read costs one I/O for a fixed-width value and two for a variable-width one.
  - The format paper (arXiv 2504.15247) finds Lance 2.1 tying or beating Parquet on random access and
    "equal to or greater than Parquet" on full scans. The "100x faster than Parquet" claim compares
    against Parquet's defaults; against tuned Parquet the result is close to a tie.
- **Transactions.**
  - Versions form a linear chain per table; readers see a pinned snapshot.
  - Conflicts are detected between writes, by fragment or row, and read sets are never tracked, so
    write skew is possible.
  - `transaction.proto` declares `oneof operation`: a transaction holds one operation.
  - Nothing in the specification or the source commits across tables, so adding a node and an edge
    stored in two tables cannot commit together.
- **Commit cost.**
  - Every commit writes new data files, a `.txn` file and the whole manifest. Open issue #5194 tracks
    commits slowing as fragments accumulate.
  - Lance's own benchmark (Lance 7.0.0, c7i.48xlarge): 140 ms per commit on S3, 27.6 ms on S3 Express,
    and about 5.7 commits per second with concurrent writers on S3.
  - On the benchmark machine (Appendix B), a single-row append took 1.87 ms at the median after 300
    commits, rising to 5.40 ms after 5,000, with no fsync involved.
- **Durability.**
  - Lance's source contains no `fsync`, `sync_all` or `sync_data`.
  - The local store is `object_store`'s `LocalFileSystem`, whose `with_fsync` option is documented
    as "disabled by default".
  - Open issue #9608 reports zero-length manifest, `.txn` and data files on ext4 after an unexplained
    machine stop.
- **Footprint.**
  - 373 transitive crates with default features off, 583 with them on, including DataFusion,
    `arrow`, `object_store` and `tokio`.
  - Thirteen breaking major releases between 1.0.0 (2025-12-18) and 13.0.0.
- **Streaming writes.** The MemWAL write path is marked experimental and has an open data-loss bug
  (#8293).
- **Graph layer.** `lance-graph` 0.5.4 pins `lance` 1.0.0, maps Cypher onto DataFusion joins and
  builds its CSR in memory; the format has no on-disk adjacency.

Lance has two roles in this design:
- **Export.** A frozen checkpoint becomes one Lance commit, readable by DuckDB or DataFusion.
- **A `Substrate` backend for large values.** `deposit` is idempotent per node, so a value can be
  written to Lance before the log commits the reference to it.

### 3.2 GraphOne contributes the read-view model

GraphOne (Kumar and Huang, ACM Transactions on Storage 15(4), 2019, the extended version of the
FAST 2019 paper) combines two structures:
- a small circular edge log of recent updates;
- an adjacency store with a multi-versioned degree array.

Edges move from the log to the adjacency store in batches ("archiving") once 2^16 edges are pending.
A static view combines the latest adjacency snapshot with the log entries after it, giving snapshot
isolation for analytics.

- **Transactions.** Each update is atomic; there are no multi-operation transactions. §7.3.3:
  "GraphOne offers snapshot isolation and not the strict serializability required for transaction
  support, hence GraphOne cannot compete against workloads that require transaction support."
- **Durability (§5.1.3).**
  - The edge log is "periodically appended to a durable file in a separate thread context", and this
    "will not guarantee durability unless fsync() is called".
  - The durable log is a prefix of the ingested data, so "GraphOne may lose some recent data in the
    case of an unplanned shutdown". Recovery replays the lost data from an upstream source such as
    Kafka.
- **Numbers (Table 8;** two 14-core Xeon E5-2683, 512 GB, Samsung 950 Pro NVMe):
  - logging: 36–83 M edges/s per thread;
  - archiving: 29–78 M edges/s;
  - recovery from the log: 30–181 M edges/s in memory.
- **No on-disk format.** GraphOne is an in-memory structure. Its code is GPL-3.0 C++, last pushed on
  2021-01-05.

### 3.3 TuringDB keeps immutable parts and snapshot isolation, with no log

TuringDB's documentation and its C++ source (commit `fc9539336`, 2026-10-15) describe:

- **Commits.** "Every graph in TuringDB is stored as a sequence of immutable commits, much like Git.
  Each commit is composed of DataParts." A snapshot is a reference-counted pin on one commit's list
  of DataParts.
- **Concurrency.** A "Change" is a private branch held in memory.
  - An exclusive lock serialises submits, and a submit rebases the Change if the main branch has
    moved.
  - Conflict checks compare write sets per node or edge, and the first submitter wins. No read set
    is tracked, so write skew is possible.
- **Durability.** The documentation states "No WAL". A submit publishes the new head in memory before
  `dumpGraph` writes it to disk, so a visible commit can be lost in a crash.
- **Storage costs.**
  - Reads walk DataParts from newest to oldest until the manual `MERGE_DATAPARTS` runs.
  - Every commit copies the previous commit's whole tombstone set.
- **Benchmarks.** The technical report (2026-02-16) measures 48 read queries, cold and sequential,
  51.8× faster than Neo4j on average. No write throughput is published.
- **Licence.** Business Source License 1.1, converting to Apache-2.0 on 2029-11-25.

### 3.4 The research stores for dynamic graphs mostly run without durability

| System | Isolation | Durability |
|---|---|---|
| LiveGraph (PVLDB 2020) | Snapshot isolation | WAL with group commit and checkpoints. The public code holds no recovery path, and its `write`/`fdatasync` failure branches construct an exception without throwing it |
| GTX (SIGMOD 2025) | Snapshot isolation; optional serializable mode at 3–25% cost | Group redo logging, synchronous or asynchronous; per-worker checkpoints |
| BACH (PVLDB 2025) | Snapshot isolation | WAL persisted at commit |
| LSMGraph (PACMMOD 2024) | Fine-grained snapshot isolation | No WAL |
| Aspen, Teseo, Sortledton, RapidStore, RadixGraph | Varies | In memory only |

Every comparison found in these papers ran the competing systems with logging off, so their
throughput figures exclude durability.

LiveGraph reaches 8.77 M requests per second on the TAO workload in memory, against 3.24 M for LMDB
(two 12-core Xeon Gold 6126, 192 GB). The authors identify the WAL as what stops its write-heavy
workload from scaling. RadixGraph's authors call stronger isolation for their structure "an open
problem".

### 3.5 The production graph databases trade consistency or durability for speed

- **Memgraph.**
  - Snapshot isolation at most; no isolation level it offers prevents write skew.
  - Its default `--storage-wal-file-flush-every-n-tx=100000` fsyncs the WAL once per 100,000
    transactions.
  - STRICT_SYNC replication uses two-phase commit.
- **Kuzu.** Serializable because it allows only one writer. It fsyncs the WAL on every commit with no
  group commit. Its repository was archived on 2025-10-10.
- **Neo4j.** Read committed by default. One leader per database takes all writes, and commits use
  group commit.
- **Dgraph.** One Raft group per predicate group, with timestamps and conflict detection at the Zero
  leader. Jepsen (1.1.1, 2020) found snapshot isolation that is "not linearizable in general", with
  lost writes during tablet moves.
- **NebulaGraph.**
  - One Raft group per partition.
  - Each edge is stored in both endpoints' partitions. According to the 3.x documentation (seen in a
    search excerpt, not fetched), its TOSS mechanism keeps the two copies only eventually consistent.
  - It reaches large scale by giving up atomic edges across partitions.
- **FalkorDB.** Asynchronous Redis replication.
- **TigerGraph.** Read committed, according to its current documentation.

None of them documents strict serializability on a single node.

## 4. Iggy has the right shape for the server and the cluster

From Iggy's source at `fe3b4269`:

- **One VSR group per partition.** A packed namespace maps each (stream, topic, partition) to one
  consensus group, one shard and one directory (`core/server_common/src/sharding/namespace.rs`).
  Group 0 is the metadata plane.
- **Single-threaded shards.** Consensus runs on single-threaded compio shards, and its futures are
  deliberately `!Send` (`core/consensus/src/plane_mux.rs`).
- **Group commit.** One durability barrier covers up to 256 prepares or 8 MiB
  (`core/partitions/src/persistence.rs`).
- **Pipeline.** Up to 32 prepares are in flight and 64 requests queued (`core/consensus/src/impls.rs`).
- **The metadata plane is a deterministic state machine.** `StateHandler::apply` must return a "pure
  function of `(state, command)`", and the primary's timestamp in the prepare header is the only
  clock a handler may read (`core/metadata/src/stm/mod.rs`). Readers on other shards get left-right
  copies.
- **The metadata WAL fsyncs each append before its PrepareOk** (`core/journal/src/prepare_journal.rs`).
- **Every group spans every replica.** A group's membership is `ReplicaTopology { self_replica_id,
  replica_count }`, built from the cluster topology (`core/shard/src/lib.rs`,
  `core/server/src/boot/recovery.rs`). No per-partition replica set exists.
- **Every server crate is unpublished.** All are `publish = false`. `partitions`, `server_common` and
  `message_bus` contain `unsafe` (79, 53 and 13 matches); `consensus` and `shard` contain none.
- **A deterministic simulator** (`core/simulator`) injects network faults, crashes, power loss and
  torn writes.

From Iggy's documentation and issue tracker:

- **Release.** Clustering shipped in server 0.9.0 (2026-09-18). It is pre-1.0 and off by default. The
  protocol is Viewstamped Replication Revisited with TigerBeetle's capped flexible quorums.
- **Durability policies.** `replicated`, the default, acknowledges after quorum commit and local apply
  with no stable-storage barrier: "Success therefore does not prove that the operation reached stable
  storage on any replica." `persisted` acknowledges only once copies are on stable storage at the
  quorum.
- **Benchmarks.** 0.9.0, three nodes, 800 MB/s, AWS i4i, `persisted`: p50 2.91 ms, p99 7.07 ms,
  p999 12.40 ms. A later snapshot brings p999 to 3.89 ms; upstream calls it work in progress.
- **Reads.** Followers serve reads from local state, so those reads are not linearizable.
- **Open issues.**
  - #4284: a replica commits a prepare that was never acknowledged, and committed logs diverge.
  - #4438: state transfer never completes after rolling restarts.
  - #4294: the metadata primary panics.
  - #4436: multi-second stalls.
- **Writers and membership.** "All writes go through a single primary today"; multi-leader is in
  progress. Membership cannot change.

Each partition is an independent group, and nothing commits atomically across groups. An edge between
two partitions would need a protocol that Iggy lacks.

Iggy is the template for the Rust idioms, the superblock procedure, the WAL repair rule and the
simulator, and its per-partition directories match the stage-1 layout (§10). Reassess it as a
dependency after its 1.0 release.

## 5. Aeron and LMAX contribute the node pipeline and the shadow replica

**What LMAX and Aeron do.**
- **LMAX** ([Fowler](https://martinfowler.com/articles/lmax.html)):
  - The journaller, replicator and unmarshaller read the input ring buffer in parallel, and "all of
    them need to be done before the Business Logic Processor works on a message".
  - The processor is single-threaded and in memory. It recovers from a snapshot plus a replay of the
    journal, in under a minute.
  - Three processors, two in the main data centre and one at the recovery site, process every event.
  - The odd/even split divides journalling work between two journallers.
- **Aeron Cluster** runs Raft and "sequences multiple client connections into a single, replicated
  log". Its services consume the log once a majority of members have recorded it. Its sharding
  guidance:
  - "don't shard", which it calls "by far the simplest";
  - several services on the same log, where "Each service receives the same message, and must decide
    for itself if the message should be consumed or ignored."

**What the store takes from them.**
- **The node pipeline:** journal and replicate in parallel, then apply on a pinned core.
- **Shadow replicas for analytics.**
  - A replica pauses apply at op N, runs the algorithm on its Live rows in place, then resumes and
    catches up.
  - A paused deterministic replica is a consistent snapshot at N by construction.
  - At 25 ns per in-place insert, a backlog of one million mutations replays in about 25 ms plus
    decoding (an estimate).
- **Routing by owner.** A consume-or-ignore filter must key on the vertex that owns the data. A
  modulo on the message identifier would spread one vertex's writes over several nodes.

**What it leaves aside.**
- **Microsecond latency.** It needs kernel bypass, replication inside one rack and no fsync on the
  commit path.
- **The single log.** One log caps total write bandwidth and sends the whole stream to every
  consumer.

## 6. Calvin orders transactions once and needs no two-phase commit

Calvin (Thomson et al., SIGMOD 2012), read in full:

- **Sequencing.** Time is divided into 10 ms epochs. Every machine runs a sequencer that collects the
  epoch's requests into a batch and replicates it. Each scheduler then "piece[s] together its own view
  of a global transaction order by interleaving (in a deterministic, round-robin manner) all
  sequencers' batches for that epoch".
- **Replication.** Calvin replicates transaction inputs, asynchronously or through Paxos. "Total
  transactional throughput was not affected by changing Calvin's replication mode", including across
  data centres with 100–170 ms between replicas.
- **Execution.** A transaction that spans partitions runs in five phases:
  1. read/write set analysis;
  2. local reads;
  3. forwarding of the local reads to the active participants;
  4. collection of the remote reads;
  5. execution and applying the writes.

  No two-phase commit runs. A failed node cannot abort a transaction, because a replica replays the
  same input.
- **Transactions whose write set depends on a read** use OLLP: a reconnaissance read first, then a
  sequenced transaction that re-checks the read and restarts if it changed.
- **Results.** "Half a million TPC-C transactions per second on a cluster of commodity machines",
  about 5,000 per node at 100 nodes.

Calvin's insight is that agreement happens once, on the order of inputs. Execution then needs no
agreement, so no lock is held across a network round trip. §8 applies this to the 100 TB cluster.

## 7. TigerBeetle is the specification for the write group

**TigerBeetle's documentation:**
- **Performance page.** "TigerBeetle uses a single core by design and uses a single leader node to
  process events. Adding more nodes can therefore increase reliability, but not throughput."
- **Against sharding:** "a small number of hot accounts are often involved in a large proportion of
  the transactions", so "the shards responsible for those accounts become bottlenecks".
- **Batching.** Requests carry batches of up to 8,190 transfers (the homepage says 8,189), and "the
  cost of replication is paid only once per batch".
- **Throughput claim.** The homepage claims "100K-500K TPS" and "100ms P100" latency, without
  hardware. Marvin cites one million transactions per second from TigerBeetle's trillion-transaction
  run; the post on that run (2026-03-19) gives no rate and points to the talk for the figures.
- **Scale by tiering.** For that run TigerBeetle "spun up a TigerBeetle cluster with local NVMe plus
  object storage with half a petabyte of flash" and "extended TigerBeetle's LSM engine with a
  connector to object storage for the lower levels", keeping the hot working set in one unsharded
  replicated state machine. A trillion transactions at about 128 bytes each is "around 128 TiB, just
  in storage, before indexing, before replication", by the post's own estimate. The post argues that
  sharding harms cache locality and that contention bounds throughput under Amdahl's law.
- **Jepsen** (0.16.11, 2025-06-06): it "appeared to meet its promise of Strong Serializability" as
  of 0.16.30, with caveats on recovery from lost disks before 0.16.43.
- **Data file and execution** (`docs/internals/data_file.md`, `vsr.md`):
  - one data file holds a WAL ring of 1,024 prepare slots of up to 1 MiB, a superblock in four
    copies, and a grid of 512 KiB blocks holding an LSM forest copy-on-write;
  - a checkpoint every 960 operations swaps the superblock root;
  - execution is deterministic, so replicas hold byte-identical grids and repair each other block by
    block;
  - each batch runs `prefetch`, `commit`, `compact`, `checkpoint`;
  - the primary assigns timestamps.
- **Upgrades.** TigerBeetle "will then coordinate the actual upgrade when all replicas are ready and
  have the latest version available", with about five seconds of unavailability while replicas
  restart. "Data files created by a particular past version of TigerBeetle can be migrated to any
  future version."

**A transfer and an edge have the same shape.** A transfer debits one account and credits another;
adding an edge updates one vertex's forward list and another's backward list, plus one record.

**Graph mutations break three of TigerBeetle's assumptions:**
1. **Variable size.** Records become fixed-size (source, target, key, payload reference, flags; about
   48 bytes), and payloads go to the `Substrate` first.
2. **Unbounded fan-out.** Retracting a hub node touches every incident edge. Retraction becomes O(1):
   a liveness bit flips, and stale entries leave at compaction. The Live state already works this
   way, with a mask bit per node and stale entries dropped when their row is rebuilt.
3. **A lookup before every write.** Change 1's exact `number_edges` and `EdgeNotFoundError` require
   one. In an LSM the lookup reaches the on-disk levels, which is what the prefetch phase and
   per-row filters are for.

## 8. At 100 TB, tiered storage holds the data, and the 2D grid answers write throughput

### 8.1 Tiering holds 100 TB in one write group

TigerBeetle's trillion-transaction run (§7) holds data at that scale in one unsharded group. Its LSM
keeps the hot working set on local NVMe and its lower levels in object storage. The graph store
follows the same shape:

- **One write group.** One VSR group and one deterministic apply core keep strict serializability
  with no protocol across shards.
- **The graph LSM is tiered.**
  - The log, the Live rows and the upper CSR levels sit on NVMe.
  - The lower CSR levels are immutable objects in object storage (§9).
- **Rebuilding a replica copies only the NVMe tiers.** The lower levels are shared, so the copy is a
  fraction of 100 TB.
- **Analytics scale out.** Compute nodes read the shared immutable segments at a checkpoint or epoch,
  without a full local copy and without touching the write path.

Size alone does not require sharding writes. Two limits remain:
- **The write rate of one core.** It is unmeasured (§12).
- **Bulk ingestion.** At one million edge writes per second, loading 10^12 edges takes about 11.6
  days. A bulk-load path removes that limit:
  1. A parallel job builds CSR segments outside the apply loop, checking uniqueness and liveness
     against the manifest it started from.
  2. It uploads the segments.
  3. One sequenced operation installs them.

### 8.2 The write group

- **Shape.** One VSR group of three replicas and one single-core deterministic state machine.
- **Storage engine: a graph LSM.**
  - Level 0 is UltraGraph's Live rows.
  - The levels below are CSR segments, sorted by (source, target, key) going forward and (target,
    source, key) going backward.
  - Entries are fixed-size, and payloads sit in a separate value store.
  - Compaction is deterministic.
- **Tiers.** NVMe holds the log, the upper levels and a block cache; object storage holds the lower
  levels.

### 8.3 Sharding writes is the escape hatch, and graphs are its hardest case

When the write rate exceeds what one core sustains, writes must spread over S groups. Hashing
vertices into S shards splits an edge's endpoints with probability 1 − 1/S: at S = 100, 99% of edge
writes would cross shards. The grid below keeps those writes on one shard.

Each group in the grid is a write group as in §8.2, tiered the same way. Run thread-per-core, 100
groups need about 30 machines with 10 group cores each (an estimate).

### 8.4 A 2D grid keeps each edge on one shard

Arrange the S groups as a √S × √S grid and place edge (u, v) on shard (h(u) mod √S, h(v) mod √S):

```
            column h(v)
          ┌───┬───┬───┐
row h(u)  │   │ e │   │   ← every out-edge of u is in this row
          ├───┼───┼───┤
          │   │   │   │
          └───┴───┴───┘
                ↑ every in-edge of v is in this column
```

- **Writes stay on one shard.** Both the forward and the backward entry of an edge live on its one
  shard. Adding, updating or removing an edge, including the keyed uniqueness check (every edge
  between u and v is on that shard), is one group's commit.
- **Hubs spread out.** A hub's edges fall across √S shards, so no shard becomes the hot account
  TigerBeetle warns about.
- **The spread is bounded.** Spark GraphX's `EdgePartition2D` guarantees "a `2 * sqrt(numParts)`
  bound on vertex replication". At S = 100, a vertex's edges sit on at most 20 shards.
- **Reads pay instead of writes.** A one-hop expansion gathers from √S shards (10 at S = 100). Whole
  analytics follow the 2D pattern of Graph500's partitioned BFS.
- **Growth.** A virtual grid is fixed up front (64 × 64 blocks, for example) and its blocks are
  assigned to groups, so rebalancing moves blocks by state sync without re-hashing.

### 8.5 Vertices: dense identifiers, liveness bits, retraction without fan-out

- **Dense internal identifiers.** A vertex's home shard allocates its internal identifier, never
  reissued. This is change 1's rule at cluster scope. A sharded index maps ULIDs to internal
  identifiers, and clients cache the mapping.
- **Liveness bits.** Each shard holds the liveness bits of the vertices in its row and column, 2/√S of
  all vertices: about 2.5 GB at 10^11 vertices and S = 100. Adding an edge checks both endpoints
  against these bits on the shard itself, with no message.
- **Retraction flips one bit.** The vertex's edges become invisible on every shard at the same epoch,
  and compaction drops them.

### 8.6 Epochs order the transactions that span shards

- **Epochs.** Every group closes epochs of 1–2 ms. Liveness changes from epoch e apply on every shard
  before epoch e + 1, so an edge may reference any vertex live at the previous epoch boundary.
- **A single-shard transaction** commits after its group's consensus round, without waiting for an
  epoch.
- **A multi-shard transaction** takes the path of Calvin and SLOG. Examples: creating a vertex
  together with its edges, or an invariant spanning subjects.
  1. It is sequenced once.
  2. It is placed at the same epoch boundary in every participant's log.
  3. Participants exchange their local reads in one message.
  4. Each executes deterministically.

  No lock is held across a network round trip.
- **Reads across shards** happen at an epoch boundary, which is a consistent cut by construction.
  - Waiting for the current epoch to close gives strict serializability, at a cost of up to one
    epoch.
  - Reading the last closed epoch is serializable and at most one epoch stale.

### 8.7 Mixed load

- **Writes and point reads** run on the apply core of the write group that owns them.
- **Analytics** run on compute nodes that read the shared segments at a checkpoint or epoch N, or on
  standby replicas held at N that catch up afterwards.
- **Whole-graph algorithms need a separate engine.** A 100 TB graph exceeds any one machine's memory,
  and UltraGraph's algorithms assume an in-memory graph. A whole-graph SCC or PageRank needs
  out-of-core scans over the CSR segments or a distributed engine in the BSP style. That engine is
  its own track. Scoped DeepBrain views that fit on one machine can be gathered onto it by
  materialising the projection at epoch N.

### 8.8 Estimates

None of these figures is measured:
- **One write group:** 200,000 to 1,000,000 edge writes per second, the range of TigerBeetle's own
  claims for one core.
- **A grid of 100 groups:** 20 to 100 million per second, provided most writes stay on one shard.
- **Ingestion of 10^12 edges:** about 11.6 days at one million per second through the log, and about
  5.5 hours at 50 million per second across a grid. A bulk load runs at the rate of its parallel
  segment build and upload.
- **Latency:** a single-shard write is one consensus round with fsync, a few milliseconds. A
  multi-shard write adds the wait for its epoch to close plus one message round trip.

## 9. At 100 TB, compute and storage separate

TigerBeetle took the same step for its trillion-transaction run: local NVMe plus an object-storage
connector for the LSM's lower levels (§7).

**Three tiers per write group:**

1. **Log tier: the VSR group.**
   - Three replicas on local NVMe, holding the operations since the last checkpoint.
   - The commit path never touches object storage.
   - Sealed log segments are archived to object storage continuously, for point-in-time recovery.
2. **State tier: object storage.** It holds only immutable, content-addressed objects: CSR segment
   blocks, value objects and per-checkpoint manifests listing segment hashes.
   - No mutable pointer lives there; a log entry names each shard's current manifest ("checkpoint at
     op N is manifest H").
   - The store therefore needs read-after-write consistency for new objects and no conditional
     writes. A store that ignores `If-None-Match` loses acknowledged Lance appends (Lance issue
     #9762); this design never relies on that header.
3. **Compute tier.**
   - Nodes apply the log into Live rows and keep an NVMe block cache.
   - Any compute node can be rebuilt from a manifest plus the log suffix.
   - The VSR replicas still apply the log, with their local disk serving as a cache.

**Compaction runs once.** It is a pure function of immutable inputs. It runs once on any worker, its
output can be checked by hash, and a log entry installs it. TigerBeetle's documented design compacts
on every replica. Separation stores 100 TB once, with object storage supplying redundancy, instead of
300 TB of NVMe.

**The cost is cold reads.** A cache miss costs an object-storage round trip. The prefetch phase hides
it from the apply core: batch N + 1 fetches its blocks while batch N applies. Cache size becomes the
main knob between cost and speed, and the grid keeps each shard's reads inside its own block.

**Live upgrade.**
- **The version is replicated state.** A version change is the sequenced operation "activate V at
  op N". For one release window, binaries carry the old and the new code path, and every replica
  switches at the same op, so determinism holds while versions are mixed.
- **A blue/green handover per shard** removes even TigerBeetle's restart pause:
  1. Start compute nodes on the new version from the latest manifest and log suffix.
  2. Let them catch up as followers.
  3. Hand leadership over at a log position.
  4. Retire the old nodes.
- **Formats.** Every block carries a format version. New code reads old blocks, writes new blocks in
  the new format after activation, and migrates old blocks at compaction.
- **Across groups**, the upgrade runs in two phases:
  1. Binaries that speak both protocol versions roll out group by group.
  2. The new cross-shard protocol activates at an epoch through the metadata group, once every
     group reports ready.
- **Rules borrowed from TigerBeetle.** Each release declares the oldest version it upgrades from, and
  a client is never newer than the server.

**Live backup.**
- **A backup is metadata:** the set of shard manifests at an epoch boundary E, which is a consistent
  cut, plus the archived log from E.
  - Nothing pauses.
  - The segments are immutable, so copying them to another bucket or region runs at leisure.
  - Object lock makes the copy tamper-proof.
- **Point-in-time recovery** loads the newest manifest set at or before the target and replays the
  archived log to the target op.
- **Clones.** A manifest set also starts test clusters and analytics clusters without copying data.
- **Recovery targets.**
  - Data loss: zero within the log tier's fault domain. On region loss, the archival lag, set by how
    often log segments are sealed.
  - Recovery time: compute start, plus cache warm-up, plus replay since the checkpoint.

**Garbage collection** deletes an object only when no retained manifest references it: live
manifests, backups, pinned readers and clones. It marks from those roots, with a grace period.

## 10. The stages, and the stage-1 directory

| Stage | Content |
|---|---|
| 1. Embedded store | One replica in a directory: the graph LSM, deterministic apply, fixed-size records. It measures what one core sustains |
| 2. Single-node server | The stage-1 engine behind a network protocol, with thread-per-core I/O and readers on snapshots |
| 3. Replicated write group | VSR replication and the deterministic simulator. This completes one TigerBeetle-shaped write group and suffices for a graph that fits one machine's disks |
| 4. Tiered storage | Lower levels in object storage, compute nodes on the shared segments, the bulk-load path, live upgrade and live backup. One write group then holds 100 TB |
| 5. Write grid | The grid, liveness bits, epochs and the multi-shard path, built only when one core's write rate is exceeded |
| Separate track | The analytics engine for graphs larger than one machine's memory |

The stage-1 store is a directory. Its immutable files map one to one onto object-storage keys, as
Iggy's per-partition directories do for its logs:

```
<store>/
  superblock.0, superblock.1    # alternating copies: checkpoint op, manifest hash, sequence, checksum
  wal/
    {first_op:020}.wal          # log segments named by first op; one record = one checksummed batch
  segments/
    {hash}.seg                  # immutable CSR runs: forward (src, dst, key), backward (dst, src, key)
  values/
    {hash}.val                  # payload objects, or a Substrate backend
  manifests/
    {op:020}-{hash}.manifest    # segment hashes and levels at a checkpoint op
```

- **Commit.**
  1. Append the batch to the active log segment.
  2. One `sync_data` covers the group.
  3. Acknowledge.
- **Checkpoint.**
  1. Write and sync the new segment files.
  2. Write and sync the manifest.
  3. Append `Checkpoint { op, manifest }` to the log, which stays the single authority.
  4. Rewrite the older superblock copy: write a temporary file, fsync it, rename it over the target,
     fsync the directory. This is Iggy's procedure.
  5. Only then may log segments wholly before `op` be deleted.
- **Recovery.**
  1. Take the valid superblock with the higher sequence.
  2. Load its manifest.
  3. Replay the log from `op + 1`.
  4. Apply Iggy's repair rule to a damaged log: truncate a torn final record only, and refuse to open
     when a complete record follows the damage, because that record was durable and acknowledged.

The store lives in a dedicated repository, decided by Marvin on 2026-10-16. Segment names need a
content hash, which `std` does not provide.

## 11. What UltraGraph provides, and what the store needs from it

**Provided:**
- Single-owner, deterministic apply.
- Permanent node positions (change 1), so every replica maps a replayed mutation to the same position.
- Algorithms on current data without a freeze, in the Live state.
- `freeze` from Live, which packs the rows without sorting. That is the flush from the in-memory level
  to a CSR segment.

**Specified but not yet implemented:** `materialize`, which turns a projection into an owned frozen
copy. It is group 6 of `openspec/changes/ultragraph-live-state-maintained-views/`.

**Needed:**
- A codec for mutations over the node payload `N` and the edge payload `W`. The `Storable` trait in
  `deep_causality_context` is the precedent.
- `compact_ids` as a sequenced operation, or as an operation allowed only at a checkpoint boundary.
- A fixed record layout with payloads held by reference.

## 12. Risks and open questions

- **Per-core throughput** of the graph LSM is unmeasured; stage 1 exists to measure it. Every
  estimate in §8.8 rests on it, and it decides whether the grid of §8.3–8.6 is ever needed.
- **Determinism and the simulator.** Deterministic apply forbids hash-map iteration order and any
  clock other than the primary's stamp. A simulator took TigerBeetle years to mature.
- **The epoch barrier.** A slow shard delays visibility across shards, though not single-shard writes.
- **Declared write sets.** Write sets must be known before sequencing; a transaction whose write set
  depends on a read takes OLLP's reconnaissance step.
- **The ULID index** adds a hop for clients that know only ULIDs.
- **Re-gridding** beyond the virtual grid chosen at the start moves all data.
- **Garbage collection** in shared storage must never delete an object a retained manifest references.
- **The distributed analytics engine** is a project of its own.
- **Unmeasured barriers.** Directory fsync with `F_FULLFSYNC` on macOS is unverified, and the barrier
  cost on Linux NVMe with power-loss protection is unmeasured.
- **Iggy** is reassessed as a dependency after its 1.0 release.

## Sources

UltraGraph and DeepCausality (this repository):
- `openspec/notes/ultragraph/RadixGraphUpdate.md`
- `openspec/notes/ultragraph/BenchAfterChange2LiveView/BenchAfterChange2LiveView.md`
- `openspec/changes/ultragraph-live-state-maintained-views/`
- `deep_causality_context_store/src/traits/context_storage.rs`, `context_storage_stream.rs`,
  `substrate.rs`

Papers:
- Pradeep Kumar, H. Howie Huang. *GraphOne: A Data Store for Real-time Analytics on Evolving
  Graphs*. ACM Transactions on Storage 15(4), 2019. [10.1145/3364180](https://doi.org/10.1145/3364180).
  Code: [github.com/the-data-lab/GraphOne](https://github.com/the-data-lab/GraphOne).
- Alexander Thomson et al. *Calvin: Fast Distributed Transactions for Partitioned Database Systems*.
  SIGMOD 2012. [PDF](https://www.cs.princeton.edu/courses/archive/fall19/cos418/papers/calvin.pdf).
- Xiaowei Zhu et al. *LiveGraph: A Transactional Graph Storage System with Purely Sequential Adjacency
  List Scans*. PVLDB 13(7), 2020. [arXiv 1910.05773](https://arxiv.org/abs/1910.05773). Code:
  [github.com/thu-pacman/LiveGraph](https://github.com/thu-pacman/LiveGraph).
- Libin Zhou et al. *GTX* (SIGMOD 2025). [arXiv 2405.01418](https://arxiv.org/abs/2405.01418).
- *BACH*. PVLDB 18(5). [PDF](https://www.vldb.org/pvldb/vol18/p1509-miao.pdf).
- *LSMGraph*. PACMMOD 2(6), 2024. [arXiv 2411.06392](https://arxiv.org/abs/2411.06392).
- Dean De Leo, Peter Boncz. *Teseo and the Analysis of Structural Dynamic Graphs*. PVLDB 14(6) and its
  errata. [PDF](http://www.vldb.org/pvldb/vol14/p1053-leo.pdf).
- Haoxuan Xie et al. *RadixGraph*. SIGMOD 2026. [arXiv 2601.01444](https://arxiv.org/abs/2601.01444).
- Wenting Zheng, Stephen Tu, Eddie Kohler, Barbara Liskov. *Fast Databases with Fast Durability and
  Recovery Through Multicore Parallelism* (SiloR). OSDI 2014.
- Weston Pace et al. *Lance: Efficient Random Access in Columnar Storage*. [arXiv 2504.15247](https://arxiv.org/abs/2504.15247).

Systems:
- Apache Iggy: [github.com/apache/iggy](https://github.com/apache/iggy) at `fe3b4269`;
  [durability](https://iggy.apache.org/docs/clustering/durability),
  [VSR](https://iggy.apache.org/docs/clustering/vsr/),
  [0.9.0 release](https://iggy.apache.org/blogs/2026/09/21/release-0.9.0),
  [benchmarks](https://benchmarks.iggy.apache.org), issues
  [#4284](https://github.com/apache/iggy/issues/4284),
  [#4438](https://github.com/apache/iggy/issues/4438),
  [#4294](https://github.com/apache/iggy/issues/4294),
  [#4436](https://github.com/apache/iggy/issues/4436),
  [discussion #4440](https://github.com/apache/iggy/discussions/4440).
- TigerBeetle: [performance](https://docs.tigerbeetle.com/concepts/performance/),
  [safety](https://docs.tigerbeetle.com/concepts/safety/),
  [upgrading](https://docs.tigerbeetle.com/operating/upgrading/),
  [vsr.md](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/vsr.md),
  [data_file.md](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/internals/data_file.md),
  [homepage](https://tigerbeetle.com/), [Jepsen 0.16.11](https://jepsen.io/analyses/tigerbeetle-0.16.11),
  [A trillion transactions](https://tigerbeetle.com/blog/2026-03-19-a-trillion-transactions/)
  (2026-03-19) and the talk it links ([youtu.be/y2_BqkKTbD8](https://youtu.be/y2_BqkKTbD8?t=999)).
- Aeron: [cluster overview](https://aeron.io/docs/aeron-cluster/overview/),
  [on sharding](https://aeron.io/docs/aeron-cluster/on-sharding/),
  [cluster component model](https://github.com/real-logic/aeron/wiki/Cluster-Component-Model).
- LMAX: Martin Fowler, [The LMAX Architecture](https://martinfowler.com/articles/lmax.html).
- Lance: [github.com/lance-format/lance](https://github.com/lance-format/lance) at `d1ba47e1`;
  [transactions](https://lance.org/format/table/transaction/),
  [performance guide](https://lance.org/guide/performance/),
  [MemWAL](https://lance.org/format/table/mem_wal/), issues
  [#5194](https://github.com/lance-format/lance/issues/5194),
  [#8293](https://github.com/lance-format/lance/issues/8293),
  [#9608](https://github.com/lance-format/lance/issues/9608),
  [#9762](https://github.com/lance-format/lance/issues/9762);
  [metadata benchmark](https://lancedb.com/blog/a-metadata-benchmark-of-lance-delta-lake-and-iceberg-on-s3);
  [lance-graph](https://github.com/lance-format/lance-graph).
- TuringDB: [snapshots](https://docs.turingdb.ai/concepts/snapshots),
  [github.com/turing-db/turingdb](https://github.com/turing-db/turingdb) at `fc9539336`.
- Spark GraphX: [EdgePartition2D](https://spark.apache.org/docs/latest/api/scala/org/apache/spark/graphx/PartitionStrategy$$EdgePartition2D$.html).
- Memgraph: [configuration](https://memgraph.com/docs/database-management/configuration),
  [transactions](https://memgraph.com/docs/fundamentals/transactions),
  [replication](https://memgraph.com/docs/clustering/replication).
- Kuzu: [CIDR 2023 paper](https://www.cidrdb.org/cidr2023/papers/p48-jin.pdf).
- Neo4j: [clustering architecture](https://neo4j.com/docs/operations-manual/current/clustering/high-availability/architecture/).
- Dgraph: [Jepsen 1.1.1](https://jepsen.io/analyses/dgraph-1.1.1),
  [forum thread on write scaling](https://discuss.dgraph.io/t/dgraph-cant-horizontally-scale-write-speeds/5882?page=2),
  [PR #9762](https://github.com/dgraph-io/dgraph/pull/9762).
- NebulaGraph: [storage service, 3.8.0](https://docs.nebula-graph.io/3.8.0/1.introduction/3.nebula-graph-architecture/4.storage-service/).

## Appendix A: the durability barrier on the benchmark machine

Measured on 2026-10-16 on the benchmark machine (Apple M3 Max, 16 cores, 128 GB, macOS 27.0, APFS on
the internal SSD) with the script below:
- each iteration appends one buffer to a file opened with `O_APPEND` and then applies the barrier;
- 300 iterations for 64 B and 4 KiB, 60 for 1 MiB.

| Barrier | Size | Median | p99 |
|---|---:|---:|---:|
| none | 64 B | 4.6 µs | 9.0 µs |
| none | 4,096 B | 7.9 µs | 11.4 µs |
| none | 1,048,576 B | 73.2 µs | 1,114.7 µs |
| `fsync` | 64 B | 25.1 µs | 47.0 µs |
| `fsync` | 4,096 B | 40.6 µs | 59.1 µs |
| `fsync` | 1,048,576 B | 249.2 µs | 6,396.0 µs |
| `F_FULLFSYNC` | 64 B | 5,522.0 µs | 9,119.2 µs |
| `F_FULLFSYNC` | 4,096 B | 5,553.5 µs | 8,575.5 µs |
| `F_FULLFSYNC` | 1,048,576 B | 6,417.7 µs | 8,538.7 µs |

```python
import os, fcntl, time, statistics, sys
path = sys.argv[1]
def run(n, size, mode):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_APPEND, 0o644)
    buf = b"x" * size
    lat = []
    for _ in range(n):
        t = time.perf_counter()
        os.write(fd, buf)
        if mode == "fullfsync":
            fcntl.fcntl(fd, fcntl.F_FULLFSYNC)
        elif mode == "fsync":
            os.fsync(fd)
        lat.append((time.perf_counter() - t) * 1e6)
    os.close(fd)
    lat.sort()
    return statistics.median(lat), lat[int(0.99 * len(lat)) - 1]
for mode in ("none", "fsync", "fullfsync"):
    for size in (64, 4096, 1 << 20):
        med, p99 = run(300 if size < (1 << 20) else 60, size, mode)
        print(f"{mode:9s} append {size:>8d} B  median {med:9.1f} us  p99 {p99:9.1f} us")
os.remove(path)
```

## Appendix B: Lance single-row commits on the benchmark machine

Measured on 2026-10-16 on the benchmark machine with `pylance` 13.0.0, data storage version 2.2, on
APFS. No fsync is involved, because Lance issues none.

- **First script.** It creates a dataset with stable row ids, then times:
  - 300 single-row `insert` calls on one dataset handle;
  - 100 single-row `delete` calls by predicate, with no index;
  - 100 single-row `merge_insert` upserts on `id`.
- **Second script.** It times 5,000 single-row `insert` calls and reports the median per 1,000
  commits.

| Operation | Median |
|---|---:|
| Append, after about 300 fragments | 1.87 ms |
| Append, commits 1–1,000 | 2.07 ms |
| Append, commits 4,001–5,000 | 5.40 ms |
| Delete by predicate | 11.1 ms |
| `merge_insert` upsert | 9.9 ms |

The append median rises by about 0.8 ms per 1,000 uncompacted fragments. 300 appends left 301
fragments and 301 data files, plus one manifest and one `.txn` file per version.
