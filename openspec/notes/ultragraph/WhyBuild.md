# Why build a persistent UltraGraph store

* Date: 2026-10-16
* Status: decided
* Decided by Marvin on 2026-10-16: build a persistent graph store on UltraGraph, in a dedicated
  repository, specified in `openspec/notes/ultragraph/PersistentStoreStage1.md`.
* Grounded in:
  - DeepBrain's design and its current schema, outside this repository;
  - the survey of embedded graph stores in `openspec/notes/ultragraph/PersistentStoreStage1Bench.md`;
  - the evaluations in `openspec/notes/ultragraph/PersistentStore.md`;
  - the product documentation cited under Sources, read on 2026-10-16.

## Summary

1. **No available graph store meets DeepBrain's requirements without a disqualifier** (§3).
2. **A log-first store gains more than speed.** DeepBrain's current backend offers snapshot
   isolation and no ordered commit log, so DeepBrain's design carries four coordination mechanisms
   to make up for it. A store with one ordered log and deterministic apply makes all four
   unnecessary (§4).
3. **Building has a price.** The store needs secondary indexes, transactions with checks, a change
   feed and erasure of personal data, beyond the log and the checkpoints (§5).
4. **The acceptance test exists.** Through a DeepBrain adapter, the store must pass DeepBrain's
   conformance kit and then beat the figures DeepBrain measured for its current backend on the same
   machine (§6).

## 1. What DeepBrain requires of a store

DeepBrain hands every backend four catalogues: a logical model, the derivations that turn facts into
keys, the questions it answers with a cost contract for each, and the invariants it holds. A backend
decides the physical encoding, the indexes and the plans, and passes the conformance kit. From those
catalogues and the current schema, a store must provide:

1. **Equality keys with many values per node.** Each node carries the closure of its groups (up to 16
   codes) and a composite of code and kind, so "this kind within this group" is one index lookup.
2. **Ordered range indexes over integers and lists of integers:** validity start times, time periods
   and S2 cells, read as ranges.
3. **A unique identity index with upsert,** one node per identity.
4. **Walks in both directions,** filtered by group at every level and pruned there, at a cost that
   grows with the walked set.
5. **Multi-statement transactions,** to which a composing layer can add writes, and invariants whose
   concurrent violations conflict.
6. **One ordered change feed per namespace:** gapless, without duplicates, and respecting causality.
7. **Personal values kept in one place** so that erasure reaches every copy.
8. **Cost contracts:** each question grows with its answer, with its walked set, or stays bounded.
9. **Performance beyond the current backend's measured figures** (§6).

## 2. Exclusions

Marvin excluded these before the survey:
- **Dgraph**, DeepBrain's current backend: stalled development and many unfixed bugs.
- **Neo4j:** its free edition is effectively a trial.
- **Apache AGE:** its PostgreSQL backend makes graph traversal a series of heavy joins.
- **SurrealDB:** Marvin's own benchmark found no meaningful difference from Dgraph, and version 3
  only caught up with the industry's baseline.
- **Archived projects,** Kuzu among them.

## 3. The candidates

| Candidate | Verdict | Reason |
|---|---|---|
| LadybugDB | Does not meet the requirements | "Every node or relationship can have only one label", so a node cannot carry its group closure as labels. Its documented indexes are the primary-key index (hash or ART) and automatic zone maps, so requirements 1 to 3 become column scans and break the cost contracts. It is also single-node with no replication, answers Cypher only, and links a prebuilt C++ library |
| Grafeo | Closest, not ready | It supports labels, edge properties, snapshot isolation by default and serializable transactions on request. But in 0.6.0-beta.1 a commit succeeds after a failed WAL write (#498); its default durability is `Batch`, not per commit; its group-commit writer is not yet used by the engine; its property index was not scoped to snapshots at 0.5.42; its range indexes are unconfirmed; and it holds the whole graph in one process's memory |
| Memgraph | Does not meet the requirements | Snapshot isolation at most; by default its WAL is fsynced once per 100,000 transactions |
| NebulaGraph | Does not meet the requirements | Each edge is stored in both endpoints' partitions, and per its 3.x documentation (seen in a search excerpt) the two copies are kept only eventually consistent |
| ArcadeDB | Not proven | Read committed by default; JVM only; no flush at commit by default; its fit for the indexes of requirements 1 to 3 was not checked |
| FalkorDB | Does not meet the requirements | SSPL-1.0; its embedded form starts a Redis server out of process; Redis's default durability syncs once per second |
| TuringDB | Does not meet the requirements | No WAL, a commit is visible before it is durable, and its licence is the Business Source License |
| TerminusDB, JanusGraph | Do not meet the requirements | TerminusDB is a server only; JanusGraph's BerkeleyDB backend has been frozen since 2018 |

`PersistentStoreStage1Bench.md` §2 records the survey of embedded stores in full, with the revision
of each fact.

## 4. What building removes

DeepBrain's current backend provides snapshot isolation and no ordered commit log. DeepBrain's design
makes up for each with a mechanism of its own:

| Mechanism in DeepBrain's current design | Why it exists | In a log-first store |
|---|---|---|
| Every transaction that checks an invariant also writes a guard key on the invariant's subject, so two such transactions conflict and one retries | Snapshot isolation lets two transactions each keep an invariant alone and break it together | Apply is serial and checks the invariant at the transaction's position (S1-TX-2); no guard, no retry |
| Every write commits an outbox record, and a sequencer numbers the records after commit to form the change feed | The backend has no ordered commit log to read the feed from | The log is the feed (S1-FEED-1) |
| An exclusive operation (a reorganisation, an index change, a declaration change) pauses the namespace: a marker, a write deadline, a wait, then detection and repair of writes that arrived late | An index change cannot fence the writes around it | An exclusive operation is one more operation in the log, taking effect at its position (S1-TX-6) |
| Reads that need a complete snapshot avoid indexes that every write touches | An index read under concurrent writes was measured missing commits below its start | Indexes change in the same apply step as their entity, so a read at a position sees exactly that position (S1-IDX-5) |

The decision rests mainly on this table. Speed matters, but a store that never needs these four
mechanisms removes a class of coordination code from DeepBrain, along with the failure modes that
come with it.

## 5. What building requires

The log, checkpoints, recovery and the simulator were already specified. DeepBrain's requirements add
these to the stage-1 draft:

| Requirement | Draft |
|---|---|
| Atomic transactions inside a group-commit batch; checks and host invariants evaluated in apply; upsert; composing writes; operation numbers as commit results; exclusive operations in the log | S1-TX-1 to S1-TX-6 |
| Index declarations through extractor traits; unique, equality (with set sizes) and ordered indexes; indexes that match the applied prefix; index changes in the log; indexes that survive recovery | S1-IDX-1 to S1-IDX-7 |
| Lookups, intersections and filters; root then filter; walks pruned at every level; absence filters | S1-QRY-1 to S1-QRY-4 |
| The log as a gapless, ordered feed; snapshot plus tail; feed entries carrying personal values by reference | S1-FEED-1 to S1-FEED-3 |
| Declarations as store state, one namespace per engine, retraction that keeps history, personal values outside the immutable files, erasure | S1-DECL-1, S1-NS-1, S1-RET-1, S1-PII-1, S1-PII-2 |
| Natural keys | S1-ID-5 |

## 6. The acceptance test

1. **Conformance first.** Through a DeepBrain adapter, the store passes DeepBrain's conformance kit.
   For every question it gives the reference evaluator's answer (S1-CONF-1).
2. **Then the figures.** On the benchmark machine (Apple M3 Max, 16 cores, 128 GB), the store beats
   what DeepBrain measured for Dgraph v25.4.0, standalone in Docker with 8 CPUs and 8 GB, on
   2026-10-12 (S1-MEAS-4):
   - 2,488 transactions per second for identity upserts, with 16 writers;
   - about 130,000 quads per second on the hottest field;
   - 2.6 ms and 21 ms for "a kind within a group" over 10,000 and 100,000 nodes;
   - 0.4 ms and 12 ms for a group filter inside a walk over 50 and 5,000 walked nodes;
   - 0.79 ms for a recursion pruned at every level.

   Every comparison states the incumbent's container limits, and both stores' durability and
   isolation.

## 7. Risks of building

- **Time.** TigerBeetle took years to mature one deterministic state machine with a simulator.
- **Indexes and transactions with checks are new code,** beyond UltraGraph's adjacency.
- **Maturity.** An established store has years of production use behind it, and this store has
  none.

**What reduces the risks:**
- **The staged plan** (`PersistentStore.md` §10): stage 1 runs in one process and is measured before
  any server or cluster exists.
- **The conformance kit** checks correctness against an independent reference from the first day.
- **The deterministic simulator** sweeps faults over every storage operation (S1-SIM-7).
- **The TDD protocol** audits the suite against named defects before implementation
  (`PersistentStoreStage1.md` §20).

## 8. What would reopen the decision

- **Grafeo** fixes the commit-after-failed-WAL defect (#498), makes per-commit durability its
  default, wires its group commit into the engine, scopes its indexes to snapshots, and confirms
  range indexes. It would then be re-evaluated, first as DeepBrain's open adapter.
- **LadybugDB** adds several labels per node and secondary range indexes.
- **Another store appears** that provides serializable transactions, an ordered change feed and the
  index types of requirements 1 to 3, under a licence compatible with DeepBrain's open core.

## Sources

- `openspec/notes/ultragraph/PersistentStoreStage1.md`, `PersistentStoreStage1Bench.md`,
  `PersistentStore.md` (this repository).
- LadybugDB documentation: [home](https://docs.ladybugdb.com/),
  [indexes](https://docs.ladybugdb.com/cypher/indexes),
  [create table](https://docs.ladybugdb.com/cypher/data-definition/create-table), read 2026-10-16.
- Grafeo: [GrafeoDB/grafeo](https://github.com/GrafeoDB/grafeo) at `a966850` (`CHANGELOG`, #498;
  `crates/grafeo-engine/src/config.rs`).
- Memgraph: [configuration](https://memgraph.com/docs/database-management/configuration).
- NebulaGraph: [storage service, 3.8.0](https://docs.nebula-graph.io/3.8.0/1.introduction/3.nebula-graph-architecture/4.storage-service/).
- TuringDB: [snapshots](https://docs.turingdb.ai/concepts/snapshots).
- DeepBrain's design, its current schema and its measurements of Dgraph v25.4.0 (2026-10-12) are
  outside this repository.
